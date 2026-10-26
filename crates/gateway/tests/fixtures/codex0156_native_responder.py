"""Attempt-local, independent HTTPS/SSE oracle for the stock 0.156.1 runtime.

No provider credentials are read. The only accepted authorization is a synthetic
sentinel installed in the separate gateway test process. No custom model list is
served and no native request is rewritten.
"""
import argparse
import json
import os
from pathlib import Path
import ssl
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def write_json(path, value):
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, indent=2))
    temporary.replace(path)


def events(items, response_id):
    response = {'id': response_id, 'object': 'response', 'status': 'in_progress', 'output': []}
    result = [('response.created', {'response': response})]
    for index, item in enumerate(items):
        initial = dict(item)
        if item['type'] == 'custom_tool_call':
            initial['input'] = ''
        elif item['type'] == 'message':
            initial['content'] = []
        result.append(('response.output_item.added', {'output_index': index, 'item': initial}))
        if item['type'] == 'custom_tool_call':
            result.append(('response.custom_tool_call_input.delta', {
                'output_index': index, 'item_id': item['id'], 'delta': item['input']}))
            result.append(('response.custom_tool_call_input.done', {
                'output_index': index, 'item_id': item['id'], 'input': item['input']}))
        else:
            part = item['content'][0]
            result.append(('response.content_part.added', {
                'output_index': index, 'item_id': item['id'], 'content_index': 0,
                'part': {'type': 'output_text', 'text': '', 'annotations': []}}))
            result.append(('response.output_text.delta', {
                'output_index': index, 'item_id': item['id'], 'content_index': 0, 'delta': part['text']}))
            result.append(('response.output_text.done', {
                'output_index': index, 'item_id': item['id'], 'content_index': 0, 'text': part['text']}))
            result.append(('response.content_part.done', {
                'output_index': index, 'item_id': item['id'], 'content_index': 0, 'part': part}))
        result.append(('response.output_item.done', {'output_index': index, 'item': item}))
    response = dict(response, status='completed', output=items,
                    usage={'input_tokens': 10, 'output_tokens': 5, 'total_tokens': 15})
    result.append(('response.completed', {'response': response}))
    return ''.join('event: ' + kind + '\ndata: ' + json.dumps(dict(payload, type=kind)) + '\n\n'
                   for kind, payload in result).encode()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve()
    root.mkdir(parents=True, exist_ok=True)
    certificate = root / 'certificate.pem'
    key = root / 'key.pem'
    subprocess.run(['/usr/bin/openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes',
                    '-keyout', str(key), '-out', str(certificate), '-days', '1',
                    '-subj', '/CN=chatgpt.com', '-addext', 'subjectAltName=DNS:chatgpt.com'],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    key.chmod(0o600)
    lock = threading.Lock()
    requests = []

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def do_GET(self):
            with lock:
                with (root / 'discovery.jsonl').open('a') as out:
                    out.write(json.dumps({'method': 'GET', 'path': self.path}) + '\n')
            self.send_error(404, 'No fixture model catalogue')

        def do_POST(self):
            raw = self.rfile.read(int(self.headers.get('Content-Length', '0')))
            body = json.loads(raw)
            with lock:
                index = len(requests)
                requests.append(body)
                write_json(root / ('request-%03d.json' % index), {
                    'path': self.path, 'host': self.headers.get('Host'), 'body': body,
                    'observed_monotonic': time.monotonic(),
                    'request_headers': {k: v for k, v in self.headers.items() if k.lower() not in ('authorization', 'chatgpt-account-id')},
                    'synthetic_auth': self.headers.get('Authorization') == 'Bearer native0156-synthetic-token',
                    'synthetic_account': self.headers.get('ChatGPT-Account-Id') == 'native0156-synthetic-account'})
            try:
                assert self.path == '/backend-api/codex/responses', self.path
                assert self.headers.get('Host') == 'chatgpt.com'
                assert self.headers.get('Authorization') == 'Bearer native0156-synthetic-token'
                assert body['model'] == 'openai-codex/gpt-6-astra', body['model']
                scenario = json.loads((root / 'scenario.json').read_text())
                mode = scenario.get('mode', 'preflight')
                if mode == 'preflight':
                    answer = 'native0156 TLS verified'
                    items = [{'type': 'message', 'id': 'msg_preflight', 'role': 'assistant',
                              'status': 'completed', 'content': [
                                  {'type': 'output_text', 'text': answer, 'annotations': []}]}]
                else:
                    items = self.native_items(body, scenario)
                payload = events(items, 'resp_native0156_%d' % index)
                if mode == 'ambiguous' and any(i['type'] == 'message' for i in items):
                    payload = payload.split(b'event: response.completed')[0]
                write_json(root / ('response-%03d.json' % index), {
                    'response_id': 'resp_native0156_%d' % index, 'truncated': mode == 'ambiguous' and any(i['type'] == 'message' for i in items),
                    'items': items, 'bytes': len(payload), 'automatic_replay': False})
                self.send_response(200)
                self.send_header('Content-Type', 'text/event-stream')
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
                self.wfile.flush()
            except Exception as error:
                with (root / 'oracle-errors.jsonl').open('a') as out:
                    out.write(json.dumps({'index': index, 'error': repr(error)}) + '\n')
                self.send_error(422, str(error))

        def native_items(self, body, scenario):
            nonce = scenario['nonce']
            inputs = body['input']
            assert body.get('reasoning', {}).get('context') == 'all_turns', body.get('reasoning')
            assert not body.get('tools'), 'Responses Lite must advertise via additional_tools'
            advertisement = [item for item in inputs if item.get('type') == 'additional_tools']
            assert advertisement, 'missing real Responses Lite additional_tools'
            advertised = json.dumps(advertisement)
            assert 'exec_command' in advertised and 'exec' in advertised and 'grammar' in advertised, advertised
            serialized = json.dumps(inputs)
            if 'fault' in scenario:
                label = scenario['fault']
                assert nonce + ':F:' + label in serialized, 'missing fault prompt'
                call_id = 'call_' + nonce + '_' + label
                tool_input = 'text(await tools.exec_command(' + json.dumps({'cmd': scenario['command']}) + '))'
                outputs = [item for item in inputs if item.get('type') == 'custom_tool_call_output'
                           and item.get('call_id') == call_id]
                if not outputs:
                    return [{'type': 'custom_tool_call', 'id': 'ctc_' + label,
                             'call_id': call_id, 'name': 'exec', 'input': tool_input, 'status': 'completed'}]
                calls = [item for item in inputs if item.get('type') == 'custom_tool_call'
                         and item.get('call_id') == call_id]
                assert len(calls) == 1 and calls[0]['input'] == tool_input, calls
                assert len(outputs) == 1 and nonce + ':RESULT:' + label in json.dumps(outputs), outputs
                write_json(root / ('fault-' + label + '-oracle.json'), {
                    'call_id': call_id, 'input': tool_input, 'output': outputs[0]})
                return [{'type': 'message', 'id': 'msg_' + label, 'role': 'assistant',
                         'status': 'completed', 'content': [{'type': 'output_text',
                         'text': nonce + ':ANSWER:' + label, 'annotations': []}]}]
            turns = [n for n in range(1, 4) if nonce + ':T' + str(n) in serialized]
            assert turns, 'missing nonce turn prompt'
            turn = max(turns)
            # Validate history order from the actual next native request.
            previous = -1
            for prior in range(1, turn):
                for marker in [nonce + ':T' + str(prior), 'call_' + nonce + '_' + str(prior),
                               nonce + ':RESULT:T' + str(prior), nonce + ':A' + str(prior)]:
                    position = serialized.find(marker, previous + 1)
                    assert position > previous, ('lost/reordered history', marker)
                    previous = position
            call_id = 'call_' + nonce + '_' + str(turn)
            script = scenario['commands'][str(turn)]
            tool_input = 'text(await tools.exec_command(' + json.dumps({'cmd': script}) + '))'
            outputs = [item for item in inputs if item.get('type') == 'custom_tool_call_output'
                       and item.get('call_id') == call_id]
            if not outputs:
                return [{'type': 'custom_tool_call', 'id': 'ctc_' + str(turn),
                         'call_id': call_id, 'name': 'exec', 'input': tool_input, 'status': 'completed'}]
            calls = [item for item in inputs if item.get('type') == 'custom_tool_call'
                     and item.get('call_id') == call_id]
            assert len(calls) == 1 and calls[0]['input'] == tool_input, calls
            assert len(outputs) == 1 and nonce + ':RESULT:T' + str(turn) in json.dumps(outputs), outputs
            lines = (Path(scenario['workspace']) / 'markers').read_text().splitlines()
            assert lines == [nonce + ':T' + str(n) for n in range(1, turn + 1)], lines
            write_json(root / ('turn-%d-oracle.json' % turn), {
                'turn': turn, 'call_id': call_id, 'input': tool_input, 'output': outputs[0],
                'markers': lines, 'history_verified': True})
            return [{'type': 'message', 'id': 'msg_' + str(turn), 'role': 'assistant',
                     'status': 'completed', 'content': [{'type': 'output_text',
                     'text': nonce + ':A' + str(turn), 'annotations': []}]}]

    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    server.daemon_threads = True
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(certificate, key)
    server.socket = context.wrap_socket(server.socket, server_side=True)
    write_json(root / 'tls-ready.json', {'port': server.server_port, 'pid': os.getpid()})
    server.serve_forever(poll_interval=0.05)


if __name__ == '__main__':
    main()
