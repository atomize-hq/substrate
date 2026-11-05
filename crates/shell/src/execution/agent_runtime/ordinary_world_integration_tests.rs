// Candidate-correspondent, local component integration. This never qualifies
// installed authority, provider traffic, kernel cgroups or namespace isolation.
use super::super::host_session_authority::{schema::*, transition::*};
use super::*;
use std::os::fd::AsFd;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::process::CommandExt as _;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use world_api::WorldBackend as _;

const SERVICE_WORKER: &str =
    "service::ordinary_world_integration_tests::ordinary_configured_service_worker";
// Method-only hang bound: 9 × 30s callbacks plus the measured admission/prelude
// phase (129.7s observed;150s allocated), from delivery through native-ready.
const COMPONENT_STARTUP_BUDGET: Duration = Duration::from_secs(9 * 30 + 150);
// Three startup+unchanged20s terminal windows, plus180s for selected fixture
// setup/refusals/retry/cleanup. These are not product startup guarantees.
const COMPONENT_OUTER_BUDGET: Duration = Duration::from_secs(3 * (9 * 30 + 150 + 20) + 180);
const COMPONENT_POLICY: &str = r#"id: ordinary-world-component
world_fs:
  host_visible: true
  fail_closed: {routing: false}
  caged_required: false
  write: {enabled: true}
agents:
  allowed_backends: [cli:codex-host, cli:codex-world]
  world_dispatch:
    enabled: true
    allowed_backends: [cli:codex-world]
    allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]
    allowed_modes: [retained]
    max_live_retained_workers: 4
llm:
  allowed_backends: [cli:codex-host, cli:codex-world]
  constraints:
    routers: [substrate_gateway]
    providers: [openai]
    protocols: [openai.responses]
    auth_authorities: [codex_subscription]
  secrets:
    env_allowed: [SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID]
"#;

const NATIVE_FIXTURE: &str = r##"#!/usr/bin/python3
import hashlib, json, os, pathlib, sys, time, urllib.request
args = sys.argv[1:]
if args and args[0] in ('--version', '--help', 'features'):
    print({'--version':'codex 1.2.3','--help':'Usage: --add-dir --allow-dangerously-skip-permissions','features':'{"features":["add_dir"]}'}[args[0]])
    raise SystemExit(0)
root = pathlib.Path(__file__).resolve().parent
fork_phase = 'entry'
def record_fork_phase(phase, **facts):
    global fork_phase
    fork_phase = phase
    if args != ['app-server']: return
    try:
        with (root / 'fork-diagnostic.jsonl').open('a') as handle:
            handle.write(json.dumps(dict(phase=phase, pid=os.getpid(),
                parent_pid=os.getppid(), monotonic_ns=time.monotonic_ns(), **facts)) + '\n')
    except OSError:
        pass
if args == ['app-server']:
    record_fork_phase('entry', fixture_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest())
def report_component_native_failure(error_type, error, trace):
    import traceback
    # Static categories and fixture line numbers only; never copy exception text.
    error_variant = next((kind.__name__ for kind in
        (AssertionError, KeyError, ValueError, TypeError, json.JSONDecodeError,
         UnicodeDecodeError, FileNotFoundError, PermissionError, BrokenPipeError,
         OSError, urllib.error.HTTPError, urllib.error.URLError) if error_type is kind), 'unknown')
    record_fork_phase('fixture_failure', failed_phase=fork_phase,
        error_variant=error_variant,
        errno=error.errno if isinstance(error, OSError) and isinstance(error.errno, int) else None,
        http_status=error.code if isinstance(error, urllib.error.HTTPError) else None,
        fixture_line=next((f.lineno for f in reversed(traceback.extract_tb(trace))
            if f.filename == __file__), None))
    failure = dict(component_only=True, pid=os.getpid(), args=args,
        error_type=error_type.__name__, message=str(error),
        frames=[dict(file=f.filename, line=f.lineno, function=f.name)
            for f in traceback.extract_tb(trace)])
    try:
        with (root / 'native-failures.jsonl').open('a') as handle:
            handle.write(json.dumps(failure) + '\n')
    except OSError:
        pass
    # This is a real fixture protocol error, never a synthetic terminal success.
    # It also survives an unwritable diagnostic file through the owned stream.
    print(json.dumps(dict(type='error', message=json.dumps(failure))), flush=True)
    sys.__excepthook__(error_type, error, trace)
sys.excepthook = report_component_native_failure
with (root / 'native-entries.jsonl').open('a') as handle:
    handle.write(json.dumps(dict(component_only=True, pid=os.getpid(), args=args)) + '\n')
log = root / 'native-observations.jsonl'
old = log.read_text().splitlines() if log.exists() else []
index = len(old) + 1
home = pathlib.Path(os.environ['CODEX_HOME'])
config = (home / 'config.toml').read_text()
assert 'model = "synthetic-worker-model"' in config
assert 'model_provider = "substrate-managed-gateway"' in config
assert not (home / 'auth.json').exists()
for key in ('OPENAI_API_KEY','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID'):
    assert not os.environ.get(key), key
history = home / 'fixture-history'
if args == ['app-server']:
    import uuid
    assert history.is_file(), 'Fork must consume the held original source history'
    assert not os.environ.get('SUBSTRATE_ORDINARY_HOME_HANDOFF')
    assert str(home).startswith('/proc/self/fd/')
    assert pathlib.Path(os.environ['SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH']).read_text().strip()==str(os.getpid())
    origin = __import__('re').search(r'base_url = "(http://127[.]0[.]0[.]1:[0-9]+)/v1"', config).group(1)
    source_bytes=history.read_bytes()
    source_id=source_bytes.decode().splitlines()[0]
    source_meta=history.stat(); home_meta=home.stat()
    child_id=None; child_file=None
    record_fork_phase('history_ready')
    loop_end = 'stdin_eof'
    for line in sys.stdin:
        record_fork_phase('request_line', line_bytes=len(line.encode()),
            line_sha256=hashlib.sha256(line.encode()).hexdigest())
        message=json.loads(line); method=message.get('method'); params=message.get('params',{})
        if method=='initialize':
            record_fork_phase('initialize_received')
            result={'ready':True}
        elif method=='thread/fork':
            record_fork_phase('thread_fork_received')
            assert child_id is None, 'source consumption must occur once'
            assert params['threadId']==source_id
            assert params['persistExtendedHistory'] is True
            assert params['cwd']==os.getcwd()
            assert 'model' not in params and 'addDirs' not in params
            assert history.read_bytes()==source_bytes
            child_id='component-fork-'+uuid.uuid4().hex
            child_file=home/('fixture-child-'+child_id+'.json')
            child_record=dict(child_session=child_id,source_session=source_id,
                source_sha256=hashlib.sha256(source_bytes).hexdigest(),turns=0)
            with child_file.open('x') as handle: json.dump(child_record,handle)
            record_fork_phase('child_history_created')
            result={'thread':{'id':child_id}}
        elif method=='turn/start':
            record_fork_phase('turn_start_received')
            assert child_id and params['threadId']==child_id
            assert params['cwd']==os.getcwd()
            assert 'model' not in params and 'addDirs' not in params
            print(json.dumps({'jsonrpc':'2.0','method':'turn/started',
                'params':{'threadId':child_id,'turn':{'id':'component-fork-turn'}}}),flush=True)
            assert urllib.request.urlopen(origin+'/fixture-native',timeout=3).read()==b'ok'
            record_fork_phase('native_callback_complete')
            record=json.loads(child_file.read_text()); assert record['child_session']==child_id
            record['turns']=1; child_file.write_text(json.dumps(record))
            child_meta=child_file.stat()
            entry=dict(component_only=True,pid=os.getpid(),parent_pid=os.getppid(),args=args,
                cwd=os.getcwd(),home=str(home.resolve()),home_alias=str(home),
                home_device=home_meta.st_dev,home_inode=home_meta.st_ino,
                source_history=str(history.resolve()),source_history_device=source_meta.st_dev,
                source_history_inode=source_meta.st_ino,source_sha256=hashlib.sha256(source_bytes).hexdigest(),
                source_session=source_id,child_session=child_id,child_history=str(child_file.resolve()),
                child_history_device=child_meta.st_dev,child_history_inode=child_meta.st_ino,
                child_record=record,config=config,fixture_device=pathlib.Path(__file__).stat().st_dev,
                fixture_inode=pathlib.Path(__file__).stat().st_ino,
                fixture_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest())
            with (root/'fork-observations.jsonl').open('a') as handle: handle.write(json.dumps(entry)+'\n')
            (root/'ready-fork').write_text(child_id)
            record_fork_phase('native_ready')
            deadline=time.monotonic()+15
            while not (root/'release-fork').exists():
                assert time.monotonic()<deadline, 'owner did not release native Fork terminal gate'
                time.sleep(0.01)
            record_fork_phase('terminal_release_seen')
            assert history.read_bytes()==source_bytes, 'Fork must not fabricate/copy/overwrite source history'
            print(json.dumps({'jsonrpc':'2.0','method':'turn/completed',
                'params':{'threadId':child_id,'turn':{'id':'component-fork-turn','status':'completed'}}}),flush=True)
            result={'turn':{'id':'component-fork-turn','status':'completed'}}
        elif method=='shutdown':
            record_fork_phase('shutdown_received')
            result={}
        elif method=='exit':
            loop_end = 'exit_requested'
            break
        else: continue
        print(json.dumps({'jsonrpc':'2.0','id':message['id'],'result':result}),flush=True)
        record_fork_phase('response_written', method_kind=method)
    record_fork_phase(loop_end)
    raise SystemExit(0)
if index == 1:
    assert 'resume' not in args
    history.write_text('thread-ordinary-world-component')
else:
    assert 'resume' in args
    pos = args.index('resume')
    assert args[pos + 1] == 'thread-ordinary-world-component'
    assert history.read_text() == 'thread-ordinary-world-component' + ''.join('\nturn-%s' % n for n in range(1, index))
# Every process mutates this same physical history without deleting/recreating it.
with history.open('a') as handle:
    handle.write('\nturn-%s' % index)
expected_history = 'thread-ordinary-world-component' + ''.join('\nturn-%s' % n for n in range(1, index + 1))
assert history.read_text() == expected_history
assert not os.environ.get('SUBSTRATE_ORDINARY_HOME_HANDOFF')
assert str(home).startswith('/proc/self/fd/'), 'every ordinary native requires a child-held handle'
assert '--last' not in args and '--fork-session' not in args
prompt = sys.stdin.read()
assert ('ordinary component turn %s' % index) in prompt
import re
origin = re.search(r'base_url = "(http://127[.]0[.]0[.]1:[0-9]+)/v1"', config).group(1)
assert urllib.request.urlopen(origin + '/fixture-native', timeout=3).read() == b'ok'
attached = pathlib.Path(os.environ['SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH']).read_text().strip()
assert attached == str(os.getpid())
output = pathlib.Path(args[args.index('--output-last-message') + 1])
assert output.is_absolute() and output.parent == pathlib.Path(os.environ['TMPDIR'])
assert str(output).startswith('/tmp/')
output_text = 'ordinary SDK output turn %s' % index
with output.open('x') as handle:
    handle.write(output_text)
home_identity = home.stat()
history_identity = history.stat()
fixture_identity = pathlib.Path(__file__).stat()
entry = dict(cgroup_target=str(pathlib.Path(os.environ['SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH']).resolve()), index=index, pid=os.getpid(), parent_pid=os.getppid(), home_alias=str(home), output=str(output), output_text=output_text, args=args, prompt=prompt, cwd=os.getcwd(), home=str(home.resolve()),
    home_device=home_identity.st_dev, home_inode=home_identity.st_ino,
    history_device=history_identity.st_dev, history_inode=history_identity.st_ino,
    fixture_script=__file__, fixture_device=fixture_identity.st_dev, fixture_inode=fixture_identity.st_ino,
    fixture_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
    config=config, attached=attached, history=history.read_text())
with log.open('a') as handle:
    handle.write(json.dumps(entry) + '\n')
print(json.dumps(dict(type='thread.started' if index == 1 else 'thread.resumed', thread_id='thread-ordinary-world-component')), flush=True)
print(json.dumps(dict(type='turn.started', thread_id='thread-ordinary-world-component', turn_id='turn-%s' % index)), flush=True)
print(json.dumps(dict(type='turn.completed', thread_id='thread-ordinary-world-component', turn_id='turn-%s' % index)), flush=True)
(root / ('ready-turn-%s' % index)).write_text(str(os.getpid()))
deadline = time.monotonic() + 15
while not (root / ('release-turn-%s' % index)).exists():
    assert time.monotonic() < deadline, 'owner failed to release normal terminal gate'
    time.sleep(0.01)
"##;

const GATEWAY_FIXTURE: &str = r##"#!/usr/bin/python3
import hashlib, http.server, json, os, pathlib, re, sys
config = pathlib.Path(sys.argv[sys.argv.index('--config') + 1])
text = config.read_text()
port = int(re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M).group(1))
for key in ('OPENAI_API_KEY','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID'):
    assert not os.environ.get(key), key
with os.fdopen(int(os.environ['SUBSTRATE_LLM_AUTH_BUNDLE_FD']), 'rb') as handle:
    bundle = json.load(handle)
assert bundle['backend_id'] == 'cli:codex'
assert bundle['fields']['SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN'] == 'synthetic-ordinary-token'
assert bundle['fields']['SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID'] == 'synthetic-ordinary-account'
root = pathlib.Path(__file__).resolve().parent
with (root / 'gateway-observations.jsonl').open('a') as handle:
    handle.write(json.dumps(dict(pid=os.getpid(), port=port, config_sha256=hashlib.sha256(text.encode()).hexdigest(), bundle_checked=True)) + '\n')
class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/fixture-native':
            with (root / 'gateway-native-requests').open('a') as handle:
                handle.write('local-native-request\n')
        self.send_response(200 if self.path in ('/health','/fixture-native') else 404)
        self.end_headers()
        self.wfile.write(b'ok')
    def log_message(self, *args):
        pass
http.server.HTTPServer(('127.0.0.1', port), Handler).serve_forever()
"##;

const COMPONENT_CLAUDE_POLICY: &str = r#"id: ordinary-world-component
world_fs:
  host_visible: true
  fail_closed: {routing: false}
  caged_required: false
  write: {enabled: true}
agents:
  allowed_backends: [cli:codex-host, cli:claude_code-world]
  world_dispatch:
    enabled: true
    allowed_backends: [cli:claude_code-world]
    allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]
    allowed_modes: [retained]
    max_live_retained_workers: 4
llm:
  allowed_backends: [cli:codex-host, cli:claude_code-world]
  constraints:
    routers: [substrate_gateway]
    providers: [openai, anthropic]
    protocols: [openai.responses, anthropic.messages]
    auth_authorities: [codex_subscription, anthropic_api_key]
  secrets:
    env_allowed: [SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID, ANTHROPIC_API_KEY]
"#;

const CLAUDE_NATIVE_FIXTURE: &str = r##"#!/usr/bin/python3
import hashlib, json, os, pathlib, sys, time, urllib.request
args = sys.argv[1:]
root = pathlib.Path(__file__).resolve().parent
with (root / 'native-entries.jsonl').open('a') as handle:
    handle.write(json.dumps(dict(component_only=True, pid=os.getpid(), args=args)) + '\n')
log = root / 'native-observations.jsonl'
index = len(log.read_text().splitlines()) + 1 if log.exists() else 1
home = pathlib.Path(os.environ['CLAUDE_CONFIG_DIR'])
assert os.environ['HOME'] == str(home)
assert str(home).startswith('/proc/self/fd/')
assert not os.environ.get('SUBSTRATE_ORDINARY_HOME_HANDOFF')
assert (home / 'settings.json').read_text() == '{}\n'
assert not (home / 'auth.json').exists()
for key in ('OPENAI_API_KEY','ANTHROPIC_API_KEY','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID'):
    assert not os.environ.get(key), key
assert os.environ['ANTHROPIC_AUTH_TOKEN'] == 'substrate-managed-gateway'
model = os.environ['ANTHROPIC_MODEL']
assert model == 'claude-sonnet-4.5'
origin = os.environ['ANTHROPIC_BASE_URL']
assert urllib.request.urlopen(origin + '/fixture-native', timeout=3).read() == b'ok'
prompt = args[-1]
assert ('ordinary component turn %s' % index) in prompt
expected = ['--print','--output-format','stream-json','--model',model,
    '--permission-mode','bypassPermissions','--include-partial-messages']
if index > 1:
    expected += ['--resume','thread-ordinary-world-component']
expected += ['--verbose',prompt]
assert args == expected
history = home / 'fixture-history'
if index == 1:
    assert not history.exists()
    history.write_text('thread-ordinary-world-component')
else:
    assert history.read_text() == 'thread-ordinary-world-component' + ''.join('\nturn-%s' % n for n in range(1,index))
with history.open('a') as handle:
    handle.write('\nturn-%s' % index)
attached = pathlib.Path(os.environ['SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH']).read_text().strip()
assert attached == str(os.getpid())
home_identity, history_identity, fixture_identity = home.stat(), history.stat(), pathlib.Path(__file__).stat()
entry = dict(index=index,pid=os.getpid(),parent_pid=os.getppid(),args=args,prompt=prompt,cwd=os.getcwd(),
    cgroup_target=str(pathlib.Path(os.environ['SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH']).resolve()),
    home_alias=str(home),home=str(home.resolve()),home_device=home_identity.st_dev,home_inode=home_identity.st_ino,
    history_device=history_identity.st_dev,history_inode=history_identity.st_ino,history=history.read_text(),
    fixture_script=__file__,fixture_device=fixture_identity.st_dev,fixture_inode=fixture_identity.st_ino,
    fixture_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
    config=(home / 'settings.json').read_text(),attached=attached,model=model,endpoint=origin)
with log.open('a') as handle:
    handle.write(json.dumps(entry) + '\n')
print(json.dumps(dict(type='system',subtype='init',session_id='thread-ordinary-world-component')),flush=True)
print(json.dumps(dict(type='assistant',session_id='thread-ordinary-world-component',
    message=dict(content=[dict(type='text',text='ordinary SDK output turn %s' % index)]))),flush=True)
print(json.dumps(dict(type='result',subtype='success',is_error=False,
    result='ordinary SDK output turn %s' % index,session_id='thread-ordinary-world-component',
    duration_ms=1,duration_api_ms=1,num_turns=1,total_cost_usd=0)),flush=True)
(root / ('ready-turn-%s' % index)).write_text(str(os.getpid()))
deadline = time.monotonic() + 15
while not (root / ('release-turn-%s' % index)).exists():
    assert time.monotonic() < deadline, 'owner failed to release Claude normal terminal gate'
    time.sleep(0.01)
"##;

const CLAUDE_GATEWAY_FIXTURE: &str = r##"#!/usr/bin/python3
import hashlib, http.server, json, os, pathlib, re, sys
config = pathlib.Path(sys.argv[sys.argv.index('--config') + 1])
text = config.read_text()
port = int(re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M).group(1))
for key in ('ANTHROPIC_API_KEY','OPENAI_API_KEY','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID'):
    assert not os.environ.get(key), key
with os.fdopen(int(os.environ['SUBSTRATE_LLM_AUTH_BUNDLE_FD']), 'rb') as handle:
    bundle = json.load(handle)
assert bundle['backend_id'] == 'cli:claude_code'
assert bundle['fields']['SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY'] == 'synthetic-ordinary-anthropic-key'
root = pathlib.Path(__file__).resolve().parent
with (root / 'gateway-observations.jsonl').open('a') as handle:
    handle.write(json.dumps(dict(pid=os.getpid(), port=port, config_sha256=hashlib.sha256(text.encode()).hexdigest(), bundle_checked=True)) + '\n')
class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/fixture-native':
            with (root / 'gateway-native-requests').open('a') as handle:
                handle.write('local-native-request\n')
        self.send_response(200 if self.path in ('/health','/fixture-native') else 404)
        self.end_headers()
        self.wfile.write(b'ok')
    def log_message(self, *args):
        pass
http.server.HTTPServer(('127.0.0.1', port), Handler).serve_forever()
"##;

fn component_seal(
    root: &Path,
    home: &Path,
) -> config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1 {
    use config_projection::{
        ConfigProjectionCodecV1 as Codec, InstalledAcceptedHomeBootstrapHeadV1 as Head,
        InstalledAcceptedHomeBootstrapRecordV1 as Record, Timestamp,
    };
    let principal = crate::execution::install_bootstrap::current_unix_principal_and_home()
        .unwrap()
        .0;
    let PlatformPrincipalV1::Unix { account, uid } = principal else {
        panic!("Unix fixture")
    };
    assert_ne!(uid, 0);
    let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
        transport_api_types::InstallBootstrapContextV1::new_unix(
            home.to_str().unwrap(),
            &account,
            uid,
        )
        .unwrap(),
    )
    .unwrap();
    let mut record = Record {
        schema_version: 1,
        install_bootstrap_carrier: carrier.encode().unwrap(),
        host_context_commitment: carrier.host_context_commitment.clone(),
        intended_account: account,
        intended_uid: u64::from(uid),
        intended_gid: u64::from(unsafe { libc::getegid() }),
        accepted_home: config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
            File::open(home).unwrap().as_fd(),
        )
        .unwrap(),
        installed_at: Timestamp("2026-10-29T00:00:00.000000Z".into()),
        record_hash: String::new(),
    };
    let mut value = serde_json::to_value(&record).unwrap();
    value.as_object_mut().unwrap().remove("record_hash");
    record.record_hash = Codec::domain_sha256(
        "substrate.e3.installed-accepted-home-bootstrap.v1",
        &serde_json::json!({"record":value}),
    )
    .unwrap();
    let mut head = Head {
        schema_version: 1,
        head_record_hash: record.record_hash.clone(),
        predecessor_head_hash: None,
        updated_at: record.installed_at.clone(),
        head_hash: String::new(),
    };
    let mut value = serde_json::to_value(&head).unwrap();
    value.as_object_mut().unwrap().remove("head_hash");
    head.head_hash = Codec::domain_sha256(
        "substrate.e3.installed-accepted-home-bootstrap-head.v1",
        &serde_json::json!({"head":value}),
    )
    .unwrap();
    let source = root.join("bootstrap");
    std::fs::create_dir_all(source.join("records")).unwrap();
    for path in [&source, &source.join("records")] {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o750)).unwrap();
    }
    for (path, bytes, mode) in [
        (source.join("lock"), Vec::new(), 0o600),
        (
            source.join("active.json"),
            Codec::encode_canonical_json(&head).unwrap(),
            0o640,
        ),
        (
            source
                .join("records")
                .join(format!("{}.json", record.record_hash)),
            Codec::encode_canonical_json(&record).unwrap(),
            0o640,
        ),
    ] {
        std::fs::write(&path, bytes).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    std::fs::write(
        root.join("carrier.json"),
        serde_json::to_vec(&carrier).unwrap(),
    )
    .unwrap();
    let held = config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::from_component_source_for_test(&source, &carrier).unwrap();
    let mut malformed = carrier.clone();
    malformed.host_context_commitment = "invalid".into();
    assert!(config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::from_component_source_for_test(&source, &malformed).is_err());
    let other = root.join("other-home");
    std::fs::create_dir(&other).unwrap();
    let wrong_home = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
        transport_api_types::InstallBootstrapContextV1::new_unix(
            other.to_str().unwrap(),
            &record.intended_account,
            uid,
        )
        .unwrap(),
    )
    .unwrap();
    assert!(config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::from_component_source_for_test(&source, &wrong_home).is_err());
    assert!(config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::from_component_source_for_test(&other, &carrier).is_err());
    held
}

struct OwnedService(Child);
impl Drop for OwnedService {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            // The service starts its own process group; failure cleanup reaches
            // only children of this owned fixture, never an installed service.
            unsafe {
                libc::kill(-(self.0.id() as i32), libc::SIGKILL);
            }
            let _ = self.0.kill();
        }
        let waited = self.0.wait();
        if let Some(evidence) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") {
            use std::os::unix::process::ExitStatusExt as _;
            let evidence = PathBuf::from(evidence);
            let (code, signal, error) = match waited {
                Ok(status) => (status.code(), status.signal(), None),
                Err(error) => (None, None, Some(error.to_string())),
            };
            let record = serde_json::json!({"owned_service_pid":self.0.id(),
                "child_wait_exit_code":code,"child_wait_signal":signal,"child_wait_error":error,
                "failure_group_cleanup_path":std::thread::panicking()});
            if let Err(error) = std::fs::create_dir_all(&evidence).and_then(|()| {
                std::fs::write(evidence.join("service-owned-wait.json"), record.to_string())
            }) {
                eprintln!("could not retain owned service wait result: {error}");
            }
        }
    }
}

async fn wait_until(mut predicate: impl FnMut() -> bool, label: &str, budget: Duration) {
    let deadline = Instant::now() + budget;
    while !predicate() {
        assert!(Instant::now() < deadline, "timed out: {label}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn observe_terminal_gate(
    native_pid: u64,
    gateway_pid: u64,
    mut terminal_ready: impl FnMut() -> bool,
) {
    let deadline = Instant::now() + Duration::from_millis(100);
    loop {
        assert!(
            !terminal_ready(),
            "terminal became ready while the actual native exit gate remained closed"
        );
        assert!(
            PathBuf::from(format!("/proc/{native_pid}")).exists(),
            "native gate owner must remain live"
        );
        assert!(
            PathBuf::from(format!("/proc/{gateway_pid}")).exists(),
            "gateway custody must survive the native exit gate"
        );
        if Instant::now() >= deadline {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn observations(path: &Path) -> Vec<serde_json::Value> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

async fn frames(
    response: hyper::Response<hyper::body::Incoming>,
    observed: Arc<std::sync::Mutex<Vec<u8>>>,
) -> Vec<transport_api_types::ExecuteStreamFrame> {
    use http_body_util::BodyExt as _;
    let body = response.into_body().map_frame(move |frame| {
        if let Some(bytes) = frame.data_ref() {
            observed.lock().unwrap().extend_from_slice(bytes);
        }
        frame
    });
    // The caller owns one startup deadline, then the unchanged20s terminal
    // await after actual release. Collection retains every original byte/frame.
    let bytes = body.collect().await.unwrap().to_bytes();
    std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

// Failure-only projection: do not export protocol bodies, event data or error text.
fn retain_initial_fork_readiness_failure(
    terminal: &mut tokio::task::JoinHandle<Vec<transport_api_types::ExecuteStreamFrame>>,
    observed: &Arc<std::sync::Mutex<Vec<u8>>>,
    cwd: &Path,
    service_pid: u32,
) {
    use base64::Engine as _;
    use futures::FutureExt as _;
    use transport_api_types::ExecuteStreamFrame;

    // Source-defined wording is evidence of text only, never a typed SDK cause.
    let project_message = |message: &str| {
        let mut text = message.trim_end_matches(['\r', '\n']);
        let mut wrappers = Vec::new();
        for (prefix, label) in [
            ("member runtime failed: ", "member_runtime_failure"),
            ("backend error: ", "wrapper_backend_error"),
            (
                "ordinary home handoff terminal join failed; backend outcome: ",
                "ordinary_home_handoff_join_failure",
            ),
            ("backend error: ", "nested_wrapper_backend_error"),
        ] {
            if let Some(rest) = text.strip_prefix(prefix) {
                wrappers.push(label);
                text = rest;
            }
        }
        let mut cleanup_match = "unknown";
        for (suffix, label) in [
            (
                "; codex app-server cleanup failed (transport)",
                "sdk_redacted_cleanup_transport",
            ),
            (
                "; codex app-server cleanup failed (protocol)",
                "sdk_redacted_cleanup_protocol",
            ),
        ] {
            if let Some(rest) = text.strip_suffix(suffix) {
                cleanup_match = label;
                text = rest;
                break;
            }
        }
        let (wording_match, match_kind) = match text {
            "codex app-server handshake failed" => ("sdk_redacted_handshake", "exact"),
            "codex app-server rpc error" => ("sdk_redacted_rpc", "exact"),
            "codex backend error: timeout (details redacted when unsafe)" => {
                ("sdk_redacted_backend_timeout", "exact")
            }
            "codex completion task dropped" => ("sdk_completion_task_dropped", "exact"),
            "codex backend failed to resolve working directory" => {
                ("sdk_working_directory_unresolved", "exact")
            }
            "no session found" => ("sdk_no_session_found", "exact"),
            "session not found" => ("sdk_session_not_found", "exact"),
            "ordinary current-run home handoff refused" => {
                ("ordinary_home_handoff_refused", "exact")
            }
            "checked native settlement unavailable" => {
                ("checked_native_settlement_unavailable", "exact")
            }
            "transport channel closed unexpectedly" => ("mcp_channel_closed_wording", "exact"),
            _ if text.starts_with("server did not respond to initialize: timed out after ") => {
                ("mcp_initialize_timeout_wording", "prefix")
            }
            _ if text.starts_with("server did not respond to initialize: ") => {
                ("mcp_initialize_failure_wording", "prefix")
            }
            _ if text.starts_with("submission_outcome_unknown: native acknowledgment failed: ") => {
                ("native_acknowledgment_failure_wording", "prefix")
            }
            _ => ("unknown", "none"),
        };
        serde_json::json!({"bytes":message.len(),
            "sha256":format!("{:x}",Sha256::digest(message.as_bytes())),
            "static_allowlist_match":wording_match,"match_kind":match_kind,
            "static_cleanup_suffix_match":cleanup_match,"static_wrapper_matches":wrappers,
            "error_variant":"unknown","typed_sdk_cause":"unknown"})
    };
    let project_chunk = |chunk_b64: &str| match base64::engine::general_purpose::STANDARD
        .decode(chunk_b64)
    {
        Ok(bytes) => serde_json::json!({"decode":"ok","bytes":bytes.len(),
                "sha256":format!("{:x}",Sha256::digest(&bytes)),
                "whole_chunk_text_projection":std::str::from_utf8(&bytes).ok().map(project_message)}),
        Err(_) => serde_json::json!({"decode":"invalid_base64"}),
    };
    let project = |frame: &ExecuteStreamFrame| match frame {
        ExecuteStreamFrame::Start { .. } => serde_json::json!({"kind":"start"}),
        ExecuteStreamFrame::Stdout { chunk_b64, .. } => {
            serde_json::json!({"kind":"stdout","encoded_bytes":chunk_b64.len(),
                "encoded_sha256":format!("{:x}",Sha256::digest(chunk_b64.as_bytes())),
                "decoded_projection":project_chunk(chunk_b64)})
        }
        ExecuteStreamFrame::Stderr { chunk_b64, .. } => {
            serde_json::json!({"kind":"stderr","encoded_bytes":chunk_b64.len(),
                "encoded_sha256":format!("{:x}",Sha256::digest(chunk_b64.as_bytes())),
                "decoded_projection":project_chunk(chunk_b64)})
        }
        ExecuteStreamFrame::Event { event, .. } => {
            let wrapper_error = event.kind == substrate_common::agent_events::AgentEventKind::Alert
                && event.data.get("code").and_then(serde_json::Value::as_str)
                    == Some("agent_wrapper_error");
            let error_message = if wrapper_error {
                event
                    .data
                    .get("message")
                    .and_then(serde_json::Value::as_str)
                    .map(project_message)
            } else {
                None
            };
            serde_json::json!({"kind":"event","event_kind":event.kind,
                "wrapper_error_code_match":wrapper_error,"error_message":error_message})
        }
        ExecuteStreamFrame::Exit { exit, .. } => {
            serde_json::json!({"kind":"exit","exit":exit})
        }
        ExecuteStreamFrame::Error { message, .. } => {
            serde_json::json!({"kind":"error","error_variant":"unknown",
                "message_bytes":message.len(),
                "message_sha256":format!("{:x}",Sha256::digest(message.as_bytes())),
                "error_message":project_message(message)})
        }
    };
    // Inspect completion without adding a wait, timeout or native settlement claim.
    let join = if terminal.is_finished() {
        match terminal.now_or_never() {
            Some(Ok(frames)) => serde_json::json!({"state":"completed",
            "frames":frames.iter().map(&project).collect::<Vec<_>>()}),
            Some(Err(error)) => serde_json::json!({"state":"join_error",
            "variant":if error.is_panic() {"panic"} else if error.is_cancelled() {
                "cancelled"
            } else {"unknown"}}),
            None => serde_json::json!({"state":"not_ready","settlement":"unknown"}),
        }
    } else {
        serde_json::json!({"state":"unfinished","settlement":"unknown"})
    };
    let stream = match observed.lock() {
        Ok(bytes) => {
            let decoded: Vec<_> = bytes
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty())
                .map(
                    |line| match serde_json::from_slice::<ExecuteStreamFrame>(line) {
                        Ok(frame) => project(&frame),
                        Err(error) => serde_json::json!({"kind":"decode_error",
                        "category":match error.classify() {
                            serde_json::error::Category::Io => "io",
                            serde_json::error::Category::Syntax => "syntax",
                            serde_json::error::Category::Data => "data",
                            serde_json::error::Category::Eof => "eof",
                        },"bytes":line.len(),"sha256":format!("{:x}",Sha256::digest(line))}),
                    },
                )
                .collect();
            serde_json::json!({"bytes":bytes.len(),
                "sha256":format!("{:x}",Sha256::digest(bytes.as_slice())),"frames":decoded})
        }
        Err(_) => serde_json::json!({"read_error":"poisoned_lock"}),
    };
    let fixture = match std::fs::read_to_string(cwd.join("fork-diagnostic.jsonl")) {
        Ok(records) => {
            let records: Vec<_> = records
                .lines()
                .map(|line| {
                    serde_json::from_str::<serde_json::Value>(line)
                        .unwrap_or_else(|_| serde_json::json!({"read_error":"invalid_json"}))
                })
                .collect();
            serde_json::json!({"state":"read","records":records})
        }
        Err(error) => {
            serde_json::json!({"state":"unavailable","io_kind":format!("{:?}",error.kind())})
        }
    };
    let diagnostic = serde_json::json!({"component_only":true,"phase":"initial_fork_readiness",
        "service_pid":service_pid,"owned_stream_task":join,"stream":stream,"fixture":fixture,
        "fixture_sha256":format!("{:x}",Sha256::digest(NATIVE_FIXTURE.as_bytes())),
        "native_wait":"unknown","mcp_terminal_ack":"unknown"});
    // Retain the original failure before resuming its unwind through the fixture.
    eprintln!("initial Fork readiness diagnostic: {diagnostic}");
    if let Some(evidence) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") {
        let evidence = PathBuf::from(evidence);
        if let Err(error) = std::fs::create_dir_all(&evidence).and_then(|()| {
            std::fs::write(
                evidence.join("initial-fork-readiness-failure.json"),
                diagnostic.to_string(),
            )
        }) {
            eprintln!(
                "initial Fork diagnostic retention failed: {:?}",
                error.kind()
            );
        }
    }
}

struct ComponentAnthropicKey(Option<std::ffi::OsString>);
impl Drop for ComponentAnthropicKey {
    fn drop(&mut self) {
        match self.0.take() {
            Some(value) => std::env::set_var("ANTHROPIC_API_KEY", value),
            None => std::env::remove_var("ANTHROPIC_API_KEY"),
        }
    }
}

struct Fixture {
    _anthropic_key: Option<ComponentAnthropicKey>,
    _env: crate::execution::AuthorityEnvTestGuard,
    _temp: tempfile::TempDir,
    #[allow(
        dead_code,
        reason = "ordinary component fixture retains the held HSA authority through native selection and home custody assertions"
    )]
    authority: Arc<HostSessionAuthority>,
    context: NativeLaunchContextV1,
    home: PathBuf,
    configured: Arc<config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1>,
    _world_root: world::session::SharedRootDirOverrideGuard,
    component_cgroup: PathBuf,
}

fn fixture(apply: bool) -> Fixture {
    fixture_for_generation(apply, false)
}

fn fixture_for_generation(apply: bool, initial_zero: bool) -> Fixture {
    fixture_for_harness(apply, initial_zero, false)
}

fn fixture_for_harness(apply: bool, initial_zero: bool, claude: bool) -> Fixture {
    fixture_for_harness_with_narrowing(apply, initial_zero, claude, false)
}

fn ordinary_component_policy(claude: bool, narrowing: bool) -> String {
    let policy = if claude {
        COMPONENT_CLAUDE_POLICY
    } else {
        COMPONENT_POLICY
    };
    if narrowing {
        policy.replace(
            "    allowed_modes: [retained]",
            "    allowed_modes: [retained]\n    allow_capability_narrowing: true",
        )
    } else {
        policy.to_string()
    }
}

fn fixture_for_harness_with_narrowing(
    apply: bool,
    initial_zero: bool,
    claude: bool,
    narrowing: bool,
) -> Fixture {
    let env = crate::execution::AuthorityEnvTestGuard::preserve();
    // Both canonical keys are synthetic and supplied before the factory;
    // this fixture never falls through to an OS-account auth.json read.
    std::env::set_var(
        SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
        "synthetic-ordinary-token",
    );
    std::env::set_var(
        SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
        "synthetic-ordinary-account",
    );
    let anthropic_key =
        claude.then(|| ComponentAnthropicKey(std::env::var_os("ANTHROPIC_API_KEY")));
    if claude {
        std::env::set_var("ANTHROPIC_API_KEY", "synthetic-ordinary-anthropic-key");
    }
    std::env::remove_var("SUBSTRATE_WORLD_REQUEST_PROFILE");
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("accepted");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir(&home).unwrap();
    std::fs::create_dir(&workspace).unwrap();
    std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::create_dir(home.join("agents")).unwrap();
    std::fs::set_permissions(home.join("agents"), std::fs::Permissions::from_mode(0o700)).unwrap();
    let config = "world:\n  net:\n    filter: false\nagents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex-host\n";
    let mut inventory: serde_yaml::Value =
        serde_yaml::from_str(include_str!("../../../../../config/agents/codex.yaml")).unwrap();
    inventory["config"]["placements"]["host"]["cli"]["binary"] = "/bin/true".into();
    inventory["config"]["placements"]["world"]["selection"]["model"] =
        "synthetic-worker-model".into();
    for (path, bytes) in [
        (home.join("config.yaml"), config.as_bytes().to_vec()),
        (
            home.join("agents/codex.yaml"),
            serde_yaml::to_string(&inventory).unwrap().into_bytes(),
        ),
    ] {
        std::fs::write(&path, bytes).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let policy_path = home.join("policy.yaml");
    std::fs::write(&policy_path, ordinary_component_policy(claude, narrowing)).unwrap();
    std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let native_name = if claude { "claude" } else { "codex" };
    let native_fixture = if claude {
        CLAUDE_NATIVE_FIXTURE
    } else {
        NATIVE_FIXTURE
    };
    let binary = workspace.join(native_name);
    std::fs::write(&binary, native_fixture).unwrap();
    std::fs::write(temp.path().join("native-fixture-name"), native_name).unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    // Preserve the production descriptor. The existing availability probe sees
    // the local fixture, while only the private service test owns its execution.
    std::env::set_var("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &workspace);
    inventory["config"]["placements"]["world"]["cli"]["binary"] =
        "/var/lib/substrate/world-deps/bin/codex".into();
    std::fs::write(
        temp.path().join("native-fixture.sha256"),
        format!("{:x}", sha2::Sha256::digest(native_fixture.as_bytes())),
    )
    .unwrap();
    std::fs::write(
        home.join("agents/codex.yaml"),
        serde_yaml::to_string(&inventory).unwrap(),
    )
    .unwrap();
    if claude {
        let mut claude_inventory: serde_yaml::Value = serde_yaml::from_str(include_str!(
            "../../../../../config/agents/claude_code.yaml"
        ))
        .unwrap();
        claude_inventory["config"]["placements"]["world"]["cli"]["binary"] =
            binary.display().to_string().into();
        let path = home.join("agents/claude_code.yaml");
        std::fs::write(&path, serde_yaml::to_string(&claude_inventory).unwrap()).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let configured = Arc::new(component_seal(temp.path(), &home));
    let world_root = temp.path().join("worlds");
    std::fs::create_dir(&world_root).unwrap();
    std::fs::set_permissions(&world_root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let _world_root = world::SessionWorld::override_shared_root_dir_for_tests(world_root);
    let root = TrustedAuthorityRoot::open(&home).unwrap();
    let authority = Arc::new(HostSessionAuthority::from_trusted_root(root).unwrap());
    let root = authority.bootstrap().unwrap();
    let bootstrap = authority.bootstrap_home();
    let config = crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
        &workspace,
        &Default::default(),
        &bootstrap,
    )
    .unwrap();
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        &workspace, &bootstrap,
    )
    .unwrap();
    let inventory =
        crate::execution::agent_inventory::load_agent_inventory_snapshot_for_bootstrap_home_v3(
            &workspace, &policy, &bootstrap,
        )
        .unwrap();
    let native = inventory
        .resolve_selection(&config, None, None, AgentExecutionScope::Host)
        .unwrap();
    let requested = inventory
        .resolve_selection(
            &config,
            Some(if claude { "claude_code" } else { "codex" }),
            None,
            AgentExecutionScope::World,
        )
        .unwrap();
    let selections = OrdinaryStartSelectionsV1 { native, requested };
    let sources = CapturedOrdinarySourcesV1::capture(&config, &inventory, &selections).unwrap();
    let metadata = std::fs::metadata(&workspace).unwrap();
    let binding = WorkspaceBindingV1 {
        workspace_root: CanonicalDirectoryV1 {
            physical_path: workspace.to_str().unwrap().into(),
            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                device_id: metadata.dev(),
                inode: metadata.ino(),
            },
        },
        authority_store_root: root.bootstrap_home,
        authority_store_id: root.authority_store_id,
    };
    let descriptor = AgentDescriptorV1 {
        schema_version: 1,
        agent_id: selections.native.realized_agent_id.clone(),
        backend_id: selections.native.backend_id.clone(),
        backend_kind: RuntimeBackendKindV1::Codex,
        protocol: "substrate.agent.session".into(),
        execution_scope: AgentExecutionScopeV1::Host,
        binary_path: "/bin/true".into(),
    };
    let network =
        crate::execution::policy_snapshot::resolve_world_network_policy_for_bootstrap_home(
            &workspace, &bootstrap, &config,
        )
        .unwrap();
    let mut spec = crate::execution::policy_snapshot::world_spec_for_network_policy(
        workspace.clone(),
        policy.world_fs_mode,
        &network,
    );
    assert!(!spec.isolate_network && !spec.always_isolate);
    spec.reuse_mode =
        world_api::WorldReuseMode::SharedOrchestration(world_api::SharedWorldOwnerSpec {
            orchestration_session_id: "aos_selected_fixture".into(),
            action: world_api::SharedWorldOwnerAction::AttachOrCreate,
        });
    let backend = world::LinuxLocalBackend::new();
    let initial = backend.ensure_session(&spec).unwrap();
    let initial_binding = initial.shared_binding.as_ref().unwrap();
    assert_eq!(initial_binding.world_generation, 0);
    let actual = if initial_zero {
        // The decisive initial path performs no replacement or relabeling.
        initial.clone()
    } else {
        // Preserve the old component's genuine generation1 method and attribution.
        let initial_cgroup = backend.cgroup_path(&initial).unwrap();
        let mut replacement = spec.clone();
        replacement.reuse_mode =
            world_api::WorldReuseMode::SharedOrchestration(world_api::SharedWorldOwnerSpec {
                orchestration_session_id: "aos_selected_fixture".into(),
                action: world_api::SharedWorldOwnerAction::ReplaceExpectedGeneration {
                    expected_generation: initial_binding.world_generation,
                    reason: "component fixture requires a supported nonzero Spawn generation"
                        .into(),
                },
            });
        let actual = backend.ensure_session(&replacement).unwrap();
        assert_eq!(actual.shared_binding.as_ref().unwrap().world_generation, 1);
        assert_ne!(actual.id, initial.id);
        assert!(initial_cgroup.starts_with(
            PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").unwrap()).join("substrate/cgroup")
        ));
        let procs = initial_cgroup.join("cgroup.procs");
        let metadata = std::fs::symlink_metadata(&procs).unwrap();
        assert!(metadata.is_file());
        assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
        assert_eq!(metadata.nlink(), 1);
        assert_eq!(std::fs::read(&procs).unwrap(), b"");
        std::fs::remove_file(procs).unwrap();
        std::fs::remove_dir(&initial_cgroup).unwrap();
        actual
    };
    let actual_binding = actual.shared_binding.as_ref().unwrap();
    assert_eq!(
        actual_binding.world_generation,
        if initial_zero { 0 } else { 1 }
    );
    if initial_zero {
        assert_eq!(actual.id, initial.id);
        assert_eq!(actual_binding, initial_binding);
    }
    assert_eq!(
        actual_binding.binding_state,
        world_api::SharedWorldBindingState::Active
    );
    let recovered = world::LinuxLocalBackend::new()
        .find_compatible_session(&spec)
        .unwrap()
        .unwrap();
    assert_eq!(recovered.shared_binding, actual.shared_binding);
    assert_eq!(recovered.id, actual.id);
    std::fs::write(
        temp.path().join("component-world-generation.json"),
        serde_json::to_vec(&serde_json::json!({"component_only":true,
            "initial_binding":initial_binding,"selected_actual_binding":actual_binding,
            "fresh_backend_recovered_exact_binding":true,
            "empty_superseded_fallback_removed":!initial_zero,
            "initial_generation_spawn_supported":initial_zero,
            "replace_before_start_count":if initial_zero {0} else {1}}))
        .unwrap(),
    )
    .unwrap();
    let component_cgroup = backend.cgroup_path(&actual).unwrap();
    assert!(
        !component_cgroup.starts_with("/sys/fs/cgroup"),
        "component run must use the documented unprivileged fallback"
    );
    assert!(component_cgroup.join("cgroup.procs").is_file());
    let actual = actual.shared_binding.unwrap();
    let world = WorldBindingV1 {
        world_id: actual.world_id,
        world_generation: actual.world_generation,
    };
    let policy_snapshot =
        crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
            &workspace, &bootstrap,
        )
        .unwrap();
    let principal = crate::execution::install_bootstrap::current_unix_principal_and_home()
        .unwrap()
        .0;
    let context = stage_native_start_context_from_ordinary_configured(
        Arc::clone(&configured),
        NativeStartContextInputsV1 {
            authority: &authority,
            intended_principal: &principal,
            effective_policy: &policy,
            workspace_binding: &binding,
            world_binding: Some(&world),
            intent_id: "hti_selected_fixture",
            orchestration_session_id: "aos_selected_fixture",
            shell_trace_session_id: "trace_selected_fixture",
            participant_id: "ash_selected_fixture",
            run_id: "run_selected_fixture",
            descriptor: &descriptor,
            opaque_launch_plan_digest: &"a".repeat(64),
            selections: &selections,
            sources: &sources,
        },
    )
    .unwrap();
    if apply {
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: context.intent_id.clone(),
            issuer_request_id: context.issuer_request_id().unwrap(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: context.orchestration_session_id.clone(),
            shell_trace_session_id: context.shell_trace_session_id.clone(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::Repl,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: context.original_participant_id.clone(),
            target_participant_lease_token: b"synthetic-selected-lease".to_vec(),
            run_id: context.original_run_id.clone(),
            resulting_authoritative_lineage: vec![context.original_participant_id.clone()],
            workspace_binding: binding,
            world_binding: Some(world),
            start_contract: StartContractMaterialV1 {
                descriptor,
                policy: PolicyObjectHashInputV1 {
                    schema_version: 1,
                    policy_revision: "component-current-policy-revision".into(),
                    canonical_policy_snapshot_sha256: policy_snapshot.snapshot_hash.clone(),
                },
                capabilities: HostAttachCapabilitiesV1 {
                    session_resume: true,
                    session_fork: true,
                    session_stop: true,
                    status_snapshot: true,
                    event_stream: true,
                },
                launch_knobs: HostAttachLaunchKnobsV1 {
                    requested_execution_scope: AgentExecutionScopeV1::Host,
                    host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                    attach_mode_preference: HostAttachModePreferenceV1::ContinuityRequired,
                },
            },
            transition_input: None,
        };
        let issued = authority.issue_start(&request).unwrap();
        let intent = match issued {
            TransitionIssueOutcomeV1::Issued(v) | TransitionIssueOutcomeV1::Joined(v) => v,
        };
        let claimed = authority
            .claim_start(&ClaimHostSessionTransitionRequestV1 {
                intent_id: intent.intent_id,
                issuer_request_id: intent.issuer_request_id,
                payload_commitment: intent.payload_commitment,
                expected_intent_revision: intent.intent_revision,
                claim_id: "clm_selected_fixture".into(),
                claimant_attempt_id: "atm_selected_fixture".into(),
            })
            .unwrap();
        let intent = match claimed {
            TransitionClaimOutcomeV1::Claimed(v)
            | TransitionClaimOutcomeV1::Reclaimed(v)
            | TransitionClaimOutcomeV1::Joined(v) => v,
        };
        let HostSessionTransitionIntentStateV2::Claimed {
            claim_id,
            claim_revision,
            ..
        } = intent.state
        else {
            panic!("genuine claim required");
        };
        authority
            .apply_start(&ApplyHostSessionTransitionRequestV1 {
                intent_id: intent.intent_id,
                issuer_request_id: intent.issuer_request_id,
                payload_commitment: intent.payload_commitment,
                expected_intent_revision: claim_revision,
                claim_id,
                expected_claim_revision: claim_revision,
            })
            .unwrap();
    }
    Fixture {
        _anthropic_key: anthropic_key,
        _env: env,
        _temp: temp,
        authority,
        context,
        home,
        configured,
        _world_root,
        component_cgroup,
    }
}

async fn exercise_ordinary_initial_fork_for_test(
    client: &transport_api_client::AgentClient,
    fork_execute: transport_api_types::ExecuteRequest,
    root: &Path,
    cwd: &Path,
    service_pid: u32,
    source_participant: &str,
    native: &[serde_json::Value],
) -> serde_json::Value {
    use futures::FutureExt as _;
    use transport_api_types::ExecuteStreamFrame;
    let child = fork_execute
        .member_dispatch
        .as_ref()
        .unwrap()
        .as_v1()
        .unwrap();
    let child_participant = child.participant_id.clone();
    let child_run = child.run_id.clone();
    let before_native = observations(&cwd.join("fork-observations.jsonl"));
    let before_gateway = observations(&root.join("gateway-observations.jsonl"));
    // These valid requests reach the independent configured service. They
    // cannot acquire another fixture worker mapping or consume source data.
    for case in ["source-cap", "root", "source-participant"] {
        let mut wrong = fork_execute.clone();
        if case == "root" {
            wrong
                .ordinary_start_selection
                .as_mut()
                .unwrap()
                .authority_store_id = "other-store".into();
        } else {
            let member = wrong.member_dispatch.as_mut().unwrap().as_v1_mut().unwrap();
            let activation = member.e2_launch_activation.as_mut().unwrap();
            if case == "source-cap" {
                activation.commitment_ref.exact_linkage_hash = "c".repeat(64);
                activation.immutable_worker_cap_ref = activation.commitment_ref.clone();
                activation.activation_id = format!("e2a_{}", "c".repeat(32));
            } else {
                member.parent_participant_id = Some("other-source".into());
                activation.source_participant_id = Some("other-source".into());
            }
        }
        wrong.validate().unwrap();
        assert!(
            client.execute_stream(wrong).await.is_err(),
            "service Fork tamper {case}"
        );
        assert_eq!(
            observations(&cwd.join("fork-observations.jsonl")),
            before_native
        );
        assert_eq!(
            observations(&root.join("gateway-observations.jsonl")),
            before_gateway
        );
    }
    let source_history = PathBuf::from(native[0]["home"].as_str().unwrap()).join("fixture-history");
    let original_bytes = std::fs::read(&source_history).unwrap();
    let original_identity = std::fs::metadata(&source_history).unwrap();
    let response = tokio::time::timeout(
        COMPONENT_STARTUP_BUDGET,
        client.execute_stream(fork_execute.clone()),
    )
    .await
    .unwrap()
    .unwrap();
    let child_bytes = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut terminal = tokio::spawn(frames(response, Arc::clone(&child_bytes)));
    let readiness = std::panic::AssertUnwindSafe(wait_until(
        || {
            assert!(
                !terminal.is_finished(),
                "Fork ended before native child readiness"
            );
            cwd.join("ready-fork").is_file()
        },
        "actual SDK thread_fork child readiness",
        COMPONENT_STARTUP_BUDGET,
    ))
    .catch_unwind()
    .await;
    if let Err(panic) = readiness {
        retain_initial_fork_readiness_failure(&mut terminal, &child_bytes, cwd, service_pid);
        std::panic::resume_unwind(panic);
    }
    let observed = observations(&cwd.join("fork-observations.jsonl"));
    assert_eq!(observed.len(), before_native.len() + 1);
    let fork = observed.last().unwrap().clone();
    assert_eq!(fork["args"], serde_json::json!(["app-server"]));
    assert_eq!(fork["parent_pid"], service_pid);
    assert_eq!(fork["home"], native[0]["home"]);
    assert_eq!(fork["home_device"], native[0]["home_device"]);
    assert_eq!(fork["home_inode"], native[0]["home_inode"]);
    for field in ["fixture_device", "fixture_inode", "fixture_sha256"] {
        assert_eq!(
            fork[field], native[0][field],
            "same checked physical executable: {field}"
        );
    }
    assert_eq!(fork["child_history_device"], fork["source_history_device"]);
    assert_ne!(
        fork["child_history_inode"], fork["source_history_inode"],
        "native fixture creates a separately owned child record, never relabels source history"
    );
    assert_eq!(fork["source_history_device"], original_identity.dev());
    assert_eq!(fork["source_history_inode"], original_identity.ino());
    assert_eq!(
        fork["source_sha256"],
        format!("{:x}", Sha256::digest(&original_bytes))
    );
    assert_ne!(fork["child_session"], fork["source_session"]);
    assert_eq!(fork["child_record"]["child_session"], fork["child_session"]);
    let child_history = PathBuf::from(fork["child_history"].as_str().unwrap());
    assert!(child_history.starts_with(source_history.parent().unwrap()));
    let gateway = observations(&root.join("gateway-observations.jsonl"))
        .last()
        .unwrap()
        .clone();
    observe_terminal_gate(
        fork["pid"].as_u64().unwrap(),
        gateway["pid"].as_u64().unwrap(),
        || terminal.is_finished(),
    )
    .await;
    std::fs::write(cwd.join("release-fork"), b"release").unwrap();
    let frames = tokio::time::timeout(Duration::from_secs(20), &mut terminal)
        .await
        .unwrap()
        .unwrap();
    let registered = frames
        .iter()
        .find_map(|frame| match frame {
            ExecuteStreamFrame::Event { event, .. }
                if event.kind == substrate_common::agent_events::AgentEventKind::Registered =>
            {
                Some(event)
            }
            _ => None,
        })
        .expect("actual surfaced UAA child identity, never fake Spawn registration");
    assert_eq!(
        registered.participant_id.as_deref(),
        Some(child_participant.as_str())
    );
    assert_eq!(
        registered.parent_participant_id.as_deref(),
        Some(source_participant)
    );
    assert_eq!(
        registered.thread_id.as_deref(),
        fork["child_session"].as_str()
    );
    assert!(frames
        .iter()
        .any(|frame| matches!(frame, ExecuteStreamFrame::Exit { exit: 0, .. })));
    let checked = observations(&root.join("native-history-settlements.jsonl"));
    let checked = checked
        .iter()
        .find(|row| {
            row["runtime"]["participant_id"] == child_participant
                && row["runtime"]["run_id"] == child_run
        })
        .expect("independent actual SDK settlement observation");
    assert_eq!(checked["checked"], true);
    let terminal_row = observations(&root.join("gateway-terminals.jsonl"))
        .last()
        .unwrap()
        .clone();
    assert_eq!(terminal_row["runtime"]["participant_id"], child_participant);
    assert_eq!(terminal_row["runtime"]["run_id"], child_run);
    assert_eq!(terminal_row["pid"], gateway["pid"]);
    assert_eq!(std::fs::read(&source_history).unwrap(), original_bytes);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&child_history).unwrap())
            .unwrap(),
        fork["child_record"]
    );
    serde_json::json!({"component_only":true,"vendor_native_history_qualified":false,
        "genuine_source_r0":true,"actual_sdk_thread_fork":true,"fork":fork,
        "checked_sdk_terminal":checked,"owned_gateway_terminal":terminal_row})
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires exact candidate service/helper binaries; genuine Start and initial Fork timing component"]
async fn test_ordinary_initial_fork_startup_timing() {
    tokio::time::timeout(
        COMPONENT_OUTER_BUDGET,
        Box::pin(
            ordinary_configured_spawn_start_and_two_resumes_with_posture(
                false,
                false,
                false,
                OrdinaryComponentNarrowing::None,
                true,
            ),
        ),
    )
    .await
    .expect("ordinary initial Fork timing component deadline");
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the explicitly compiled candidate world-service test binary; local component proof only"]
async fn test_ordinary_configured_spawn_start_and_two_resumes() {
    // Timeout drops this owned future through its existing service Child.wait
    // cleanup. The external resource guard still samples throughout the run.
    tokio::time::timeout(
        COMPONENT_OUTER_BUDGET,
        Box::pin(ordinary_configured_spawn_start_and_two_resumes_body(
            false, false, false, false,
        )),
    )
    .await
    .expect("ordinary component outer method deadline");
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the explicitly compiled candidate world-service test binary; local initial-generation-zero component proof only"]
async fn test_ordinary_configured_initial_generation_zero_start_and_two_resumes() {
    tokio::time::timeout(
        COMPONENT_OUTER_BUDGET,
        Box::pin(ordinary_configured_spawn_start_and_two_resumes_body(
            true, false, false, false,
        )),
    )
    .await
    .expect("ordinary initial-zero component outer method deadline");
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires exact candidate service/helper binaries; synthetic ordinary Claude component only"]
async fn test_ordinary_configured_claude_start_and_two_resumes() {
    tokio::time::timeout(
        COMPONENT_OUTER_BUDGET,
        Box::pin(ordinary_configured_spawn_start_and_two_resumes_body(
            true, true, false, false,
        )),
    )
    .await
    .expect("ordinary Claude component outer method deadline");
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires exact candidate service/helper binaries; current effects/cancel component only"]
async fn test_ordinary_configured_current_effect_revocations_and_returned_control_cancel() {
    // One extra actual Resume process and cancellation have their own unchanged
    // startup/terminal method budgets. Normal Start+two Resume checks stay intact.
    tokio::time::timeout(
        COMPONENT_OUTER_BUDGET + COMPONENT_STARTUP_BUDGET + Duration::from_secs(60),
        Box::pin(ordinary_configured_spawn_start_and_two_resumes_body(
            true, false, true, false,
        )),
    )
    .await
    .expect("ordinary current-effect component outer method deadline");
}

fn component_tree_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                pending.push(entry.path());
            } else {
                assert!(
                    kind.is_file(),
                    "component registry must not contain aliases"
                );
                result.insert(
                    entry.path().strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    result
}

fn enroll_component_public_start_source(
    fixture: &Fixture,
) -> anyhow::Result<crate::execution::agent_runtime::control::StartupPromptTransportListener> {
    enroll_component_public_start_source_with_plan(fixture).map(|(listener, _, _)| listener)
}

fn enroll_component_public_start_source_with_plan(
    fixture: &Fixture,
) -> anyhow::Result<(crate::execution::agent_runtime::control::StartupPromptTransportListener,
    crate::execution::agent_runtime::control::HiddenOwnerHelperLaunchPlan,
crate::execution::agent_runtime::host_session_authority::world_bound_controller::AuthenticatedWorldBoundControllerRunV1)>{
    use crate::execution::agent_runtime::{
        control::*,
        host_session_authority::{store_schema::*, world_bound_controller::*},
        orchestration_session::*,
        state_store::{WorldControllerSessionEnrollmentV1, WorldWorkReceiptRegistry},
    };
    let context = &fixture.context;
    let original = fixture
        .authority
        .read_preserved_start_root_v2()?
        .transition_intent_map
        .get(&context.intent_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("component original Start absent"))?;
    let HostSessionTransitionIntentStateV2::Applied {
        application_result_ref,
        authority_revision_after,
        applied_at,
        ..
    } = &original.state
    else {
        anyhow::bail!("component requires genuine Applied Start")
    };
    std::env::set_var("SUBSTRATE_HOME", &fixture.home);
    let listener = register_durable_start_startup_prompt_listener(
        &context.orchestration_session_id,
        &context.original_participant_id,
    )?;
    let descriptor = ResolvedRuntimeDescriptor {
        agent_id: context.descriptor.agent_id.clone(),
        backend_id: context.descriptor.backend_id.clone(),
        backend_kind: ResolvedRuntimeBackendKind::Codex,
        protocol: context.descriptor.protocol.clone(),
        execution_scope: AgentExecutionScope::Host,
        binary_path: context.descriptor.binary_path.clone(),
    };
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        Path::new(&context.workspace_binding.workspace_root.physical_path),
        &fixture.authority.bootstrap_home(),
    )?;
    let world = context
        .original_world_binding
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("component original World absent"))?;
    let mut helper = HiddenOwnerHelperLaunchPlan {
        mode: OwnerHelperMode::Start,
        descriptor: descriptor.clone(),
        session: HiddenOwnerHelperSessionPlan {
            orchestration_session_id: context.orchestration_session_id.clone(),
            shell_trace_session_id: context.shell_trace_session_id.clone(),
            workspace_root: context
                .workspace_binding
                .workspace_root
                .physical_path
                .clone(),
            world_id: Some(world.world_id.clone()),
            world_generation: Some(world.world_generation),
        },
        participant: HiddenOwnerHelperParticipantPlan {
            participant_id: context.original_participant_id.clone(),
            lease_token: "synthetic-selected-lease".into(),
            run_id: context.original_run_id.clone(),
            resumed_from_participant_id: None,
            internal_uaa_session_id: None,
        },
        host_attach_contract: Some(HostAttachContract {
            backend_id: descriptor.backend_id.clone(),
            execution_scope: AgentExecutionScope::Host,
            protocol: descriptor.protocol.clone(),
            launch_descriptor: descriptor,
            capabilities: HostAttachCapabilities {
                session_resume: true,
                session_fork: true,
                session_stop: true,
                status_snapshot: true,
                event_stream: true,
            },
            attach_launch_knobs: HostAttachLaunchKnobs {
                requested_execution_scope: AgentExecutionScope::Host,
                host_execution_client_start: HostAttachExecutionClientStart::StartNow,
                attach_mode_preference: HostAttachModePreference::ContinuityRequired,
            },
            effective_policy: serde_json::to_value(&policy)?,
            continuity_uaa_session_id: None,
        }),
        startup_prompt: Some(HiddenOwnerHelperStartupPromptPlan {
            prompt_text: "ordinary component controller source".into(),
            stream_path: listener.path().to_path_buf(),
            request_key_sha256: None,
            prompt_sha256: None,
            public_backend_id: Some(context.selections.requested.backend_id.clone()),
            public_scope: Some(AgentExecutionScope::World),
        }),
        source_orchestration_session_id: None,
    };
    let (prompt_sha, key) =
        crate::execution::agents_cmd::ordinary_component_public_start_keys_for_test(
            &helper,
            &context.selections,
        )?;
    helper.startup_prompt.as_mut().unwrap().request_key_sha256 = Some(key.clone());
    helper.startup_prompt.as_mut().unwrap().prompt_sha256 = Some(prompt_sha.clone());
    let mut transaction_id = Sha256::new();
    transaction_id.update(b"substrate.a1.3-p1.start-transaction-id.v1");
    transaction_id.update([0]);
    transaction_id.update(key.as_bytes());
    transaction_id.update([0]);
    transaction_id.update(context.orchestration_session_id.as_bytes());
    fixture
        .authority
        .begin_start_transaction(&StartTransactionRecordV1 {
            schema_version: 1,
            transaction_id: format!("stx_{:x}", transaction_id.finalize()),
            request_key_sha256: key,
            prompt_sha256: prompt_sha,
            authority_store_id: context.workspace_binding.authority_store_id.clone(),
            orchestration_session_id: context.orchestration_session_id.clone(),
            shell_trace_session_id: context.shell_trace_session_id.clone(),
            authoritative_participant_id: context.original_participant_id.clone(),
            backend_id: context.descriptor.backend_id.clone(),
            protocol: context.descriptor.protocol.clone(),
            workspace_root: context
                .workspace_binding
                .workspace_root
                .physical_path
                .clone(),
            world_id: Some(world.world_id.clone()),
            world_generation: Some(world.world_generation),
            public_backend_id: context.selections.requested.backend_id.clone(),
            public_scope: "world".into(),
            start_intent_id: context.intent_id.clone(),
            start_issuer_request_id: context.issuer_request_id()?,
            start_payload_commitment: original.payload_commitment.clone(),
            start_application_result_ref: application_result_ref.clone(),
            start_run_id: context.original_run_id.clone(),
            start_authority_revision: *authority_revision_after,
            created_at: applied_at.clone(),
            updated_at: applied_at.clone(),
            state: StartTransactionStateV1::PromptNotSubmitted,
        })?;
    let selected = Arc::new(
        OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(Arc::clone(
            &fixture.configured,
        ))?
        .authenticate(&context.orchestration_session_id)?,
    );
    let controller = AuthenticatedWorldBoundControllerRunV1::authenticate_received_helper(
        selected, &helper, None,
    )?;
    let origin = &controller.origin().observation;
    let registry = WorldWorkReceiptRegistry::bind(
        &fixture.home,
        &origin.bootstrap_home,
        &origin.authority_store_id,
    )?;
    anyhow::ensure!(
        registry.enroll_ordinary_world_controller_session(&controller)?
            == WorldControllerSessionEnrollmentV1::Created,
        "fresh component Start source was not enrolled"
    );
    Ok((listener, helper, controller))
}

#[test]
fn test_ordinary_component_public_start_enrollment_uses_issued_source() {
    let fixture = fixture_for_generation(true, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let root: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(root["schema_version"], 3);
    assert_eq!(root["start_transaction_map"].as_object().unwrap().len(), 1);
    let selected = OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(
        Arc::clone(&fixture.configured),
    )
    .unwrap()
    .authenticate(&fixture.context.orchestration_session_id)
    .unwrap();
    crate::execution::agent_runtime::host_session_authority::world_bound_controller::
        authenticate_ordinary_world_publication(&selected).unwrap();
}

#[test]
fn test_ordinary_readonly_snapshot_authenticates_interleaved_start_history() {
    use crate::execution::agent_runtime::host_session_authority::{
        facade::RetainedWorkerAuthorityPreconditionV1, store,
    };
    use crate::execution::agent_runtime::retained_worker_runtime::{
        RetainedWorkerRegistrationPlanV1, RetainedWorkerRuntime,
    };
    let fixture = fixture_for_generation(true, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let current = fixture
        .authority
        .resolve_current_exact(&fixture.context.orchestration_session_id, None)
        .unwrap();
    let mut worker = fixture.context.descriptor.clone();
    worker.agent_id = fixture
        .context
        .selections
        .requested
        .realized_agent_id
        .clone();
    worker.backend_id = fixture.context.selections.requested.backend_id.clone();
    worker.execution_scope = AgentExecutionScopeV1::World;
    RetainedWorkerRuntime
        .register_retained_target(
            &fixture.authority,
            &RetainedWorkerRegistrationPlanV1 {
                registration_request_id: "readonly-interleaved-registration".into(),
                orchestration_session_id: fixture.context.orchestration_session_id.clone(),
                expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                    authority_store_id: current.observation.authority_store_id,
                    authority_revision: current.observation.authority_revision,
                    authority_record_commitment: current.observation.authority_record_commitment,
                },
                retained_participant_id: "readonly-interleaved-worker".into(),
                descriptor: worker,
                internal_uaa_session_id: "readonly-interleaved-native".into(),
            },
        )
        .unwrap();
    let root_path = fixture.home.join("authority-v1/state-root-v1.json");
    let bytes = std::fs::read(&root_path).unwrap();
    let root: crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV3 =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        root.validate().unwrap_err().to_string(),
        "Start transaction state conflicts with current authority"
    );
    let before = component_tree_snapshot(&fixture.home);
    let read = store::read_existing_accepted_work_authority_snapshot(&fixture.authority);
    assert!(
        before == component_tree_snapshot(&fixture.home),
        "read changed persistent state"
    );
    assert_eq!(read.unwrap().hsa_state_root_bytes(), bytes);
    let mut orphaned = root.clone();
    let mut orphan = root.start_transaction_map.values().next().unwrap().clone();
    orphan.transaction_id = "zz_orphaned_start_transaction".into();
    orphan.orchestration_session_id = "absent_start_session".into();
    orphaned
        .start_transaction_map
        .insert(orphan.transaction_id.clone(), orphan);
    let orphaned_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&orphaned)
            .unwrap();
    std::fs::write(&root_path, &orphaned_bytes).unwrap();
    let malformed = component_tree_snapshot(&fixture.home);
    let invalid = store::read_existing_accepted_work_authority_snapshot(&fixture.authority);
    assert!(malformed == component_tree_snapshot(&fixture.home));
    assert!(
        invalid.is_err(),
        "unrelated malformed Start transaction was accepted"
    );
    std::fs::write(&root_path, &bytes).unwrap();
    // The accepted structural exception must still authenticate its historical
    // objects, including physical replacement at the final read boundary.
    let original = &root.transition_intent_map[&fixture.context.intent_id];
    let object_path = fixture
        .home
        .join("authority-v1/objects/host-attach-contract/v1")
        .join(format!("{}.obj", original.host_attach_contract_ref.ref_id));
    let object_bytes = std::fs::read(&object_path).unwrap();
    std::fs::write(&object_path, b"{}").unwrap();
    let damaged = component_tree_snapshot(&fixture.home);
    assert!(store::read_existing_accepted_work_authority_snapshot(&fixture.authority).is_err());
    assert!(damaged == component_tree_snapshot(&fixture.home));
    std::fs::write(&object_path, &object_bytes).unwrap();
    let attacked = std::rc::Rc::new(std::cell::RefCell::new(None));
    let capture = attacked.clone();
    let home = fixture.home.clone();
    store::set_e2_rm_before_final_verify_hook(move || {
        std::fs::rename(&object_path, object_path.with_extension("held")).unwrap();
        std::fs::write(&object_path, object_bytes).unwrap();
        std::fs::set_permissions(&object_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        *capture.borrow_mut() = Some(component_tree_snapshot(&home));
    });
    let error =
        store::read_existing_accepted_work_authority_snapshot(&fixture.authority).unwrap_err();
    assert!(matches!(error,
        crate::execution::agent_runtime::dispatch_policy_commitment::ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged { .. }
        | crate::execution::agent_runtime::dispatch_policy_commitment::ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged { .. }
    ), "{error:?}");
    assert!(attacked.borrow().as_ref().unwrap() == &component_tree_snapshot(&fixture.home));
}
#[test]
fn test_ordinary_world_publication_capture_preserves_original_v2_root() {
    use crate::execution::agent_runtime::host_session_authority::world_bound_controller;
    let fixture = fixture_for_generation(true, true);
    let authority = OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(
        Arc::clone(&fixture.configured),
    )
    .unwrap();
    let selected = authority
        .authenticate(&fixture.context.orchestration_session_id)
        .unwrap();
    let root_path = fixture.home.join("authority-v1/state-root-v1.json");
    let root_before = std::fs::read(&root_path).unwrap();
    let root_value: serde_json::Value = serde_json::from_slice(&root_before).unwrap();
    assert_eq!(root_value["schema_version"], 2);
    let before = component_tree_snapshot(&fixture.home);
    let publication = world_bound_controller::authenticate_ordinary_world_publication(&selected);
    let root_after = std::fs::read(&root_path).unwrap();
    let after = component_tree_snapshot(&fixture.home);
    assert!(
        before == after,
        "publication source capture wrote owner state"
    );
    assert!(
        root_before == root_after,
        "publication capture changed root schema/revision/bytes"
    );
    publication.expect("genuine original V2 Start must retain publication source custody");
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum OrdinaryComponentNarrowing {
    None,
    HostHidden,
    RoutingFailClosed,
}

async fn ordinary_configured_spawn_start_and_two_resumes_body(
    initial_zero: bool,
    claude: bool,
    current_effects: bool,
    narrowing: bool,
) {
    // Keep the large integration future boxed through this compatibility entry.
    Box::pin(
        ordinary_configured_spawn_start_and_two_resumes_with_posture(
            initial_zero,
            claude,
            current_effects,
            if narrowing {
                OrdinaryComponentNarrowing::HostHidden
            } else {
                OrdinaryComponentNarrowing::None
            },
            false,
        ),
    )
    .await;
}

async fn ordinary_configured_spawn_start_and_two_resumes_with_posture(
    initial_zero: bool,
    claude: bool,
    current_effects: bool,
    posture: OrdinaryComponentNarrowing,
    fork_startup_only: bool,
) {
    let narrowing = posture != OrdinaryComponentNarrowing::None;
    use crate::execution::agent_runtime::{
        dispatch_contract::*, retained_worker_runtime::RetainedWorkerRuntime,
        state_store::AgentRuntimeStateStore,
        world_work_execution_supervisor::WorldWorkExecutionSupervisor,
    };
    use crate::execution::build_ordinary_agent_client_and_member_dispatch_request_for_cwd;
    use crate::execution::orchestrator_world_dispatch::{
        continue_ordinary_world_worker, prepare_ordinary_authority_bound_spawn_world_worker,
    };
    use futures::FutureExt as _;
    use transport_api_types::{ExecuteStreamFrame, MemberRuntimeBackendKindV1};

    let service_binary = PathBuf::from(
        std::env::var_os("SUBSTRATE_TEST_ORDINARY_SERVICE_TEST_BINARY")
            .expect("supply the exact emitted candidate world-service lib test binary"),
    );
    assert!(service_binary.is_absolute() && service_binary.is_file());
    let helper_binary = PathBuf::from(
        std::env::var_os("SUBSTRATE_TEST_ORDINARY_LANDLOCK_HELPER_BINARY")
            .expect("supply the exact emitted candidate world-service non-test binary"),
    );
    let helper_metadata = std::fs::symlink_metadata(&helper_binary).unwrap();
    assert!(helper_binary.is_absolute() && helper_metadata.file_type().is_file());
    assert_eq!(helper_metadata.uid(), unsafe { libc::geteuid() });
    assert_ne!(helper_metadata.mode() & 0o111, 0);
    assert_eq!(helper_metadata.mode() & 0o022, 0);
    for harness in [&service_binary, &std::env::current_exe().unwrap()] {
        let metadata = std::fs::metadata(harness).unwrap();
        assert_ne!(
            (helper_metadata.dev(), helper_metadata.ino()),
            (metadata.dev(), metadata.ino()),
            "the policy helper must be the actual candidate binary, not a test harness"
        );
    }
    let mut executable_pair = BTreeMap::new();
    for (name, path) in [
        ("shell_test_binary", std::env::current_exe().unwrap()),
        ("service_test_binary", service_binary.clone()),
        ("landlock_helper_binary", helper_binary.clone()),
    ] {
        let mut file = File::open(&path).unwrap();
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 65536];
        loop {
            let count = file.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            digest.update(&buffer[..count]);
        }
        executable_pair.insert(
            name,
            serde_json::json!({"path":path,"sha256":format!("{:x}",digest.finalize())}),
        );
    }
    let component_policy = ordinary_component_policy(claude, narrowing);
    let native_fixture = if claude {
        CLAUDE_NATIVE_FIXTURE
    } else {
        NATIVE_FIXTURE
    };
    let native_name = if claude { "claude" } else { "codex" };
    let fixture = if narrowing {
        fixture_for_harness_with_narrowing(true, true, false, true)
    } else if claude {
        fixture_for_harness(true, initial_zero, true)
    } else if initial_zero {
        fixture_for_generation(true, true)
    } else {
        fixture(true)
    };
    // The actual Start owner enrolls B1 before the first independent World
    // input. Later turns must continue to refuse absent coverage.
    let _controller_source_listener = enroll_component_public_start_source(&fixture).unwrap();
    let root = fixture._temp.path();
    let socket = root.join("world.sock");
    std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket);
    let authority = OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(
        Arc::clone(&fixture.configured),
    )
    .unwrap();
    let selected = Arc::new(
        authority
            .authenticate(&fixture.context.orchestration_session_id)
            .unwrap(),
    );
    selected.require_ordinary_configured_authority().unwrap();
    let carrier: transport_api_types::InstallBootstrapContextCarrierV1 =
        serde_json::from_slice(&std::fs::read(root.join("carrier.json")).unwrap()).unwrap();
    let world = fixture.context.original_world_binding.as_ref().unwrap();
    let spawn_request = WorldDispatchRequestV1 {
        request_id: Some("req_component_spawn".into()),
        idempotency_key: Some("idem_component_spawn".into()),
        orchestration_session_id: Some(fixture.context.orchestration_session_id.clone()),
        caller_participant_id: Some(fixture.context.original_participant_id.clone()),
        action: WorldDispatchActionV1::SpawnWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(selected.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(world.world_id.clone()),
        world_generation: Some(world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
            prompt: "ordinary component turn 1".into(),
        }),
    }
    .validate()
    .unwrap();
    if initial_zero {
        assert_eq!(world.world_generation, 0);
        for (id, generation) in [
            ("unknown-world".to_string(), 0),
            (world.world_id.clone(), 1),
        ] {
            let mut wrong = spawn_request.clone();
            wrong.world_id = id;
            wrong.world_generation = generation;
            assert!(prepare_ordinary_authority_bound_spawn_world_worker(
                Arc::clone(&selected),
                wrong
            )
            .is_err());
        }
    }
    let prepared = prepare_ordinary_authority_bound_spawn_world_worker(
        Arc::clone(&selected),
        spawn_request.clone(),
    )
    .unwrap();
    let selected = Arc::clone(&prepared.selection);
    assert!(
        prepare_ordinary_authority_bound_spawn_world_worker(Arc::clone(&selected), spawn_request)
            .is_err(),
        "a claimed Spawn cannot silently deliver a second bootstrap"
    );
    let transport = crate::execution::routing::MemberDispatchTransportRequest {
        orchestration_session_id: prepared.request.orchestration_session_id.clone(),
        participant_id: prepared
            .launch_authority_proof
            .retained_participant_id
            .clone(),
        orchestrator_participant_id: prepared.request.caller_participant_id.clone(),
        parent_participant_id: None,
        resumed_from_participant_id: None,
        backend_id: prepared.descriptor.backend_id.clone(),
        protocol: prepared.descriptor.protocol.clone(),
        run_id: prepared.launch_authority_proof.bootstrap_run_id.clone(),
        world_id: prepared.request.world_id.clone(),
        world_generation: prepared.request.world_generation,
        initial_prompt: Some("ordinary component turn 1".into()),
        backend_kind: if claude {
            MemberRuntimeBackendKindV1::ClaudeCode
        } else {
            MemberRuntimeBackendKindV1::Codex
        },
        binary_path: prepared.descriptor.binary_path.display().to_string(),
        retained_worker_launch_authority: Some(prepared.launch_authority_proof.clone()),
        e2_launch_activation: Some(prepared.e2_launch_activation.clone()),
        config_projection: None,
        exact_policy_snapshot: Some(prepared.exact_policy_snapshot.clone()),
    };
    let cwd = Path::new(selected.workspace_root());
    assert!(
        build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
            &selected, &transport, root
        )
        .is_err()
    );
    let (client, execute, _) =
        build_ordinary_agent_client_and_member_dispatch_request_for_cwd(&selected, &transport, cwd)
            .unwrap();
    // Capture genuine original issuance before the service consumes it. After
    // actual native Registered/R0 this same guard is the Fork source witness.
    let initial_fork_source_admission =
        if !initial_zero && !claude && !current_effects && !narrowing {
            Some(Arc::new(selected.prepare_world_execute(&execute).unwrap()))
        } else {
            None
        };
    let expected_binary = if claude {
        cwd.join("claude").display().to_string()
    } else {
        "/var/lib/substrate/world-deps/bin/codex".to_string()
    };
    assert_eq!(transport.binary_path, expected_binary);
    assert_eq!(
        execute
            .member_dispatch
            .as_ref()
            .unwrap()
            .as_v1()
            .unwrap()
            .resolved_runtime
            .binary_path,
        expected_binary
    );
    let bytes = serde_json::to_vec(&execute).unwrap();
    let decoded: transport_api_types::ExecuteRequest = serde_json::from_slice(&bytes).unwrap();
    // The strict transport codec preserves all fields and validates the typed
    // request. JSON object order is not wire identity: env is a HashMap.
    let wire_value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(serde_json::to_value(&decoded).unwrap(), wire_value);
    let mut unknown_field = wire_value.clone();
    unknown_field
        .as_object_mut()
        .unwrap()
        .insert("unexpected_component_field".into(), true.into());
    assert!(serde_json::from_value::<transport_api_types::ExecuteRequest>(unknown_field).is_err());
    let mut wrong_schema = wire_value.clone();
    wrong_schema["schema_version"] = 1.into();
    assert!(serde_json::from_value::<transport_api_types::ExecuteRequest>(wrong_schema).is_err());
    assert!(!String::from_utf8_lossy(&bytes).contains("synthetic-ordinary-token"));
    let gateway = root.join("gateway-fixture.py");
    std::fs::write(
        &gateway,
        if claude {
            CLAUDE_GATEWAY_FIXTURE
        } else {
            GATEWAY_FIXTURE
        },
    )
    .unwrap();
    std::fs::set_permissions(&gateway, std::fs::Permissions::from_mode(0o700)).unwrap();
    // METHOD correction only: an owned, static literal /tmp namespace for this
    // independent service child. Parent HOME/TMPDIR and old failed proof stay intact.
    let service_tmp = tempfile::Builder::new()
        .prefix("substrate-ordinary-component-")
        .tempdir_in("/tmp")
        .unwrap();
    std::fs::set_permissions(service_tmp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let service_tmp_identity = std::fs::metadata(service_tmp.path()).unwrap();
    if let Some(evidence) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") {
        let evidence = PathBuf::from(evidence);
        std::fs::create_dir_all(&evidence).unwrap();
        std::fs::write(
            evidence.join("owned-service-tmp.json"),
            serde_json::to_vec_pretty(
                &serde_json::json!({"component_only":true,"path":service_tmp.path(),
                "dev":service_tmp_identity.dev(),"ino":service_tmp_identity.ino(),
                "uid":service_tmp_identity.uid(),"mode":service_tmp_identity.mode(),
                "method":"child-only literal /tmp scratch; original custom-TMP proof retained"}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    let mut service = OwnedService(
        Command::new(service_binary)
            .args(["--exact", SERVICE_WORKER, "--ignored", "--nocapture"])
            .env("SUBSTRATE_TEST_ORDINARY_ROOT", root)
            .env(
                "SUBSTRATE_TEST_ORDINARY_EXTRA_TURN",
                if current_effects { "1" } else { "0" },
            )
            .env("TMPDIR", service_tmp.path())
            .env("SUBSTRATE_GATEWAY_BINARY", &gateway)
            // Existing production turn policy invokes only the helper's early
            // __substrate_world_landlock_exec dispatch, never its daemon path.
            .env("SUBSTRATE_LANDLOCK_HELPER_SRC", &helper_binary)
            .env("SUBSTRATE_GATEWAY_RUNTIME_ROOT", root.join("gateways"))
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    eprintln!("BEGIN independently invoked ordinary service worker: pid={} binary={:?} argv=[--exact,{SERVICE_WORKER},--ignored,--nocapture]", service.0.id(), executable_pair["service_test_binary"]["path"]);
    wait_until(
        || {
            assert!(
                service.0.try_wait().unwrap().is_none(),
                "candidate service worker exited before ready"
            );
            root.join("service-ready").is_file()
        },
        "independent service ready",
        Duration::from_secs(15),
    )
    .await;
    let mut wrong = execute.clone();
    wrong
        .ordinary_start_selection
        .as_mut()
        .unwrap()
        .context_sha256 = "f".repeat(64);
    assert!(
        client.execute_stream(wrong).await.is_err(),
        "independent service must reject substituted origin"
    );
    let mut wrong = execute.clone();
    wrong.cwd = Some(root.display().to_string());
    assert!(
        client.execute_stream(wrong).await.is_err(),
        "independent service must reject substituted workspace"
    );
    assert!(observations(&cwd.join("native-observations.jsonl")).is_empty());
    let initial_startup_deadline = Instant::now() + COMPONENT_STARTUP_BUDGET;
    let response = tokio::time::timeout_at(
        initial_startup_deadline.into(),
        client.execute_stream(execute.clone()),
    )
    .await
    .unwrap()
    .unwrap();
    let start_stream_bytes = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut initial_terminal = tokio::spawn(frames(response, Arc::clone(&start_stream_bytes)));
    let start_result = std::panic::AssertUnwindSafe(async {
        wait_until(
            || {
                let ready = cwd.join("ready-turn-1").is_file();
                assert!(
                    ready || !initial_terminal.is_finished(),
                    "actual Start stream ended before native readiness"
                );
                ready
            },
            "Start normal completion gate",
            initial_startup_deadline.saturating_duration_since(Instant::now()),
        )
        .await;
        let initial_pid: u64 = std::fs::read_to_string(cwd.join("ready-turn-1"))
            .unwrap()
            .parse()
            .unwrap();
        let initial_gateway_pid = observations(&root.join("gateway-observations.jsonl"))
            .last()
            .unwrap()["pid"]
            .as_u64()
            .unwrap();
        observe_terminal_gate(initial_pid, initial_gateway_pid, || {
            initial_terminal.is_finished()
        })
        .await;
        std::fs::write(cwd.join("release-turn-1"), b"release").unwrap();
        let start_frames = tokio::time::timeout(Duration::from_secs(20), &mut initial_terminal)
            .await
            .unwrap()
            .unwrap();
        assert!(
            start_frames
                .iter()
                .any(|frame| matches!(frame, ExecuteStreamFrame::Exit { exit: 0, .. })),
            "{start_frames:?}"
        );
        assert!(
            !start_frames
                .iter()
                .any(|frame| matches!(frame, ExecuteStreamFrame::Error { .. })),
            "{start_frames:?}"
        );
        start_frames
    })
    .catch_unwind()
    .await;
    let start_frames = match start_result {
        Ok(frames) => frames,
        Err(panic) => {
            if let Some(evidence) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") {
                let bytes = start_stream_bytes.lock().unwrap().clone();
                let stream = String::from_utf8_lossy(&bytes);
                let decoded: Vec<_> = stream
                    .lines()
                    .map(
                        |line| match serde_json::from_str::<serde_json::Value>(line) {
                            Ok(frame) => frame,
                            Err(error) => serde_json::json!({"partial_or_invalid_frame":line,
                            "decode_error":error.to_string()}),
                        },
                    )
                    .collect();
                let failure = serde_json::json!({"component_only":true,"turn":1,
                    "service_pid":service.0.id(),"stream_finished":initial_terminal.is_finished(),
                    "actual_stream_bytes":stream,"decoded_frames":decoded,
                    "native":observations(&cwd.join("native-observations.jsonl")),
                    "native_entries":observations(&cwd.join("native-entries.jsonl")),
                    "native_failures":observations(&cwd.join("native-failures.jsonl")),
                    "gateway":observations(&root.join("gateway-observations.jsonl")),
                    "received":observations(&root.join("received.jsonl")),
                    "response_errors":observations(&root.join("response-errors.jsonl"))});
                let evidence = PathBuf::from(evidence);
                if let Err(error) = std::fs::create_dir_all(&evidence).and_then(|()| {
                    std::fs::write(
                        evidence.join("start-gate-failure.json"),
                        serde_json::to_vec_pretty(&failure).unwrap(),
                    )
                }) {
                    eprintln!("could not retain actual Start gate failure: {error}");
                }
            }
            std::panic::resume_unwind(panic);
        }
    };
    let (frame_identity, event) = start_frames
        .iter()
        .find_map(|frame| match frame {
            ExecuteStreamFrame::Event {
                frame_identity,
                event,
            } if event.kind == substrate_common::agent_events::AgentEventKind::Registered => {
                Some((frame_identity, event))
            }
            _ => None,
        })
        .expect("actual independent-service registration frame");
    RetainedWorkerRuntime
        .mark_admission_routable(
            &prepared.authority,
            &prepared.admission_plan,
            &transport.participant_id,
            Some(&prepared.dispatch_policy_reservation_proof),
            frame_identity,
            event,
            TimestampV1::parse(event.ts.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
                .unwrap(),
        )
        .unwrap();
    let selected = Arc::new(
        authority
            .authenticate(&fixture.context.orchestration_session_id)
            .unwrap(),
    );
    assert!(
        client.execute_stream(execute.clone()).await.is_err(),
        "routable Spawn must not re-deliver its bootstrap"
    );
    fixture._env.install_home(&fixture.home);
    let store = AgentRuntimeStateStore::new().unwrap();
    let initial_fork_execute = initial_fork_source_admission
        .as_ref()
        .map(|source_admission| {
            assert_initial_fork_admission_joins(
                &fixture,
                &selected,
                &store,
                &transport,
                &execute,
                source_admission,
            )
        });
    if fork_startup_only {
        assert!(!initial_zero && !claude && !current_effects && !narrowing);
        let native = observations(&cwd.join("native-observations.jsonl"));
        assert_eq!(
            native.len(),
            1,
            "one genuine checked Start creates the source history"
        );
        let fork = exercise_ordinary_initial_fork_for_test(
            &client,
            initial_fork_execute.expect("genuine source admission"),
            root,
            cwd,
            service.0.id(),
            &transport.participant_id,
            &native,
        )
        .await;
        std::fs::write(root.join("stop-service"), b"stop").unwrap();
        wait_until(
            || service.0.try_wait().unwrap().is_some(),
            "service graceful shutdown",
            Duration::from_secs(15),
        )
        .await;
        let service_exit = service.0.wait().unwrap();
        assert!(service_exit.success());
        let after = std::fs::metadata(service_tmp.path()).unwrap();
        assert_eq!(
            (service_tmp_identity.dev(), service_tmp_identity.ino()),
            (after.dev(), after.ino())
        );
        let scratch_path = service_tmp.path().to_path_buf();
        service_tmp.close().unwrap();
        assert!(
            !scratch_path.exists(),
            "owned scratch close after checked service wait"
        );
        std::fs::remove_dir_all(&fixture.component_cgroup).unwrap();
        eprintln!("ordinary initial Fork timing component: service_pid={} fork_pid={} checked_native_and_gateway=true service_wait_success=true owned_temp_close=true",
            service.0.id(), fork["fork"]["pid"]);
        return;
    }
    let observation = prepared
        .authority
        .resolve_current_exact(&fixture.context.orchestration_session_id, None)
        .unwrap();
    let supervisor = WorldWorkExecutionSupervisor::bind(
        &fixture.home,
        &observation.observation.bootstrap_home,
        &observation.observation.authority_store_id,
    )
    .unwrap();
    let original_selection = selected.requested_selection().clone();
    // Captured selection/defaults/source identity survive later source changes.
    std::fs::write(
        fixture.home.join("config.yaml"),
        "world:\n  net:\n    filter: false\nagents:\n  enabled: false\n",
    )
    .unwrap();
    std::fs::write(
        fixture.home.join(if claude {
            "agents/claude_code.yaml"
        } else {
            "agents/codex.yaml"
        }),
        "malformed later inventory\n",
    )
    .unwrap();
    let later_config = crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
        cwd,
        &Default::default(),
        &prepared.authority.bootstrap_home(),
    )
    .unwrap();
    assert!(!later_config.agents.enabled);
    assert!(
        !later_config.world.net.filter,
        "later defaults must retain the permitted component network placement"
    );
    let mut routing_receipts = Vec::new();
    for index in 2..=3 {
        let mut request = WorldDispatchRequestV1 {
            request_id: Some(format!("req_component_turn_{index}")),
            idempotency_key: Some(format!("idem_component_turn_{index}")),
            orchestration_session_id: Some(fixture.context.orchestration_session_id.clone()),
            caller_participant_id: Some(fixture.context.original_participant_id.clone()),
            action: WorldDispatchActionV1::ContinueWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(transport.backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(transport.participant_id.clone()),
            world_id: Some(world.world_id.clone()),
            world_generation: Some(world.world_generation),
            dispatch_policy_narrowing: None,
            payload: WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                prompt: format!("ordinary component turn {index}"),
                thread_id: Some("thread-ordinary-world-component".into()),
            }),
        };

        if narrowing {
            assert_eq!(world.world_generation, 0);
            let current = selected
                .held_authority()
                .resolve_current_exact(selected.original_session_id(), None)
                .unwrap();
            request.dispatch_policy_narrowing =
                Some(transport_api_types::DispatchPolicyNarrowingPatchV1 {
                    schema_version: 1,
                    request_id: request.request_id.clone().unwrap(),
                    orchestration_session_id: request.orchestration_session_id.clone().unwrap(),
                    caller_participant_id: request.caller_participant_id.clone().unwrap(),
                    target_backend_id: request.target_backend_id.clone().unwrap(),
                    target_world: transport_api_types::WorldBindingRefV1 {
                        world_id: world.world_id.clone(),
                        world_generation: 0,
                    },
                    applies_to:
                        transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerTurn {
                            retained_participant_id: transport.participant_id.clone(),
                        },
                    parent_policy_ref: crate::execution::agent_runtime::dispatch_policy_commitment::transport_policy_ref(
                        current.authority.current_policy_ref.as_ref().unwrap(),
                    ),
                    parent_policy_revision: current.current_policy.policy_revision,
                    restricted_policy_patch: transport_api_types::RestrictedPolicyPatchV1 {
                        world_fs: Some(match posture {
                            OrdinaryComponentNarrowing::HostHidden => {
                                transport_api_types::RestrictedWorldFsPatchV1 {
                                    host_visible: Some(false),
                                    ..Default::default()
                                }
                            }
                            OrdinaryComponentNarrowing::RoutingFailClosed => {
                                assert_eq!(
                                    current.current_policy.canonical_policy_snapshot_sha256,
                                    prepared.exact_policy_snapshot.policy_snapshot_hash
                                );
                                let cap = &prepared.exact_policy_snapshot.policy_snapshot;
                                assert!(cap.world_fs.host_visible);
                                assert!(!cap.world_fs.fail_closed.routing);
                                transport_api_types::RestrictedWorldFsPatchV1 {
                                    fail_closed: Some(
                                        transport_api_types::RestrictedWorldFsFailClosedPatchV1 {
                                            routing: Some(true),
                                        },
                                    ),
                                    ..Default::default()
                                }
                            }
                            OrdinaryComponentNarrowing::None => unreachable!(),
                        }),
                    },
                    reason: Some("ordinary zero component narrowing".into()),
                });
            assert!(request.clone().validate().is_err());
            request.clone().validate_ordinary_representation().unwrap();
            let received_before_wrong_policy = observations(&root.join("received.jsonl"));
            let native_before_wrong_policy = observations(&cwd.join("native-entries.jsonl"));
            for mutation in 0..4 {
                let mut wrong_policy = request.clone();
                let parent = &mut wrong_policy
                    .dispatch_policy_narrowing
                    .as_mut()
                    .unwrap()
                    .parent_policy_ref;
                match mutation {
                    0 => parent.ref_id = format!("ao_{}", "0".repeat(32)),
                    1 => {
                        parent.object_kind =
                            transport_api_types::AuthorityObjectKindV1::RetainedWorker
                    }
                    2 => parent.schema_version += 1,
                    _ => {
                        parent.commitment =
                            transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                                digest_hex: "0".repeat(64),
                            }
                    }
                }
                assert!(
                    continue_ordinary_world_worker(
                        &store,
                        Arc::clone(&selected),
                        carrier.clone(),
                        wrong_policy
                    )
                    .await
                    .is_err(),
                    "wrong current policy field {mutation}"
                );
                assert_eq!(
                    observations(&root.join("received.jsonl")),
                    received_before_wrong_policy
                );
                assert_eq!(
                    observations(&cwd.join("native-entries.jsonl")),
                    native_before_wrong_policy
                );
            }
            let received_before_unsealed = observations(&root.join("received.jsonl"));
            let native_before_unsealed = observations(&cwd.join("native-entries.jsonl"));
            if let Ok(unsealed) = crate::execution::agent_runtime::native_launch_context::authenticate_ordinary_selection_from_authority(Arc::clone(&fixture.authority), &fixture.context.orchestration_session_id) {
                assert!(unsealed.require_ordinary_configured_authority().is_err());
                assert!(continue_ordinary_world_worker(&store, Arc::new(unsealed), carrier.clone(), request.clone()).await.is_err());
            }
            assert_eq!(
                observations(&root.join("received.jsonl")),
                received_before_unsealed
            );
            assert_eq!(
                observations(&cwd.join("native-entries.jsonl")),
                native_before_unsealed
            );
        }
        let resume_startup_deadline = Instant::now() + COMPONENT_STARTUP_BUDGET;
        let outcome = tokio::time::timeout_at(
            resume_startup_deadline.into(),
            continue_ordinary_world_worker(
                &store,
                Arc::clone(&selected),
                carrier.clone(),
                request.clone(),
            ),
        )
        .await
        .unwrap();
        if let Err(error) = &outcome {
            if let Some(evidence) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") {
                let evidence = PathBuf::from(evidence);
                std::fs::create_dir_all(&evidence).unwrap();
                let failure = serde_json::json!({"component_only":true,"turn":index,
                    "wrapper_error":error.to_string(),"request":request,"service_pid":service.0.id(),
                    "start_frames":start_frames,"native":observations(&cwd.join("native-observations.jsonl")),
                    "gateway":observations(&root.join("gateway-observations.jsonl")),
                    "response_errors":observations(&root.join("response-errors.jsonl"))});
                std::fs::write(
                    evidence.join(format!("continue-failure-{index}.json")),
                    serde_json::to_vec_pretty(&failure).unwrap(),
                )
                .unwrap();
            }
        }
        let outcome = outcome.unwrap();
        let WorldDispatchOutcomeV1::AcceptedForeground(receipt) = outcome else {
            panic!("genuine B1 retained acceptance required")
        };
        let AcceptedForegroundReceiptV1::Retained(receipt) = *receipt else {
            panic!("retained receipt required")
        };
        if posture == OrdinaryComponentNarrowing::RoutingFailClosed {
            routing_receipts.push((receipt.clone(), request.clone()));
        }
        // Preserve the actual post-receipt cause before fixture cleanup.
        // Readiness uses the explicit method budget; terminal/identity assertions remain.
        let gate_result = std::panic::AssertUnwindSafe(async {
            wait_until(
                || {
                    let ready = cwd.join(format!("ready-turn-{index}")).is_file();
                    if !ready {
                        if let Some(observed) = supervisor
                            .inspect_observation_by_acceptance_id(&receipt.acceptance_record_id)
                            .unwrap()
                        {
                            assert!(
                                observed.terminal.is_none() && observed.interruption.is_none(),
                                "actual Resume ended before native readiness: {:?} {:?}",
                                observed.terminal,
                                observed.interruption
                            );
                        }
                    }
                    ready
                },
                "Resume normal completion gate",
                resume_startup_deadline.saturating_duration_since(Instant::now()),
            )
            .await;
            let active_pid: u64 = std::fs::read_to_string(cwd.join(format!("ready-turn-{index}")))
                .unwrap()
                .parse()
                .unwrap();
            let gateway_pid = observations(&root.join("gateway-observations.jsonl"))
                .last()
                .unwrap()["pid"]
                .as_u64()
                .unwrap();
            observe_terminal_gate(active_pid, gateway_pid, || {
                supervisor
                    .inspect_observation_by_acceptance_id(&receipt.acceptance_record_id)
                    .unwrap()
                    .and_then(|value| value.terminal)
                    .is_some()
            })
            .await;
            std::fs::write(cwd.join(format!("release-turn-{index}")), b"release").unwrap();
            wait_until(
                || {
                    supervisor
                        .inspect_observation_by_acceptance_id(&receipt.acceptance_record_id)
                        .unwrap()
                        .and_then(|value| value.terminal)
                        .is_some_and(|terminal| terminal.exit_code == 0)
                },
                "actual Resume terminal journal",
                Duration::from_secs(15),
            )
            .await;
            assert_eq!(
                observations(&cwd.join("native-observations.jsonl")).len(),
                index
            );
        })
        .catch_unwind()
        .await;
        if let Err(panic) = gate_result {
            if let Some(evidence) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") {
                let evidence = PathBuf::from(evidence);
                let observation = match supervisor
                    .inspect_observation_by_acceptance_id(&receipt.acceptance_record_id)
                {
                    Ok(Some(observed)) => {
                        let decoded_frames: Vec<_> = observed
                            .journal
                            .iter()
                            .map(|entry| match entry.decode_for_c1(&observed.claim) {
                                Ok(frame) => serde_json::to_value(frame).unwrap(),
                                Err(error) => {
                                    serde_json::json!({"decode_error":format!("{error:#}")})
                                }
                            })
                            .collect();
                        serde_json::json!({"claim":observed.claim,
                            "journal":observed.journal,"decoded_frames":decoded_frames,
                            "terminal":observed.terminal,"interruption":observed.interruption})
                    }
                    Ok(None) => serde_json::json!({"missing_observation":true}),
                    Err(error) => serde_json::json!({"observation_error":format!("{error:#}")}),
                };
                let failure = serde_json::json!({"component_only":true,"turn":index,
                    "request":request,"acceptance_record_id":receipt.acceptance_record_id,
                    "service_pid":service.0.id(),"start_frames":start_frames,
                    "observation":observation,
                    "native":observations(&cwd.join("native-observations.jsonl")),
                    "native_entries":observations(&cwd.join("native-entries.jsonl")),
                    "native_failures":observations(&cwd.join("native-failures.jsonl")),
                    "gateway":observations(&root.join("gateway-observations.jsonl")),
                    "received":observations(&root.join("received.jsonl")),
                    "response_errors":observations(&root.join("response-errors.jsonl"))});
                if let Err(error) = std::fs::create_dir_all(&evidence).and_then(|()| {
                    std::fs::write(
                        evidence.join(format!("resume-gate-failure-{index}.json")),
                        serde_json::to_vec_pretty(&failure).unwrap(),
                    )
                }) {
                    eprintln!("could not retain Resume gate failure: {error}");
                }
            }
            std::panic::resume_unwind(panic);
        }
        let received_before = observations(&root.join("received.jsonl")).len();
        let retry =
            continue_ordinary_world_worker(&store, Arc::clone(&selected), carrier.clone(), request)
                .await
                .unwrap();
        let WorldDispatchOutcomeV1::AcceptedForeground(retry) = retry else {
            panic!("same accepted work required")
        };
        let AcceptedForegroundReceiptV1::Retained(retry) = *retry else {
            panic!("retained retry required")
        };
        assert_eq!(retry.acceptance_record_id, receipt.acceptance_record_id);
        assert_eq!(
            observations(&root.join("received.jsonl")).len(),
            received_before,
            "accepted retry must not redeliver HTTP input"
        );
        assert_eq!(
            observations(&cwd.join("native-observations.jsonl")).len(),
            index
        );
    }
    assert_eq!(selected.requested_selection(), &original_selection);
    let native = observations(&cwd.join("native-observations.jsonl"));
    assert_eq!(
        native.len(),
        3,
        "one Start and exactly two Resume processes"
    );
    let pids: std::collections::BTreeSet<_> = native
        .iter()
        .map(|value| value["pid"].as_u64().unwrap())
        .collect();
    assert_eq!(pids.len(), 3);
    let fixture_metadata = std::fs::metadata(cwd.join(native_name)).unwrap();
    let fixture_digest = format!("{:x}", sha2::Sha256::digest(native_fixture.as_bytes()));
    for value in &native {
        assert_eq!(value["fixture_device"], fixture_metadata.dev());
        assert_eq!(value["fixture_inode"], fixture_metadata.ino());
        assert_eq!(value["fixture_sha256"], fixture_digest);
        assert_eq!(
            value["fixture_script"],
            cwd.join(native_name).display().to_string()
        );
        assert_eq!(
            value["home"], native[0]["home"],
            "one retained history home"
        );
        for identity in [
            "home_device",
            "home_inode",
            "history_device",
            "history_inode",
        ] {
            assert_eq!(
                value[identity], native[0][identity],
                "retained physical history changed: {identity}"
            );
        }
        assert_eq!(value["cwd"], cwd.display().to_string());
        assert_eq!(
            value["cgroup_target"],
            fixture
                .component_cgroup
                .join("cgroup.procs")
                .canonicalize()
                .unwrap()
                .display()
                .to_string()
        );
        let index = value["index"].as_u64().unwrap();
        let expected_history = format!(
            "thread-ordinary-world-component{}",
            (1..=index)
                .map(|i| format!("\nturn-{i}"))
                .collect::<String>()
        );
        assert_eq!(value["history"], expected_history);
        assert_eq!(
            value["parent_pid"],
            service.0.id(),
            "actual SDK exec chain must retain the service parent"
        );
        assert!(value["home_alias"]
            .as_str()
            .unwrap()
            .starts_with("/proc/self/fd/"));
        if claude {
            assert_eq!(value["config"], "{}\n");
            assert_eq!(
                value["model"],
                selected.requested_selection().routed_model_id
            );
            assert!(value["endpoint"]
                .as_str()
                .unwrap()
                .starts_with("http://127.0.0.1:"));
            assert!(
                !value["args"].as_array().unwrap().iter().any(|arg| matches!(
                    arg.as_str(),
                    Some("--continue" | "--fork-session" | "--add-dir")
                ))
            );
        } else {
            assert!(Path::new(value["output"].as_str().unwrap()).starts_with(service_tmp.path()));
            assert!(value["config"]
                .as_str()
                .unwrap()
                .contains("model = \"synthetic-worker-model\""));
            assert!(value["config"]
                .as_str()
                .unwrap()
                .contains("http://127.0.0.1:"));
        }
    }
    let sdk_completions = observations(&root.join("sdk-completions.jsonl"));
    assert_eq!(sdk_completions.len(), 3);
    for (index, completion) in sdk_completions.iter().enumerate() {
        assert_eq!(completion["success"], true);
        assert_eq!(
            completion["final_text"],
            format!("ordinary SDK output turn {}", index + 1),
            "actual pinned SDK parent must read native output"
        );
        assert_eq!(completion["handoff"], "joined_used");
        assert_eq!(completion["latest_join"][1], "joined_used");
    }
    assert_eq!(
        std::fs::read_to_string(root.join("gateway-native-requests"))
            .unwrap()
            .lines()
            .count(),
        3
    );
    let gateways = observations(&root.join("gateway-observations.jsonl"));
    assert_eq!(
        gateways.len(),
        3,
        "separate owned gateway per native process"
    );
    let terminals = observations(&root.join("gateway-terminals.jsonl"));
    assert_eq!(
        terminals.len(),
        3,
        "explicit owned terminal result per native turn"
    );
    for (index, (gateway, terminal)) in gateways.iter().zip(&terminals).enumerate() {
        assert_eq!(terminal["pid"], gateway["pid"]);
        let expected_run = if index == 0 {
            transport.run_id.clone()
        } else {
            format!("req_component_turn_{}", index + 1)
        };
        assert_eq!(terminal["runtime"]["run_id"], expected_run);
        assert_eq!(
            terminal["runtime"]["participant_id"],
            transport.participant_id
        );
        assert_eq!(
            terminal["runtime"]["orchestration_session_id"],
            transport.orchestration_session_id
        );
        assert_eq!(terminal["runtime"]["backend_id"], transport.backend_id);
        assert_eq!(terminal["runtime"]["world_id"], transport.world_id);
        assert_eq!(
            terminal["runtime"]["world_generation"],
            transport.world_generation
        );
        assert_eq!(terminal["native_open"], false);
        assert_eq!(terminal["owned_child_terminal"], true);
        assert_eq!(terminal["manifest_and_entry_disposed"], true);
        assert!(terminal["code"].is_number() || terminal["signal"].is_number());
    }
    wait_until(
        || {
            gateways.iter().all(|value| {
                !PathBuf::from(format!("/proc/{}", value["pid"].as_u64().unwrap())).exists()
            })
        },
        "owned gateway cleanup after normal terminal",
        Duration::from_secs(15),
    )
    .await;
    let received = observations(&root.join("received.jsonl"));
    let turns: Vec<_> = received
        .iter()
        .filter(|value| value["path"] == "/v1/ordinary/member_turn/stream")
        .collect();
    assert_eq!(turns.len(), 2);
    for turn in turns {
        for (field, expected) in [
            (
                "orchestration_session_id",
                transport.orchestration_session_id.as_str(),
            ),
            ("participant_id", transport.participant_id.as_str()),
            (
                "orchestrator_participant_id",
                transport.orchestrator_participant_id.as_str(),
            ),
            ("backend_id", transport.backend_id.as_str()),
            ("world_id", world.world_id.as_str()),
        ] {
            assert_eq!(turn["body"][field], expected);
        }

        if narrowing {
            use base64::Engine as _;
            let carrier = &turn["body"]["policy_snapshot_carrier"];
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(carrier["policy_snapshot_bytes_base64"].as_str().unwrap())
                .unwrap();
            let snapshot: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            if posture == OrdinaryComponentNarrowing::HostHidden {
                assert_eq!(snapshot["world_fs"]["host_visible"], false);
            } else {
                use crate::execution::agent_runtime::dispatch_policy_commitment::*;
                use crate::execution::agent_runtime::state_store::{
                    WorldWorkReceiptRegistry, WorldWorkSubmissionIdentityV1,
                };
                assert_eq!(routing_receipts.len(), 2);
                assert_eq!(snapshot["world_fs"]["host_visible"], true);
                assert_eq!(snapshot["world_fs"]["fail_closed"]["routing"], true);
                // The current parent and immutable cap are unchanged; the one
                // requested restriction must be the entire received policy delta.
                let mut expected = prepared.exact_policy_snapshot.policy_snapshot.clone();
                expected.world_fs.fail_closed.routing = true;
                assert_eq!(bytes, serde_json::to_vec(&expected).unwrap());
                assert_ne!(bytes, prepared.exact_policy_snapshot.policy_snapshot_bytes);
                let (receipt, request) = routing_receipts
                    .iter()
                    .find(|(receipt, _)| {
                        carrier["subject"]["active_run_id"] == receipt.active_run_id
                    })
                    .unwrap();
                assert_eq!(carrier["subject"]["message_id"], receipt.message_id);
                assert_eq!(
                    carrier["policy_snapshot_hash"],
                    receipt.turn_policy_snapshot_hash
                );
                assert_eq!(
                    carrier["policy_snapshot_ref"],
                    serde_json::to_value(transport_policy_ref(&receipt.turn_policy_snapshot_ref))
                        .unwrap()
                );
                let receipt_registry = WorldWorkReceiptRegistry::bind(
                    Path::new(&observation.observation.bootstrap_home.physical_path),
                    &observation.observation.bootstrap_home,
                    &observation.observation.authority_store_id,
                )
                .unwrap();
                let b1 = receipt_registry
                    .inspect_world_work_acceptance_by_id(
                        &receipt.dispatch_policy_commitment_ref.authority_store_id,
                        &receipt.acceptance_record_id,
                    )
                    .unwrap()
                    .unwrap();
                let proposal = receipt_registry
                    .inspect_world_work_proposal_for_accepted_record(&b1)
                    .unwrap()
                    .unwrap();
                let WorldWorkSubmissionIdentityV1::RetainedTurn {
                    validated_dispatch_request,
                    ..
                } = proposal.submission_identity
                else {
                    panic!("exact retained B1 proposal required")
                };
                assert_eq!(
                    validated_dispatch_request,
                    request.clone().validate_ordinary_representation().unwrap()
                );
                let patch = request.dispatch_policy_narrowing.as_ref().unwrap();
                assert!(!patch
                    .restricted_policy_patch
                    .world_fs
                    .as_ref()
                    .unwrap()
                    .is_empty());
                let key = DispatchPolicyCommitmentLookupKeyV1 {
                    authority_store_id: b1.authority_store_id.clone(),
                    orchestration_session_id: b1.orchestration_session_id.clone(),
                    request_id: b1.request_id.clone(),
                    subject: DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerTurn {
                        active_run_id: receipt.active_run_id.clone(),
                        message_id: receipt.message_id.clone(),
                    },
                };
                let AcceptedWorkReceiptMaterialResolutionV1::Resolved(e2) =
                    resolve_accepted_work_receipt_material(selected.held_authority(), &key, &b1)
                        .unwrap()
                else {
                    panic!("exact B1/E2 routing narrowing material required")
                };
                assert_eq!(e2.commitment_ref(), &receipt.dispatch_policy_commitment_ref);
                assert_eq!(e2.acceptance_record_id(), receipt.acceptance_record_id);
                assert_eq!(e2.policy_snapshot_bytes(), bytes);
                assert_eq!(e2.policy_snapshot_hash(), receipt.turn_policy_snapshot_hash);
                let cap = e2.retained_worker_cap().unwrap();
                assert_eq!(
                    serde_json::to_value(cap.cap_ref()).unwrap(),
                    serde_json::to_value(&prepared.e2_launch_activation.immutable_worker_cap_ref)
                        .unwrap()
                );
                assert_eq!(
                    cap.cap_policy_snapshot_bytes(),
                    prepared.exact_policy_snapshot.policy_snapshot_bytes
                );
            }
            assert_eq!(
                format!("{:x}", sha2::Sha256::digest(&bytes)),
                carrier["policy_snapshot_hash"].as_str().unwrap()
            );
        }
        assert_eq!(turn["body"]["world_generation"], world.world_generation);
        assert_eq!(
            turn["body"]["policy_snapshot_carrier"]["immutable_worker_cap_ref"],
            serde_json::to_value(
                &transport
                    .e2_launch_activation
                    .as_ref()
                    .unwrap()
                    .immutable_worker_cap_ref
            )
            .unwrap()
        );
    }
    let exact_turn = received
        .iter()
        .rev()
        .find(|row| row["path"] == "/v1/ordinary/member_turn/stream")
        .unwrap()["body"]
        .clone();
    let exact_ordinary: transport_api_types::OrdinaryMemberTurnSubmitRequestV1 =
        serde_json::from_value(exact_turn.clone()).unwrap();
    if initial_zero {
        let native_before = observations(&cwd.join("native-observations.jsonl"));
        let gateways_before = observations(&root.join("gateway-observations.jsonl"));
        let registry_before = component_tree_snapshot(&root.join("turn-state"));
        assert!(
            client
                .submit_member_turn_stream(exact_ordinary.as_request().clone())
                .await
                .is_err(),
            "ordinary zero cannot enter the protected codec/route"
        );
        for mutation in 0..4 {
            let mut bad = exact_turn.clone();
            match mutation {
                0 => {
                    bad["world_id"] = "unknown-world".into();
                    bad["policy_snapshot_carrier"]["target_world"]["world_id"] =
                        "unknown-world".into();
                }
                1 => {
                    bad["world_generation"] = 1.into();
                    bad["policy_snapshot_carrier"]["target_world"]["world_generation"] = 1.into();
                }
                2 => {
                    bad["policy_snapshot_carrier"]["immutable_worker_cap_ref"]
                        ["exact_linkage_hash"] = "e".repeat(64).into()
                }
                3 => {
                    bad["participant_id"] = "unknown-ordinary-member".into();
                    bad["policy_snapshot_carrier"]["subject"]["retained_participant_id"] =
                        "unknown-ordinary-member".into();
                }
                _ => unreachable!(),
            }
            // Every negative passes the strict ordinary representation codec:
            // real admission, not parser failure, must refuse its authority.
            let bad: transport_api_types::OrdinaryMemberTurnSubmitRequestV1 =
                serde_json::from_value(bad).unwrap();
            assert!(client
                .submit_ordinary_member_turn_stream(bad)
                .await
                .is_err());
            assert_eq!(
                component_tree_snapshot(&root.join("turn-state")),
                registry_before
            );
        }
        let replay = client
            .submit_ordinary_member_turn_stream(exact_ordinary.clone())
            .await
            .unwrap();
        let replay = frames(replay, Arc::new(std::sync::Mutex::new(Vec::new()))).await;
        assert!(replay
            .iter()
            .any(|frame| matches!(frame, ExecuteStreamFrame::Exit { exit: 0, .. })));
        assert_eq!(
            component_tree_snapshot(&root.join("turn-state")),
            registry_before
        );
        assert_eq!(
            observations(&cwd.join("native-observations.jsonl")),
            native_before
        );
        assert_eq!(
            observations(&root.join("gateway-observations.jsonl")),
            gateways_before
        );
    }
    // Use a new, genuinely admitted but unconsumed Spawn. An already-routable
    // bootstrap replay cannot discriminate held-source revocation.
    let fresh_request = WorldDispatchRequestV1 {
        request_id: Some("req_component_source_revocation".into()),
        idempotency_key: Some("idem_component_source_revocation".into()),
        orchestration_session_id: Some(fixture.context.orchestration_session_id.clone()),
        caller_participant_id: Some(fixture.context.original_participant_id.clone()),
        action: WorldDispatchActionV1::SpawnWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(transport.backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(world.world_id.clone()),
        world_generation: Some(world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
            prompt: "unconsumed source-revocation discriminator".into(),
        }),
    }
    .validate()
    .unwrap();
    let fresh =
        prepare_ordinary_authority_bound_spawn_world_worker(Arc::clone(&selected), fresh_request)
            .unwrap();
    assert_ne!(
        fresh.launch_authority_proof.retained_participant_id,
        transport.participant_id
    );
    assert_ne!(
        fresh.launch_authority_proof.bootstrap_run_id,
        transport.run_id
    );
    let fresh_transport = crate::execution::routing::MemberDispatchTransportRequest {
        participant_id: fresh.launch_authority_proof.retained_participant_id.clone(),
        run_id: fresh.launch_authority_proof.bootstrap_run_id.clone(),
        initial_prompt: Some("unconsumed source-revocation discriminator".into()),
        retained_worker_launch_authority: Some(fresh.launch_authority_proof.clone()),
        e2_launch_activation: Some(fresh.e2_launch_activation.clone()),
        exact_policy_snapshot: Some(fresh.exact_policy_snapshot.clone()),
        ..transport.clone()
    };
    let (fresh_client, fresh_execute, _) =
        build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
            &fresh.selection,
            &fresh_transport,
            cwd,
        )
        .unwrap();
    let fresh_admission = fresh
        .selection
        .prepare_world_execute(&fresh_execute)
        .unwrap();
    fresh_admission.revalidate_before_world_effect().unwrap();
    if initial_zero {
        let actual: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("component-world-generation.json")).unwrap(),
        )
        .unwrap();
        let binding: world_api::SharedWorldBindingSnapshot =
            serde_json::from_value(actual["selected_actual_binding"].clone()).unwrap();
        fresh_admission.revalidate_for_effect(&binding).unwrap();
        for state in [
            world_api::SharedWorldBindingState::Replacing,
            world_api::SharedWorldBindingState::Replaced,
            world_api::SharedWorldBindingState::Abandoned,
        ] {
            let mut revoked = binding.clone();
            revoked.binding_state = state;
            assert!(fresh_admission.revalidate_for_effect(&revoked).is_err());
        }
        let mut stale = binding.clone();
        stale.world_generation = 1;
        assert!(fresh_admission.revalidate_for_effect(&stale).is_err());
        let mut wrong = binding;
        wrong.world_id = "unknown-world".into();
        assert!(fresh_admission.revalidate_for_effect(&wrong).is_err());
    }
    let mut service_account_revocation = serde_json::Value::Null;
    let mut actual_backend_revocation = serde_json::Value::Null;
    let mut replacement_cgroup = None;

    if current_effects {
        let native_before = observations(&cwd.join("native-observations.jsonl"));
        let gateways_before = observations(&root.join("gateway-observations.jsonl"));
        let before = component_tree_snapshot(&root.join("turn-state"));
        std::fs::write(
            root.join("service-account-control.json"),
            serde_json::to_vec(&serde_json::json!({"sequence":1,"state":"revoked"})).unwrap(),
        )
        .unwrap();
        assert!(
            client
                .submit_ordinary_member_turn_stream(exact_ordinary.clone())
                .await
                .is_err(),
            "service-child changed account must precede accepted replay"
        );
        let ack: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("service-account-ack.json")).unwrap())
                .unwrap();
        assert_eq!(ack["service_pid"], service.0.id());
        assert_eq!(ack["sequence"], 1);
        assert_eq!(ack["state"], "revoked");
        assert_eq!(
            std::env::var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID).unwrap(),
            "synthetic-ordinary-account",
            "parent account remains original"
        );
        assert!(
            fresh_client
                .execute_stream(fresh_execute.clone())
                .await
                .is_err(),
            "prebuilt fresh request must be refused by the independent service account check"
        );
        assert_eq!(component_tree_snapshot(&root.join("turn-state")), before);
        assert_eq!(
            observations(&cwd.join("native-observations.jsonl")),
            native_before
        );
        assert_eq!(
            observations(&root.join("gateway-observations.jsonl")),
            gateways_before
        );
        std::fs::write(
            root.join("service-account-control.json"),
            serde_json::to_vec(&serde_json::json!({"sequence":2,"state":"original"})).unwrap(),
        )
        .unwrap();
        let replay = client
            .submit_ordinary_member_turn_stream(exact_ordinary.clone())
            .await
            .unwrap();
        let replay = frames(replay, Arc::new(std::sync::Mutex::new(Vec::new()))).await;
        assert!(replay
            .iter()
            .any(|frame| matches!(frame, ExecuteStreamFrame::Exit { exit: 0, .. })));
        let restored: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("service-account-ack.json")).unwrap())
                .unwrap();
        assert_eq!(restored["service_pid"], service.0.id());
        assert_eq!(restored["sequence"], 2);
        assert_eq!(restored["state"], "original");
        std::fs::remove_file(root.join("service-account-control.json")).unwrap();
        assert_eq!(component_tree_snapshot(&root.join("turn-state")), before);
        assert_eq!(
            observations(&cwd.join("native-observations.jsonl")),
            native_before
        );
        assert_eq!(
            observations(&root.join("gateway-observations.jsonl")),
            gateways_before
        );
        service_account_revocation = serde_json::json!({"revoked_ack":ack,
            "restored_ack":restored,"parent_account_unchanged":true,
            "fresh_and_accepted_replay_refused":true,"redelivery_after_restoration":false});
    }
    let mut initial_fork_component = serde_json::Value::Null;
    if let Some(fork_execute) = initial_fork_execute {
        initial_fork_component = exercise_ordinary_initial_fork_for_test(
            &client,
            fork_execute,
            root,
            cwd,
            service.0.id(),
            &transport.participant_id,
            &native,
        )
        .await;
    }
    let received_before_revocation = observations(&root.join("received.jsonl"));
    assert_eq!(
        received_before_revocation.iter().filter(|row|
            row["body"]["member_dispatch"]["participant_id"] == fresh_transport.participant_id
        ).count(),
        if current_effects { 1 } else { 0 },
        "fresh discriminator has only the explicit refused service-account delivery; never a native effect"
    );
    let native_before_policy = observations(&cwd.join("native-observations.jsonl"));
    let gateways_before_policy = observations(&root.join("gateway-observations.jsonl"));
    let policy_path = fixture.home.join("policy.yaml");
    std::fs::write(
        &policy_path,
        component_policy.replace(
            if claude {
                "  allowed_backends: [cli:codex-host, cli:claude_code-world]\n"
            } else {
                "  allowed_backends: [cli:codex-host, cli:codex-world]\n"
            },
            "  allowed_backends: []\n",
        ),
    )
    .unwrap();
    if initial_zero {
        let before = component_tree_snapshot(&root.join("turn-state"));
        assert!(
            client
                .submit_ordinary_member_turn_stream(exact_ordinary.clone())
                .await
                .is_err(),
            "current policy denial must precede ordinary accepted-request replay"
        );
        assert_eq!(component_tree_snapshot(&root.join("turn-state")), before);
    }
    assert!(
        build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
            &fresh.selection,
            &fresh_transport,
            cwd
        )
        .is_err()
    );
    if current_effects {
        let before = component_tree_snapshot(&root.join("turn-state"));
        assert!(
            fresh_client
                .execute_stream(fresh_execute.clone())
                .await
                .is_err(),
            "current service policy must refuse the prebuilt fresh request"
        );
        assert_eq!(component_tree_snapshot(&root.join("turn-state")), before);
    }
    std::fs::write(&policy_path, component_policy).unwrap();
    assert_eq!(
        observations(&cwd.join("native-observations.jsonl")),
        native_before_policy
    );
    assert_eq!(
        observations(&root.join("gateway-observations.jsonl")),
        gateways_before_policy
    );
    if current_effects {
        // A positive restored-policy replay separates policy refusal from
        // cancellation or backend revocation, which happen only afterward.
        let before = component_tree_snapshot(&root.join("turn-state"));
        let replay = client
            .submit_ordinary_member_turn_stream(exact_ordinary.clone())
            .await
            .unwrap();
        let replay = frames(replay, Arc::new(std::sync::Mutex::new(Vec::new()))).await;
        assert!(replay
            .iter()
            .any(|frame| matches!(frame, ExecuteStreamFrame::Exit { exit: 0, .. })));
        assert_eq!(component_tree_snapshot(&root.join("turn-state")), before);
        assert_eq!(
            observations(&cwd.join("native-observations.jsonl")),
            native_before_policy
        );
        assert_eq!(
            observations(&root.join("gateway-observations.jsonl")),
            gateways_before_policy
        );
    }

    build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
        &fresh.selection,
        &fresh_transport,
        cwd,
    )
    .unwrap();
    if !claude {
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-revoked-account",
        );
        assert!(fresh
            .selection
            .revalidate_requested_account(
                &crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                    cwd,
                    &prepared.authority.bootstrap_home()
                )
                .unwrap()
            )
            .is_err());
        assert!(
            build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
                &fresh.selection,
                &fresh_transport,
                cwd,
            )
            .is_err(),
            "changed account must refuse before constructing a fresh delivery"
        );
        std::env::set_var(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "synthetic-ordinary-account",
        );
        build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
            &fresh.selection,
            &fresh_transport,
            cwd,
        )
        .unwrap();
    }
    fresh_admission.revalidate_before_world_effect().unwrap();
    let mut returned_control_cancel = serde_json::Value::Null;
    if current_effects {
        assert!(
            !claude,
            "the account revocation discriminator targets Codex account identity"
        );
        let request = WorldDispatchRequestV1 {
            request_id: Some("req_component_cancel_turn_4".into()),
            idempotency_key: Some("idem_component_cancel_turn_4".into()),
            orchestration_session_id: Some(transport.orchestration_session_id.clone()),
            caller_participant_id: Some(transport.orchestrator_participant_id.clone()),
            action: WorldDispatchActionV1::ContinueWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(transport.backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(transport.participant_id.clone()),
            world_id: Some(world.world_id.clone()),
            world_generation: Some(world.world_generation),
            dispatch_policy_narrowing: None,
            payload: WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                prompt: "ordinary component turn 4".into(),
                thread_id: Some("thread-ordinary-world-component".into()),
            }),
        };
        let outcome = tokio::time::timeout(
            COMPONENT_STARTUP_BUDGET,
            continue_ordinary_world_worker(&store, Arc::clone(&selected), carrier.clone(), request),
        )
        .await
        .unwrap()
        .unwrap();
        let WorldDispatchOutcomeV1::AcceptedForeground(receipt) = outcome else {
            panic!("cancel requires actual returned foreground receipt")
        };
        let AcceptedForegroundReceiptV1::Retained(receipt) = *receipt else {
            panic!("cancel requires actual retained receipt")
        };
        wait_until(
            || cwd.join("ready-turn-4").is_file(),
            "actual fourth native after returned control",
            COMPONENT_STARTUP_BUDGET,
        )
        .await;
        let native_pid: u64 = std::fs::read_to_string(cwd.join("ready-turn-4"))
            .unwrap()
            .parse()
            .unwrap();
        let gateway = observations(&root.join("gateway-observations.jsonl"))
            .last()
            .unwrap()
            .clone();
        observe_terminal_gate(native_pid, gateway["pid"].as_u64().unwrap(), || {
            supervisor
                .inspect_observation_by_acceptance_id(&receipt.acceptance_record_id)
                .unwrap()
                .and_then(|observation| observation.terminal)
                .is_some()
        })
        .await;
        let observation = supervisor
            .inspect_observation_by_acceptance_id(&receipt.acceptance_record_id)
            .unwrap()
            .unwrap();
        let span = observation
            .journal
            .iter()
            .find_map(
                |entry| match entry.decode_for_c1(&observation.claim).unwrap() {
                    ExecuteStreamFrame::Start { span_id, .. } => Some(span_id),
                    _ => None,
                },
            )
            .expect("cancel uses the actual durably observed Start span");
        assert_eq!(span, observation.claim.runtime_submission_id);
        let cancel = client
            .cancel_execute(transport_api_types::ExecuteCancelRequestV1 {
                span_id: span.clone(),
                sig: "INT".into(),
            })
            .await
            .unwrap();
        assert!(
            cancel.delivered,
            "actual independent service must accept cancellation"
        );
        wait_until(
            || {
                supervisor
                    .inspect_observation_by_acceptance_id(&receipt.acceptance_record_id)
                    .unwrap()
                    .and_then(|observation| observation.terminal)
                    .is_some_and(|terminal| terminal.exit_code != 0)
                    && observations(&root.join("sdk-completions.jsonl")).len() == 4
                    && observations(&root.join("gateway-terminals.jsonl")).len() == 4
            },
            "actual cancelled SDK completion and owned gateway terminal",
            Duration::from_secs(20),
        )
        .await;
        let completed = observations(&root.join("sdk-completions.jsonl"))
            .last()
            .unwrap()
            .clone();
        assert_eq!(completed["success"], false);
        assert_eq!(completed["handoff"], "joined_used");
        let gateway_terminal = observations(&root.join("gateway-terminals.jsonl"))
            .last()
            .unwrap()
            .clone();
        assert_eq!(gateway_terminal["pid"], gateway["pid"]);
        assert_eq!(gateway_terminal["native_open"], false);
        assert_eq!(gateway_terminal["owned_child_terminal"], true);
        assert_eq!(gateway_terminal["manifest_and_entry_disposed"], true);
        assert!(
            !cwd.join("release-turn-4").exists(),
            "normal gate was never released"
        );
        let terminal = supervisor
            .inspect_observation_by_acceptance_id(&receipt.acceptance_record_id)
            .unwrap()
            .unwrap()
            .terminal
            .unwrap();
        let cancelled_native = observations(&cwd.join("native-observations.jsonl"))
            .last()
            .unwrap()
            .clone();
        assert!(
            !pids.contains(&native_pid),
            "cancelled turn uses a distinct actual native process"
        );
        assert_eq!(cancelled_native["parent_pid"], service.0.id());
        for identity in [
            "home",
            "home_device",
            "home_inode",
            "history_device",
            "history_inode",
        ] {
            assert_eq!(
                cancelled_native[identity], native[0][identity],
                "cancelled Resume retains the exact original history: {identity}"
            );
        }
        assert_eq!(
            gateway_terminal["runtime"]["run_id"],
            "req_component_cancel_turn_4"
        );
        assert_eq!(
            gateway_terminal["runtime"]["participant_id"],
            transport.participant_id
        );
        assert_eq!(gateway_terminal["runtime"]["world_id"], world.world_id);
        assert_eq!(
            gateway_terminal["runtime"]["world_generation"],
            world.world_generation
        );
        assert_eq!(
            std::fs::read_to_string(root.join("gateway-native-requests"))
                .unwrap()
                .lines()
                .count(),
            4
        );
        returned_control_cancel = serde_json::json!({
            "actual_span":span,"terminal":terminal,"sdk_completion":completed,
            "gateway":gateway,"gateway_terminal":gateway_terminal,
            "normal_exit_gate_released":false,"native":cancelled_native
        });
    }
    if current_effects {
        let native_before = observations(&cwd.join("native-observations.jsonl"));
        let gateways_before = observations(&root.join("gateway-observations.jsonl"));
        let before = component_tree_snapshot(&root.join("turn-state"));
        // Revoke the actual persisted backend binding, not an in-memory clone.
        let mut spec = fresh_admission
            .current_world_parent_policy()
            .unwrap()
            .world_spec()
            .clone();
        spec.reuse_mode =
            world_api::WorldReuseMode::SharedOrchestration(world_api::SharedWorldOwnerSpec {
                orchestration_session_id: transport.orchestration_session_id.clone(),
                action: world_api::SharedWorldOwnerAction::ReplaceExpectedGeneration {
                    expected_generation: world.world_generation,
                    reason:
                        "component actual backend revocation discriminator after terminal turns"
                            .into(),
                },
            });
        let backend = world::LinuxLocalBackend::new();
        let replacement = backend.ensure_session(&spec).unwrap();
        let binding = replacement.shared_binding.as_ref().unwrap();
        assert_ne!(binding.world_id, world.world_id);
        assert_eq!(binding.world_generation, world.world_generation + 1);
        let recovered = world::LinuxLocalBackend::new()
            .find_compatible_session(&spec)
            .unwrap()
            .unwrap();
        assert_eq!(recovered.shared_binding, replacement.shared_binding);
        assert!(fresh_admission.revalidate_for_effect(binding).is_err());
        assert!(
            client
                .submit_ordinary_member_turn_stream(exact_ordinary.clone())
                .await
                .is_err(),
            "actual backend replacement must precede accepted replay"
        );
        assert!(
            fresh_client
                .execute_stream(fresh_execute.clone())
                .await
                .is_err(),
            "actual backend replacement must refuse a prebuilt fresh request for the old binding"
        );
        assert_eq!(component_tree_snapshot(&root.join("turn-state")), before);
        assert_eq!(
            observations(&cwd.join("native-observations.jsonl")),
            native_before
        );
        assert_eq!(
            observations(&root.join("gateway-observations.jsonl")),
            gateways_before
        );
        replacement_cgroup = Some(backend.cgroup_path(&replacement).unwrap());
        actual_backend_revocation = serde_json::json!({"old_world":world,
            "replacement":binding,"fresh_backend_observed_replacement":true,
            "fresh_and_accepted_replay_refused":true,"new_native_effects":0});
    }

    let requests_before_source_revocation = observations(&root.join("received.jsonl")).len();
    let native_before_revocation = observations(&cwd.join("native-observations.jsonl"));
    let gateway_before_revocation = observations(&root.join("gateway-observations.jsonl"));
    std::fs::set_permissions(
        root.join("bootstrap/active.json"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert_eq!(
        fixture.configured.revalidate().unwrap_err(),
        config_projection::ConfigProjectionFailureV1::UnsupportedSecurityPosture
    );
    assert!(fresh.selection.revalidate().is_err());
    assert!(fresh_admission.revalidate_before_world_effect().is_err());
    if initial_zero {
        let before = component_tree_snapshot(&root.join("turn-state"));
        assert!(
            client
                .submit_ordinary_member_turn_stream(exact_ordinary)
                .await
                .is_err(),
            "revoked configured custody must precede ordinary accepted-request replay"
        );
        assert_eq!(component_tree_snapshot(&root.join("turn-state")), before);
    }
    let revocation_error = match fresh_client.execute_stream(fresh_execute.clone()).await {
        Ok(_) => panic!("independent service admitted a fresh Spawn after held-source revocation"),
        Err(error) => error,
    };
    assert!(
        revocation_error
            .to_string()
            .contains("UnsupportedSecurityPosture"),
        "fresh request must refuse for revoked source custody: {revocation_error:#}"
    );
    let received_after_revocation = observations(&root.join("received.jsonl"));
    assert_eq!(
        received_after_revocation.len(),
        if current_effects {
            requests_before_source_revocation + 2
        } else {
            received_before_revocation.len() + 1 + if initial_zero { 2 } else { 0 }
        }
    );
    assert_eq!(
        received_after_revocation.last().unwrap()["body"],
        serde_json::to_value(&fresh_execute).unwrap()
    );
    assert_eq!(
        observations(&cwd.join("native-observations.jsonl")),
        native_before_revocation
    );
    assert_eq!(
        observations(&root.join("gateway-observations.jsonl")),
        gateway_before_revocation
    );
    assert_eq!(
        observations(&cwd.join("native-observations.jsonl")).len(),
        if current_effects { 4 } else { 3 }
    );
    std::fs::write(root.join("stop-service"), b"stop").unwrap();
    wait_until(
        || service.0.try_wait().unwrap().is_some(),
        "service graceful shutdown",
        Duration::from_secs(15),
    )
    .await;
    let service_exit = service.0.wait().unwrap();
    let service_tmp_after = std::fs::metadata(service_tmp.path()).unwrap();
    assert_eq!(
        (service_tmp_identity.dev(), service_tmp_identity.ino()),
        (service_tmp_after.dev(), service_tmp_after.ino())
    );
    let service_tmp_path = service_tmp.path().to_path_buf();
    service_tmp.close().unwrap();
    assert!(
        !service_tmp_path.exists(),
        "owned service scratch disposal after actual service wait"
    );
    if let Some(evidence) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") {
        std::fs::write(
            PathBuf::from(evidence).join("owned-service-tmp-cleanup.json"),
            serde_json::to_vec_pretty(
                &serde_json::json!({"path":service_tmp_path,"dev":service_tmp_identity.dev(),
                "ino":service_tmp_identity.ino(),"actual_service_wait":service_exit.to_string(),
                "owned_temp_close_succeeded":true,"path_absent":true}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    eprintln!(
        "END independently invoked ordinary service worker: pid={} exit={service_exit}",
        service.0.id()
    );
    assert!(service_exit.success());
    // History survived all turns. Its service-owned scratch was explicitly
    // closed only after actual service wait; verify that exact namespace here.
    let home = PathBuf::from(native[0]["home"].as_str().unwrap());
    let launcher = home
        .ancestors()
        .find(|path| {
            path.file_name().is_some_and(|name| {
                name.to_string_lossy()
                    .starts_with("substrate-member-runtime-entry-")
            })
        })
        .unwrap();
    assert_eq!(launcher.parent(), Some(service_tmp_path.as_path()));
    assert!(
        !launcher.exists(),
        "owned scratch close must dispose the launcher"
    );
    assert!(
        !home.exists(),
        "owned scratch close must dispose terminal history"
    );
    std::fs::remove_dir_all(&fixture.component_cgroup).unwrap();
    if let Some(path) = replacement_cgroup {
        assert!(path.starts_with(
            PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").unwrap()).join("substrate/cgroup")
        ));
        std::fs::remove_dir_all(path).unwrap();
    }
    let proof = serde_json::json!({"component_only":true,"installed_qualified":false,"kernel_isolation_qualified":false,"owned_fixture_cleanup_complete":true,
        "canonical_guest_descriptor":!claude,"checked_local_executable_component_mapping":true,"guest_installation_qualified":false,"ordinary_claude":claude,"claude_exact_configured_local_descriptor":claude,"production_start_builder":true,"production_continue_calls":if current_effects {3} else {2},"normal_terminal_gate_checks":3,"normal_terminal_gate_window_ms":100,"native":native,"gateway":gateways,"gateway_terminals":terminals,
        "world_generation_fixture":serde_json::from_slice::<serde_json::Value>(&std::fs::read(root.join("component-world-generation.json")).unwrap()).unwrap(),
        "synthetic_secrets_only":true,"provider_requests":0,"accepted_retry_redeliveries":0,
        "authentic_initial_generation_zero":initial_zero,"ordinary_zero_wire_refusals_and_exact_replay_checked":initial_zero,
        "nonactive_binding_snapshot_refusals_checked":initial_zero,"actual_backend_revocation_performed":current_effects,
        "sdk_partial_startup_settlement_qualified":false,
        "returned_control_cancel":returned_control_cancel,
        "service_child_account_revocation":service_account_revocation,
        "actual_backend_revocation":actual_backend_revocation,
        "initial_fork_component":initial_fork_component,
        "native_total":observations(&cwd.join("native-observations.jsonl")),
        "fresh_unconsumed_spawn_source_revocation_checked":true,"source_revocation_error":revocation_error.to_string(),
        "executable_pair":executable_pair,"service_worker_argv":["--exact",SERVICE_WORKER,"--ignored","--nocapture"],
        "service_worker_pid":service.0.id(),"service_worker_exit_code":service_exit.code(),
        "test_result_accounting":"one explicit shell integration; one nested independent service worker; never sum harness result lines"});
    eprintln!("ordinary sealed World integration: {proof}");
    if let Some(evidence) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") {
        let evidence = PathBuf::from(evidence);
        std::fs::create_dir_all(&evidence).unwrap();
        std::fs::write(
            evidence.join("integration-proof.json"),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
        std::fs::copy(root.join("received.jsonl"), evidence.join("received.jsonl")).unwrap();
    }
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires exact candidate service/helper binaries; ordinary zero narrowing component only"]
async fn test_ordinary_configured_zero_narrowing_uses_original_issuer_and_no_redelivery() {
    tokio::time::timeout(
        COMPONENT_OUTER_BUDGET,
        Box::pin(ordinary_configured_spawn_start_and_two_resumes_body(
            true, false, false, true,
        )),
    )
    .await
    .expect("ordinary zero narrowing component deadline");
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires exact candidate service/helper binaries; supported ordinary zero routing narrowing component only"]
async fn test_ordinary_configured_zero_routing_narrowing_uses_original_issuer_and_no_redelivery() {
    tokio::time::timeout(
        COMPONENT_OUTER_BUDGET,
        Box::pin(
            ordinary_configured_spawn_start_and_two_resumes_with_posture(
                true,
                false,
                false,
                OrdinaryComponentNarrowing::RoutingFailClosed,
                false,
            ),
        ),
    )
    .await
    .expect("ordinary zero routing narrowing component deadline");
}

#[test]
fn test_ordinary_unchanged_parent_admission_is_exact_and_recovers() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*,
        dispatch_policy_commitment::*,
        retained_worker_runtime::{
            test_ordinary_admission_scope_rejects_substitution, RetainedWorkerAdmissionStateV1,
            RetainedWorkerRuntime,
        },
    };
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_authority_bound_spawn_world_worker;
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let selected = Arc::new(
        OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(Arc::clone(
            &fixture.configured,
        ))
        .unwrap()
        .authenticate(&fixture.context.orchestration_session_id)
        .unwrap(),
    );
    let world = fixture.context.original_world_binding.as_ref().unwrap();
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_ordinary_unchanged".into()),
        idempotency_key: Some("idem_ordinary_unchanged".into()),
        orchestration_session_id: Some(fixture.context.orchestration_session_id.clone()),
        caller_participant_id: Some(fixture.context.original_participant_id.clone()),
        action: WorldDispatchActionV1::SpawnWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(selected.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(world.world_id.clone()),
        world_generation: Some(world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
            prompt: "unchanged ordinary request".into(),
        }),
    }
    .validate()
    .unwrap();
    let prepared =
        prepare_ordinary_authority_bound_spawn_world_worker(Arc::clone(&selected), request.clone())
            .unwrap();
    let plan = &prepared.admission_plan;
    let proof = &prepared.dispatch_policy_reservation_proof;
    assert!(plan.policy_and_admission_cap.allow_capability_narrowing);
    assert!(
        !proof.permits_capability_narrowing(),
        "permission is not an applied patch attestation"
    );
    let authority = &prepared.authority;
    let runtime = RetainedWorkerRuntime;
    let before = runtime
        .read_admission_record(
            authority,
            &plan.spawn_request.orchestration_session_id,
            proof.retained_participant_id(),
        )
        .unwrap()
        .unwrap();
    let joined = runtime
        .reserve_admission_slot(authority, plan, Some(proof))
        .unwrap();
    assert!(joined.joined);
    assert_eq!(
        joined.record.retained_participant_id,
        before.retained_participant_id
    );
    assert_eq!(joined.record.bootstrap_run_id, before.bootstrap_run_id);
    let reference = prepared
        .e2_launch_activation
        .reservation_ref
        .as_ref()
        .unwrap();
    let reference = DispatchPolicyCommitmentReservationRefV1 {
        authority_store_id: reference.authority_store_id.clone(),
        reservation_id: reference.reservation_id.clone(),
        reservation_hash: reference.reservation_hash.clone(),
    };
    let snapshot = validate_policy_snapshot_material(
        &prepared.exact_policy_snapshot.policy_snapshot,
        &prepared.exact_policy_snapshot.policy_snapshot_bytes,
        &plan.policy_and_admission_cap.current_policy_ref,
        &prepared.exact_policy_snapshot.policy_snapshot_hash,
        &plan.policy_and_admission_cap.current_policy.policy_revision,
    )
    .unwrap();
    let bare = authenticate_fresh_spawn_reservation(
        authority,
        &reference,
        &plan.spawn_request,
        plan,
        &snapshot,
    )
    .unwrap();
    assert!(!bare.permits_capability_narrowing());
    assert!(
        runtime
            .reserve_admission_slot(authority, plan, Some(&bare))
            .is_err(),
        "a bare generic proof from a real ordinary reservation grants no exception"
    );
    let registry_path = fixture
        ._temp
        .path()
        .join("accepted/authority-v1/retained-worker-admission-v1/registry-v1.json");
    let registry_before = std::fs::read(&registry_path).unwrap();
    for mutation in 0..12 {
        let mut wrong = plan.clone();
        match mutation {
            0 => wrong.spawn_request.request_id.push_str("_wrong"),
            1 => wrong.spawn_request.idempotency_key.push_str("_wrong"),
            2 => wrong
                .spawn_request
                .orchestration_session_id
                .push_str("_wrong"),
            3 => wrong.spawn_request.caller_participant_id.push_str("_wrong"),
            4 => wrong.spawn_request.target_backend_id.push_str("_wrong"),
            5 => wrong.spawn_request.world_id.push_str("_wrong"),
            6 => wrong.spawn_request.world_generation += 1,
            7 => wrong.spawn_request.payload.prompt.push_str("_wrong"),
            8 => {
                wrong.policy_and_admission_cap.current_policy_ref.ref_id =
                    format!("ao_{}", "0".repeat(32))
            }
            9 => wrong
                .policy_and_admission_cap
                .current_policy
                .policy_revision
                .push_str("_wrong"),
            10 => wrong
                .exact_authority
                .authority
                .workspace_binding
                .authority_store_root
                .physical_path
                .push_str("_wrong"),
            _ => wrong.issuer_request_id.push_str("_wrong"),
        }
        assert!(
            runtime
                .reserve_admission_slot(authority, &wrong, Some(proof))
                .is_err(),
            "substitution {mutation}"
        );
        assert_eq!(std::fs::read(&registry_path).unwrap(), registry_before);
    }
    let mut wrong_after_check = plan.clone();
    wrong_after_check
        .spawn_request
        .payload
        .prompt
        .push_str("_after_check");
    assert!(
        test_ordinary_admission_scope_rejects_substitution(
            authority,
            plan,
            proof,
            &wrong_after_check
        )
        .unwrap(),
        "E2 check A cannot be paired with the B3 local borrow of request B"
    );
    if let Ok(unsealed) = authenticate_ordinary_selection_from_authority(
        Arc::clone(authority),
        &plan.spawn_request.orchestration_session_id,
    ) {
        assert!(
            AuthenticatedOrdinaryFreshSpawnInputV1::authenticate(
                Arc::new(unsealed),
                FreshSpawnReservationInputV1 {
                    spawn_request: plan.spawn_request.clone(),
                    caller_backend_id: plan.exact_authority.caller_descriptor.backend_id.clone(),
                    parent_policy_ref: plan.policy_and_admission_cap.current_policy_ref.clone(),
                    parent_policy_revision: plan
                        .policy_and_admission_cap
                        .current_policy
                        .policy_revision
                        .clone(),
                    applied_patch: AppliedDispatchPolicyPatchIdentityV1::UnchangedParent,
                    policy_snapshot: snapshot.clone(),
                    reason: None,
                }
            )
            .is_err(),
            "held authority alone cannot mint ordinary provenance"
        );
    }
    assert!(prepare_ordinary_authority_bound_spawn_world_worker(
        Arc::clone(&prepared.selection),
        request
    )
    .is_err());
    let recovered = runtime
        .read_admission_record(
            authority,
            &plan.spawn_request.orchestration_session_id,
            proof.retained_participant_id(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        recovered.retained_participant_id,
        before.retained_participant_id
    );
    assert_eq!(recovered.bootstrap_run_id, before.bootstrap_run_id);
    assert_eq!(
        recovered.canonical_spawn_fingerprint,
        before.canonical_spawn_fingerprint
    );
    // A previously authenticated condition remains usable for immutable cleanup,
    // but current false permission cannot mint a new condition for a forged true cap.
    let policy_path = fixture._temp.path().join("accepted/policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap();
    assert!(policy.contains("allow_capability_narrowing: true"));
    std::fs::write(
        &policy_path,
        policy.replace(
            "allow_capability_narrowing: true",
            "allow_capability_narrowing: false",
        ),
    )
    .unwrap();
    std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let actual_false = AuthenticatedOrdinaryFreshSpawnInputV1::authenticate(
        Arc::clone(&prepared.selection),
        FreshSpawnReservationInputV1 {
            spawn_request: plan.spawn_request.clone(),
            caller_backend_id: plan.exact_authority.caller_descriptor.backend_id.clone(),
            parent_policy_ref: plan.policy_and_admission_cap.current_policy_ref.clone(),
            parent_policy_revision: plan
                .policy_and_admission_cap
                .current_policy
                .policy_revision
                .clone(),
            applied_patch: AppliedDispatchPolicyPatchIdentityV1::UnchangedParent,
            policy_snapshot: snapshot,
            reason: None,
        },
    )
    .unwrap();
    let exact_retry = reserve_ordinary_fresh_spawn(actual_false).unwrap();
    assert!(
        exact_retry.authenticate_admission_plan(plan).is_err(),
        "a true caller flag is not actual parent permission"
    );
    assert_eq!(std::fs::read(&registry_path).unwrap(), registry_before);
    std::fs::set_permissions(
        fixture._temp.path().join("bootstrap/active.json"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert!(prepared.selection.revalidate().is_err());
    let interrupted_at =
        TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            .unwrap();
    let interrupted = runtime
        .mark_admission_interrupted(
            authority,
            plan,
            proof.retained_participant_id(),
            Some(proof),
            None,
            interrupted_at,
        )
        .unwrap();
    assert!(
        matches!(
            interrupted.state,
            RetainedWorkerAdmissionStateV1::InterruptedNonterminal { .. }
        ),
        "immutable cleanup truth must remain recordable after source permission revocation"
    );
    assert!(!fixture
        ._temp
        .path()
        .join("workspace/native-observations.jsonl")
        .exists());
    std::fs::remove_dir_all(&fixture.component_cgroup).unwrap();
}

#[test]
fn test_ordinary_nonempty_spawn_routing_patch_mints_real_attestation() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*,
        dispatch_policy_commitment::*,
        retained_worker_runtime::{RetainedWorkerAdmissionStateV1, RetainedWorkerRuntime},
    };
    use crate::execution::build_ordinary_agent_client_and_member_dispatch_request_for_cwd;
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_authority_bound_spawn_world_worker;
    use base64::Engine as _;

    // The fixture obtains and recovers its positive binding through the backend,
    // then seals that actual binding in a configured issue/claim/apply Start.
    // No generation is substituted into a previously authenticated request.
    let fixture = fixture_for_harness_with_narrowing(true, false, false, true);
    let selected = Arc::new(
        OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(Arc::clone(
            &fixture.configured,
        ))
        .unwrap()
        .authenticate(&fixture.context.orchestration_session_id)
        .unwrap(),
    );
    let world = fixture.context.original_world_binding.as_ref().unwrap();
    assert!(world.world_generation > 0);
    let current = selected
        .held_authority()
        .resolve_current_exact(selected.original_session_id(), None)
        .unwrap();
    assert_eq!(current.authority.world_binding.as_ref(), Some(world));
    let parent = crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
        Path::new(selected.workspace_root()),
        &selected.held_authority().bootstrap_home(),
    )
    .unwrap();
    assert_eq!(
        parent.snapshot_hash,
        current.current_policy.canonical_policy_snapshot_sha256
    );
    assert!(parent.snapshot.world_fs.host_visible);
    assert!(!parent.snapshot.world_fs.fail_closed.routing);
    assert!(parent.snapshot.world_fs.write.enabled);
    let patch = transport_api_types::DispatchPolicyNarrowingPatchV1 {
        schema_version: 1,
        request_id: "req_ordinary_nonempty_spawn".into(),
        orchestration_session_id: selected.original_session_id().into(),
        caller_participant_id: fixture.context.original_participant_id.clone(),
        target_backend_id: selected.requested_selection().backend_id.clone(),
        target_world: transport_api_types::WorldBindingRefV1 {
            world_id: world.world_id.clone(),
            world_generation: world.world_generation,
        },
        applies_to: transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerSpawn,
        parent_policy_ref: transport_policy_ref(
            current.authority.current_policy_ref.as_ref().unwrap(),
        ),
        parent_policy_revision: current.current_policy.policy_revision.clone(),
        restricted_policy_patch: transport_api_types::RestrictedPolicyPatchV1 {
            world_fs: Some(transport_api_types::RestrictedWorldFsPatchV1 {
                fail_closed: Some(transport_api_types::RestrictedWorldFsFailClosedPatchV1 {
                    routing: Some(true),
                }),
                ..Default::default()
            }),
        },
        reason: Some("supported positive-generation ordinary Spawn restriction".into()),
    };
    let request = WorldDispatchRequestV1 {
        request_id: Some(patch.request_id.clone()),
        idempotency_key: Some("idem_ordinary_nonempty_spawn".into()),
        orchestration_session_id: Some(patch.orchestration_session_id.clone()),
        caller_participant_id: Some(patch.caller_participant_id.clone()),
        action: WorldDispatchActionV1::SpawnWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(patch.target_backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(world.world_id.clone()),
        world_generation: Some(world.world_generation),
        dispatch_policy_narrowing: Some(patch.clone()),
        payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
            prompt: "ordinary genuine nonempty Spawn attestation".into(),
        }),
    }
    .validate()
    .unwrap();
    let prepared =
        prepare_ordinary_authority_bound_spawn_world_worker(Arc::clone(&selected), request.clone())
            .unwrap();
    let authority = &prepared.authority;
    let plan = &prepared.admission_plan;
    let proof = &prepared.dispatch_policy_reservation_proof;
    assert!(plan.policy_and_admission_cap.allow_capability_narrowing);
    assert!(
        proof.permits_capability_narrowing(),
        "real E1/E2 issuance must mint the attestation"
    );
    assert_eq!(plan.spawn_request.world_generation, world.world_generation);
    assert_eq!(
        plan.policy_and_admission_cap.current_policy_ref,
        *current.authority.current_policy_ref.as_ref().unwrap()
    );
    assert_eq!(
        plan.policy_and_admission_cap.current_policy,
        current.current_policy
    );
    let mut expected = parent.snapshot.clone();
    expected.world_fs.fail_closed.routing = true;
    assert_eq!(
        serde_json::to_vec(&prepared.exact_policy_snapshot.policy_snapshot).unwrap(),
        serde_json::to_vec(&expected).unwrap()
    );
    assert_eq!(
        prepared.exact_policy_snapshot.policy_snapshot_bytes,
        serde_json::to_vec(&expected).unwrap()
    );
    assert_ne!(
        prepared.exact_policy_snapshot.policy_snapshot_hash,
        parent.snapshot_hash
    );
    assert_eq!(
        prepared.exact_policy_snapshot.policy_snapshot_hash,
        format!(
            "{:x}",
            sha2::Sha256::digest(&prepared.exact_policy_snapshot.policy_snapshot_bytes)
        )
    );

    let applied = applied_dispatch_policy_patch_identity(Some(&patch)).unwrap();
    assert_ne!(
        applied,
        AppliedDispatchPolicyPatchIdentityV1::UnchangedParent
    );
    let AppliedDispatchPolicyPatchIdentityV1::RestrictedWorldFs {
        patch_schema_version,
        canonical_patch:
            ImmutableBytesMaterialV1::Inline {
                bytes_base64,
                byte_length,
            },
        patch_hash,
    } = &applied
    else {
        panic!("canonical nonempty E1 patch required")
    };
    assert_eq!(*patch_schema_version, 1);
    let canonical_patch = base64::engine::general_purpose::STANDARD
        .decode(bytes_base64)
        .unwrap();
    assert_eq!(*byte_length, canonical_patch.len() as u64);
    // E1/E2 uses recursively key-sorted transport JSON, not HSA A1 enum encoding.
    let mut expected_patch = serde_json::to_value(&patch).unwrap();
    expected_patch.sort_all_objects();
    assert_eq!(
        canonical_patch,
        serde_json::to_vec(&expected_patch).unwrap()
    );
    let mut hash_subject = serde_json::json!({
        "domain":"substrate.e2.dispatch-policy-patch.v1", "patch":expected_patch,
    });
    hash_subject.sort_all_objects();
    assert_eq!(
        *patch_hash,
        format!(
            "{:x}",
            sha2::Sha256::digest(serde_json::to_vec(&hash_subject).unwrap())
        )
    );
    let snapshot = validate_policy_snapshot_material(
        &expected,
        &prepared.exact_policy_snapshot.policy_snapshot_bytes,
        &plan.policy_and_admission_cap.current_policy_ref,
        &prepared.exact_policy_snapshot.policy_snapshot_hash,
        &plan.policy_and_admission_cap.current_policy.policy_revision,
    )
    .unwrap();
    let input = FreshSpawnReservationInputV1 {
        spawn_request: plan.spawn_request.clone(),
        caller_backend_id: plan.exact_authority.caller_descriptor.backend_id.clone(),
        parent_policy_ref: plan.policy_and_admission_cap.current_policy_ref.clone(),
        parent_policy_revision: plan
            .policy_and_admission_cap
            .current_policy
            .policy_revision
            .clone(),
        applied_patch: applied.clone(),
        policy_snapshot: snapshot.clone(),
        reason: patch.reason.clone(),
    };
    let retry_proof = reserve_ordinary_fresh_spawn(
        AuthenticatedOrdinaryFreshSpawnInputV1::authenticate(
            Arc::clone(&prepared.selection),
            input.clone(),
        )
        .unwrap(),
    )
    .unwrap()
    .authenticate_admission_plan(plan)
    .unwrap();
    assert!(retry_proof.permits_capability_narrowing());
    assert_eq!(
        retry_proof.retained_participant_id(),
        proof.retained_participant_id()
    );
    assert_eq!(retry_proof.bootstrap_run_id(), proof.bootstrap_run_id());
    let runtime = RetainedWorkerRuntime;
    let before = runtime
        .read_admission_record(
            authority,
            selected.original_session_id(),
            proof.retained_participant_id(),
        )
        .unwrap()
        .unwrap();
    let joined = runtime
        .reserve_admission_slot(authority, plan, Some(&retry_proof))
        .unwrap();
    assert!(joined.joined);
    assert_eq!(
        joined.record.retained_participant_id,
        before.retained_participant_id
    );
    assert_eq!(joined.record.bootstrap_run_id, before.bootstrap_run_id);
    assert_eq!(
        joined.record.canonical_spawn_fingerprint,
        before.canonical_spawn_fingerprint
    );
    let published =
        publish_fresh_spawn_commitment(authority, &retry_proof, &joined.record).unwrap();
    assert_eq!(published.record().applied_patch, applied);
    assert_eq!(
        published.record().parent_policy_ref,
        plan.policy_and_admission_cap.current_policy_ref
    );
    assert_eq!(
        published.record().parent_policy_revision,
        plan.policy_and_admission_cap.current_policy.policy_revision
    );
    assert_eq!(published.record().policy_snapshot_hash, snapshot.hash());
    assert_eq!(
        published.record().retained_worker_cap_link,
        Some(RetainedWorkerCapLinkV1::ThisCommitment)
    );
    assert_eq!(
        serde_json::to_value(published.commitment_ref()).unwrap(),
        serde_json::to_value(&prepared.e2_launch_activation.immutable_worker_cap_ref).unwrap()
    );
    let cap =
        authenticate_dispatch_policy_commitment(authority, &published.commitment_ref()).unwrap();
    assert_eq!(cap.policy_snapshot_bytes(), snapshot.bytes());
    assert_eq!(
        cap.subject(),
        &DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch {
            retained_participant_id: before.retained_participant_id.clone(),
            bootstrap_run_id: before.bootstrap_run_id.clone(),
        }
    );
    let reference = prepared
        .e2_launch_activation
        .reservation_ref
        .as_ref()
        .unwrap();
    let reference = DispatchPolicyCommitmentReservationRefV1 {
        authority_store_id: reference.authority_store_id.clone(),
        reservation_id: reference.reservation_id.clone(),
        reservation_hash: reference.reservation_hash.clone(),
    };
    let authenticated = authenticate_fresh_spawn_reservation(
        authority,
        &reference,
        &plan.spawn_request,
        plan,
        &snapshot,
    )
    .unwrap();
    assert!(authenticated.permits_capability_narrowing());
    let registry_path = fixture
        ._temp
        .path()
        .join("accepted/authority-v1/retained-worker-admission-v1/registry-v1.json");
    let registry_before = std::fs::read(&registry_path).unwrap();
    let mut wrong_request = plan.spawn_request.clone();
    wrong_request
        .payload
        .prompt
        .push_str("_different_complete_request");
    assert!(authenticate_fresh_spawn_reservation(
        authority,
        &reference,
        &wrong_request,
        plan,
        &snapshot
    )
    .is_err());
    let mut wrong_reference = reference.clone();
    wrong_reference.reservation_hash = "0".repeat(64);
    assert!(authenticate_fresh_spawn_reservation(
        authority,
        &wrong_reference,
        &plan.spawn_request,
        plan,
        &snapshot
    )
    .is_err());
    let unchanged_snapshot = validate_policy_snapshot_material(
        &parent.snapshot,
        &serde_json::to_vec(&parent.snapshot).unwrap(),
        &plan.policy_and_admission_cap.current_policy_ref,
        &parent.snapshot_hash,
        &plan.policy_and_admission_cap.current_policy.policy_revision,
    )
    .unwrap();
    assert!(authenticate_fresh_spawn_reservation(
        authority,
        &reference,
        &plan.spawn_request,
        plan,
        &unchanged_snapshot
    )
    .is_err());
    let mut wrong_plan = plan.clone();
    wrong_plan
        .policy_and_admission_cap
        .current_policy_ref
        .ref_id = format!("ao_{}", "0".repeat(32));
    assert!(runtime
        .reserve_admission_slot(authority, &wrong_plan, Some(proof))
        .is_err());
    assert_eq!(std::fs::read(&registry_path).unwrap(), registry_before);

    let transport = crate::execution::routing::MemberDispatchTransportRequest {
        orchestration_session_id: prepared.request.orchestration_session_id.clone(),
        participant_id: proof.retained_participant_id().into(),
        orchestrator_participant_id: prepared.request.caller_participant_id.clone(),
        parent_participant_id: None,
        resumed_from_participant_id: None,
        backend_id: prepared.descriptor.backend_id.clone(),
        protocol: prepared.descriptor.protocol.clone(),
        run_id: proof.bootstrap_run_id().into(),
        world_id: world.world_id.clone(),
        world_generation: world.world_generation,
        initial_prompt: Some(plan.spawn_request.payload.prompt.clone()),
        backend_kind: transport_api_types::MemberRuntimeBackendKindV1::Codex,
        binary_path: prepared.descriptor.binary_path.display().to_string(),
        retained_worker_launch_authority: Some(prepared.launch_authority_proof.clone()),
        e2_launch_activation: Some(prepared.e2_launch_activation.clone()),
        config_projection: None,
        exact_policy_snapshot: Some(prepared.exact_policy_snapshot.clone()),
    };
    let (_, execute, _) = build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
        &prepared.selection,
        &transport,
        Path::new(selected.workspace_root()),
    )
    .unwrap();
    let _admission = prepared.selection.prepare_world_execute(&execute).unwrap();
    let mut wrong_execute = execute.clone();
    wrong_execute
        .member_dispatch
        .as_mut()
        .unwrap()
        .as_v1_mut()
        .unwrap()
        .e2_launch_activation
        .as_mut()
        .unwrap()
        .immutable_worker_cap_ref
        .exact_linkage_hash = "f".repeat(64);
    assert!(prepared
        .selection
        .prepare_world_execute(&wrong_execute)
        .is_err());
    assert!(prepare_ordinary_authority_bound_spawn_world_worker(
        Arc::clone(&prepared.selection),
        request
    )
    .is_err());

    // A new current policy decision must refuse the previously admitted effect.
    // The immutable attestation continues to permit recording cleanup truth.
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap();
    assert!(policy.contains("allow_capability_narrowing: true"));
    std::fs::write(
        &policy_path,
        policy.replace(
            "allow_capability_narrowing: true",
            "allow_capability_narrowing: false",
        ),
    )
    .unwrap();
    std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let current_false = AuthenticatedOrdinaryFreshSpawnInputV1::authenticate(
        Arc::clone(&prepared.selection),
        input,
    )
    .unwrap();
    assert!(reserve_ordinary_fresh_spawn(current_false)
        .unwrap()
        .authenticate_admission_plan(plan)
        .is_err());
    assert!(prepared.selection.prepare_world_execute(&execute).is_err());
    assert_eq!(std::fs::read(&registry_path).unwrap(), registry_before);
    let interrupted = runtime
        .mark_admission_interrupted(
            authority,
            plan,
            proof.retained_participant_id(),
            Some(proof),
            None,
            TimestampV1::parse(
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            )
            .unwrap(),
        )
        .unwrap();
    assert!(matches!(
        interrupted.state,
        RetainedWorkerAdmissionStateV1::InterruptedNonterminal { .. }
    ));
    assert!(!fixture
        ._temp
        .path()
        .join("workspace/native-observations.jsonl")
        .exists());
    assert!(!fixture
        ._temp
        .path()
        .join("gateway-observations.jsonl")
        .exists());
    std::fs::remove_dir_all(&fixture.component_cgroup).unwrap();
}

// One focused composition helper over the genuine existing configured fixture.
// It runs only after the service-owned SDK source has surfaced Registered and
// the real issuer has marked that exact source R0 Routable. No native Fork or
// provider call is made here; vendor history consumption remains separate proof.
fn assert_initial_fork_admission_joins(
    fixture: &Fixture,
    selected: &Arc<AuthenticatedOrdinarySelectionV1>,
    store: &crate::execution::agent_runtime::state_store::AgentRuntimeStateStore,
    source: &crate::execution::routing::MemberDispatchTransportRequest,
    source_execute: &transport_api_types::ExecuteRequest,
    source_admission: &crate::execution::orchestrator_world_dispatch::OrdinaryWorldExecuteAdmissionV1,
) -> transport_api_types::ExecuteRequest {
    use crate::execution::agent_runtime::dispatch_contract::*;
    use crate::execution::build_ordinary_agent_client_and_member_dispatch_request_for_cwd;
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_fork_world_worker_bootstrap;
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_component_initial_fork_source_join".into()),
        idempotency_key: Some("idem_component_initial_fork_source_join".into()),
        orchestration_session_id: Some(source.orchestration_session_id.clone()),
        caller_participant_id: Some(source.orchestrator_participant_id.clone()),
        action: WorldDispatchActionV1::ForkWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(source.participant_id.clone()),
        world_id: Some(source.world_id.clone()),
        world_generation: Some(source.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
            prompt: "component initial Fork authenticated source join".into(),
            fork_reason: None,
            fork_strategy: None,
        }),
    }
    .validate()
    .unwrap();
    let prepared =
        prepare_ordinary_fork_world_worker_bootstrap(Arc::clone(selected), store, request).unwrap();
    let committed = prepared.bootstrap.policy_commitment.as_ref().unwrap();
    let transport = crate::execution::routing::MemberDispatchTransportRequest {
        participant_id: committed.child_participant_id.clone(),
        run_id: committed.bootstrap_run_id.clone(),
        parent_participant_id: Some(source.participant_id.clone()),
        retained_worker_launch_authority: None,
        e2_launch_activation: Some(committed.e2_launch_activation.clone()),
        exact_policy_snapshot: Some(committed.exact_policy_snapshot.clone()),
        initial_prompt: Some("component initial Fork authenticated source join".into()),
        ..source.clone()
    };
    let (_, execute, _) = build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
        &prepared.selection,
        &transport,
        Path::new(selected.workspace_root()),
    )
    .unwrap();
    let child = prepared.selection.prepare_world_execute(&execute).unwrap();
    let authenticated_source = child
        .validate_initial_fork_source(source_admission)
        .unwrap();
    assert_eq!(
        Some(&authenticated_source),
        source.e2_launch_activation.as_ref()
    );
    assert_eq!(
        authenticated_source.retained_participant_id,
        source.participant_id
    );
    // This is real current admission, not a private structural decoder exception.
    for case in [
        "source-cap",
        "issuer",
        "root",
        "principal-caller",
        "session",
        "world",
        "world-generation",
        "runtime",
        "backend",
        "selection",
        "native-selection",
        "workspace",
        "source-participant",
    ] {
        let mut wrong = execute.clone();
        match case {
            "issuer" => {
                wrong
                    .ordinary_start_selection
                    .as_mut()
                    .unwrap()
                    .original_start_issuer_request_id = format!("req_{}", "d".repeat(64))
            }
            "root" => {
                wrong
                    .ordinary_start_selection
                    .as_mut()
                    .unwrap()
                    .authority_store_id = "different-ordinary-store".into()
            }
            "selection" => {
                wrong
                    .ordinary_start_selection
                    .as_mut()
                    .unwrap()
                    .requested_selection_sha256 = "d".repeat(64)
            }
            "native-selection" => {
                wrong
                    .ordinary_start_selection
                    .as_mut()
                    .unwrap()
                    .native_selection_sha256 = "d".repeat(64)
            }
            "workspace" => wrong.cwd = Some(fixture.home.display().to_string()),
            other => {
                let member = wrong.member_dispatch.as_mut().unwrap().as_v1_mut().unwrap();
                let activation = member.e2_launch_activation.as_mut().unwrap();
                match other {
                    "source-cap" => {
                        activation.commitment_ref.exact_linkage_hash = "d".repeat(64);
                        activation.immutable_worker_cap_ref = activation.commitment_ref.clone();
                        activation.activation_id = format!("e2a_{}", "d".repeat(32));
                    }
                    "principal-caller" => {
                        member.orchestrator_participant_id = "other-caller".into();
                        activation.caller_participant_id = "other-caller".into();
                    }
                    "session" => {
                        member.orchestration_session_id = "other-session".into();
                        activation.orchestration_session_id = "other-session".into();
                    }
                    "world" => {
                        member.world_id = "other-world".into();
                        activation.target_world.world_id = "other-world".into();
                    }
                    "world-generation" => {
                        member.world_generation += 1;
                        activation.target_world.world_generation = member.world_generation;
                    }
                    "runtime" => member.resolved_runtime.binary_path = "/bin/false".into(),
                    "backend" => {
                        member.backend_id = "cli:other-world".into();
                        activation.target_backend_id = "cli:other-world".into();
                    }
                    "source-participant" => {
                        member.parent_participant_id = Some("other-source".into());
                        activation.source_participant_id = Some("other-source".into());
                    }
                    _ => unreachable!(),
                }
            }
        }
        wrong.validate().unwrap_or_else(|error| {
            panic!("{case} must reach authentic admission, not parser refusal: {error}")
        });
        assert!(
            prepared.selection.prepare_world_execute(&wrong).is_err(),
            "substituted {case}"
        );
        child
            .validate_initial_fork_source(source_admission)
            .unwrap();
    }
    assert_eq!(
        child
            .recheck_initial_fork_source_request_for_test(source_admission, source_execute)
            .unwrap(),
        authenticated_source
    );
    for case in ["original-spawn-issuer", "source-cap", "source-store"] {
        let mut wrong = source_execute.clone();
        let member = wrong.member_dispatch.as_mut().unwrap().as_v1_mut().unwrap();
        match case {
            "original-spawn-issuer" => {
                member
                    .retained_worker_launch_authority
                    .as_mut()
                    .unwrap()
                    .issuer_request_id = "substituted-spawn-issuer".into()
            }
            "source-store" => {
                member
                    .retained_worker_launch_authority
                    .as_mut()
                    .unwrap()
                    .authority_store_id = "substituted-source-store".into()
            }
            "source-cap" => {
                let activation = member.e2_launch_activation.as_mut().unwrap();
                activation.commitment_ref.exact_linkage_hash = "e".repeat(64);
                activation.immutable_worker_cap_ref = activation.commitment_ref.clone();
                activation.activation_id = format!("e2a_{}", "e".repeat(32));
            }
            _ => unreachable!(),
        }
        wrong.validate().unwrap_or_else(|error| {
            panic!("source {case} must reach authentic admission: {error}")
        });
        assert!(
            child
                .recheck_initial_fork_source_request_for_test(source_admission, &wrong)
                .is_err(),
            "substituted genuine source request: {case}"
        );
        child
            .validate_initial_fork_source(source_admission)
            .unwrap();
    }
    // A principal substitution is structurally canonical but cannot replace the
    // original issuer/context. Restore bytes before inspecting any outcome.
    let context_path = fixture
        .home
        .join("run/agent-hub/ordinary-start-contexts")
        .join(format!("{}.json", fixture.context.intent_id));
    let context = std::fs::read(&context_path).unwrap();
    let mut forged: serde_json::Value = serde_json::from_slice(&context).unwrap();
    let principal = &mut forged["accounts"]["native"]["principal"];
    let uid = principal["uid"]
        .as_u64()
        .expect("genuine Unix principal UID");
    principal["uid"] = (uid + 1).into();
    let _: NativeLaunchContextV1 = serde_json::from_value(forged.clone()).unwrap();
    std::fs::write(&context_path, canonical_json::to_vec(&forged).unwrap()).unwrap();
    let principal_result = child.validate_initial_fork_source(source_admission);
    std::fs::write(&context_path, &context).unwrap();
    assert!(
        principal_result.is_err(),
        "substituted original Unix principal"
    );
    child
        .validate_initial_fork_source(source_admission)
        .unwrap();

    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read(&policy_path).unwrap();
    let mut revoked: serde_yaml::Value = serde_yaml::from_slice(&policy).unwrap();
    revoked["agents"]["world_dispatch"]["allowed_actions"] =
        serde_yaml::Value::Sequence(Vec::new());
    std::fs::write(&policy_path, serde_yaml::to_string(&revoked).unwrap()).unwrap();
    let policy_result = child.validate_initial_fork_source(source_admission);
    std::fs::write(&policy_path, &policy).unwrap();
    assert!(
        policy_result.is_err(),
        "current policy revocation must close held Fork admission"
    );
    child
        .validate_initial_fork_source(source_admission)
        .unwrap();

    let account_key = SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID;
    let account = std::env::var_os(account_key).unwrap();
    std::env::set_var(account_key, "substituted-component-account");
    let account_result = child.validate_initial_fork_source(source_admission);
    std::env::set_var(account_key, account);
    assert!(
        account_result.is_err(),
        "current selected account substitution"
    );
    child
        .validate_initial_fork_source(source_admission)
        .unwrap();

    let moved = fixture.home.with_extension("held-fork-origin");
    std::fs::rename(&fixture.home, &moved).unwrap();
    std::fs::create_dir(&fixture.home).unwrap();
    std::fs::set_permissions(&fixture.home, std::fs::Permissions::from_mode(0o700)).unwrap();
    let root_result = child.validate_initial_fork_source(source_admission);
    std::fs::remove_dir(&fixture.home).unwrap();
    std::fs::rename(moved, &fixture.home).unwrap();
    assert!(
        root_result.is_err(),
        "copied accepted-home pathname cannot replace held root"
    );
    child
        .validate_initial_fork_source(source_admission)
        .unwrap();
    eprintln!("initial Fork authentic source join: genuine original source/R0 and current configured selection; substituted cap/issuer/root/principal/session/world/runtime/backend/selection/source and policy/account refused; component only");
    execute
}

/// Owner/state-machine test with synthetic service observations. The original
/// Start, selection, E2 reservation and R0 graph are genuine publishers; this
/// does not qualify native process/gateway custody, kernel or installed proof.
#[test]
fn test_native_bootstrap_completion_preserves_r0_and_refuses_resurrection() {
    use crate::execution::agent_runtime::{dispatch_contract::*, retained_worker_runtime::*};
    use crate::execution::orchestrator_world_dispatch::{
        ordinary_start_selection_carrier, prepare_ordinary_authority_bound_spawn_world_worker,
    };
    use substrate_common::agent_events::{
        AgentEvent, MessageEventKind, RuntimeEventIdentityV1, RuntimeFrameIdentityV1,
        RuntimeTerminalIdentityV1,
    };
    use transport_api_types::{
        NativeProcessTerminationV1, NativeSettlementAcknowledgmentV1, OrdinaryGatewayDisposalV1,
        OrdinaryGatewayRuntimeIdentityV1, RetainedBootstrapCompletionV1,
        RetainedBootstrapObservationV1,
    };
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let selected = Arc::new(
        OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(Arc::clone(
            &fixture.configured,
        ))
        .unwrap()
        .authenticate(&fixture.context.orchestration_session_id)
        .unwrap(),
    );
    let world = fixture.context.original_world_binding.as_ref().unwrap();
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_native_bootstrap_owner".into()),
        idempotency_key: Some("idem_native_bootstrap_owner".into()),
        orchestration_session_id: Some(selected.original_session_id().into()),
        caller_participant_id: Some(fixture.context.original_participant_id.clone()),
        action: WorldDispatchActionV1::SpawnWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(selected.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(world.world_id.clone()),
        world_generation: Some(world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
            prompt: "bootstrap owner unit".into(),
        }),
    }
    .validate()
    .unwrap();
    let prepared = prepare_ordinary_authority_bound_spawn_world_worker(selected, request).unwrap();
    let authority = &prepared.authority;
    let plan = &prepared.admission_plan;
    let proof = &prepared.dispatch_policy_reservation_proof;
    let participant = proof.retained_participant_id();
    let runtime = RetainedWorkerRuntime;
    let registry_path = fixture
        .home
        .join("authority-v1/retained-worker-admission-v1/registry-v1.json");
    let absent_registry = std::fs::read(&registry_path).unwrap();
    assert!(
        serde_json::from_slice::<serde_json::Value>(&absent_registry)
            .unwrap()
            .get("native_lifecycle_v1")
            .is_none()
    );
    runtime
        .claim_admission_transport(authority, plan, participant, Some(proof))
        .unwrap();
    let start = RuntimeFrameIdentityV1 {
        schema_version: 1,
        stream_id: "rts_native_bootstrap_owner".into(),
        frame_sequence: 1,
    };
    let span = "native-bootstrap-owner-span";
    runtime
        .observe_admission_transport_start(authority, plan, participant, Some(proof), &start, span)
        .unwrap();
    let registered_frame = RuntimeFrameIdentityV1 {
        frame_sequence: 2,
        ..start.clone()
    };
    let mut registered = AgentEvent::message(
        "ambient-telemetry",
        prepared.selection.original_session_id(),
        proof.bootstrap_run_id(),
        MessageEventKind::Registered,
        "registered",
    );
    registered.participant_id = Some(participant.into());
    registered.backend_id = Some(plan.spawn_request.target_backend_id.clone());
    registered.role = Some("member".into());
    registered.world_id = Some(world.world_id.clone());
    registered.world_generation = Some(world.world_generation);
    registered.span_id = Some(span.into());
    registered.event_identity = Some(RuntimeEventIdentityV1 {
        event_id: "bootstrap-registered-event".into(),
        event_sequence: 1,
    });
    registered.data = serde_json::json!({"schema":"agent_api.session.handle.v1", "session":{"id":"logical-unit-session"}, "internal":{"uaa_session_id":"  native-unit-session  "}});
    let now = || {
        TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            .unwrap()
    };
    let routable = runtime
        .mark_admission_routable(
            authority,
            plan,
            participant,
            Some(proof),
            &registered_frame,
            &registered,
            now(),
        )
        .unwrap();
    let event_identity = RuntimeEventIdentityV1 {
        event_id: "bootstrap-terminal-event".into(),
        event_sequence: 3,
    };
    let report = RetainedBootstrapCompletionV1::from_observation(RetainedBootstrapObservationV1 {
        source_start: ordinary_start_selection_carrier(&prepared.selection).unwrap(),
        runtime: OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id: plan.spawn_request.orchestration_session_id.clone(),
            caller_participant_id: plan.spawn_request.caller_participant_id.clone(),
            participant_id: participant.into(),
            backend_id: plan.spawn_request.target_backend_id.clone(),
            run_id: proof.bootstrap_run_id().into(),
            world_id: world.world_id.clone(),
            world_generation: world.world_generation,
        },
        activation: prepared.e2_launch_activation.clone(),
        native_session_id: "native-unit-session".into(),
        span_id: span.into(),
        frame_identity: RuntimeFrameIdentityV1 {
            frame_sequence: 4,
            ..start.clone()
        },
        terminal_identity: RuntimeTerminalIdentityV1::from(&event_identity),
        event_identity,
        native_settlement: NativeSettlementAcknowledgmentV1::SdkChecked,
        native_termination: NativeProcessTerminationV1::Exited { exit_code: 0 },
        gateway_disposal: OrdinaryGatewayDisposalV1::ProcessWaited {
            runtime_id: "synthetic-unit-gateway".into(),
            pid: 12345,
            termination: NativeProcessTerminationV1::Signaled { signal: 15 },
        },
    })
    .unwrap();
    let physical_snapshot = || {
        let mut snapshot = BTreeMap::new();
        let mut pending = vec![fixture.home.clone()];
        while let Some(path) = pending.pop() {
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            assert!(metadata.is_file() || metadata.is_dir());
            let bytes = metadata.is_file().then(|| std::fs::read(&path).unwrap());
            snapshot.insert(
                path.clone(),
                (
                    metadata.dev(),
                    metadata.ino(),
                    metadata.uid(),
                    metadata.gid(),
                    metadata.mode(),
                    metadata.nlink(),
                    metadata.mtime(),
                    metadata.mtime_nsec(),
                    metadata.ctime(),
                    metadata.ctime_nsec(),
                    bytes,
                ),
            );
            if metadata.is_dir() {
                pending.extend(
                    std::fs::read_dir(&path)
                        .unwrap()
                        .map(|entry| entry.unwrap().path()),
                );
            }
        }
        snapshot
    };
    let before = physical_snapshot();
    let mut wrong = report.observation().clone();
    wrong.source_start.context_sha256 = "d".repeat(64);
    let wrong = RetainedBootstrapCompletionV1::from_observation(wrong).unwrap();
    assert!(runtime
        .record_native_bootstrap_completion(
            authority,
            &prepared.selection,
            &registered_frame,
            &registered,
            &wrong
        )
        .is_err());
    assert_eq!(before, physical_snapshot());
    let recorded = runtime
        .record_native_bootstrap_completion(
            authority,
            &prepared.selection,
            &registered_frame,
            &registered,
            &report,
        )
        .unwrap();
    assert_eq!(
        recorded, routable,
        "bootstrap completion must preserve original R0 bytes/state"
    );
    let published = physical_snapshot();
    runtime
        .record_native_bootstrap_completion(
            authority,
            &prepared.selection,
            &registered_frame,
            &registered,
            &report,
        )
        .unwrap();
    assert_eq!(
        published,
        physical_snapshot(),
        "exact retry must not rewrite"
    );
    let mut conflicting_registered = registered.clone();
    conflicting_registered.data["changed_observation"] = true.into();
    assert!(runtime
        .record_native_bootstrap_completion(
            authority,
            &prepared.selection,
            &registered_frame,
            &conflicting_registered,
            &report
        )
        .is_err());
    assert_eq!(
        published,
        physical_snapshot(),
        "changed Registered retry must not rewrite history"
    );
    let registry_bytes = std::fs::read(&registry_path).unwrap();
    let mut registry: serde_json::Value = serde_json::from_slice(&registry_bytes).unwrap();
    registry["native_lifecycle_v1"]["body"]["revision"] = 99.into();
    let corrupt = super::super::host_session_authority::canonical_json::to_vec(&registry).unwrap();
    std::fs::write(&registry_path, &corrupt).unwrap();
    let corrupted = physical_snapshot();
    assert!(runtime
        .read_existing_admission_record(
            authority,
            prepared.selection.original_session_id(),
            participant
        )
        .is_err());
    assert_eq!(corrupted, physical_snapshot());
    std::fs::write(&registry_path, &registry_bytes).unwrap();
    let observed = report.observation();
    let terminal = runtime
        .mark_admission_terminal_for_transport(
            authority,
            plan,
            participant,
            Some(proof),
            &observed.frame_identity,
            &observed.event_identity,
            &observed.terminal_identity,
            span,
            0,
            now(),
        )
        .unwrap();
    assert!(matches!(
        terminal.state,
        RetainedWorkerAdmissionStateV1::Terminal { .. }
    ));
    assert_eq!(
        runtime
            .record_native_bootstrap_completion(
                authority,
                &prepared.selection,
                &registered_frame,
                &registered,
                &report
            )
            .unwrap(),
        terminal
    );
    // Simulate the exact old V1 representation of a previously terminal worker.
    // No exit-zero/old-Terminal inference may manufacture a new completion.
    let mut old: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&registry_path).unwrap()).unwrap();
    old.as_object_mut().unwrap().remove("native_lifecycle_v1");
    std::fs::write(
        &registry_path,
        super::super::host_session_authority::canonical_json::to_vec(&old).unwrap(),
    )
    .unwrap();
    let old_terminal = physical_snapshot();
    assert!(runtime
        .record_native_bootstrap_completion(
            authority,
            &prepared.selection,
            &registered_frame,
            &registered,
            &report
        )
        .is_err());
    assert_eq!(old_terminal, physical_snapshot());
    std::fs::remove_dir_all(&fixture.component_cgroup).unwrap();
}

// Genuine Start/E2/R0 publishers with synthetic service observations only.
// This fixture confers no native or kernel-custody qualification.
fn completed_spawn_for_fork_claim_test() -> (
    Fixture,
    crate::execution::orchestrator_world_dispatch::PreparedOrdinarySpawnWorldWorkerBootstrap,
    transport_api_types::RetainedBootstrapCompletionV1,
) {
    completed_spawn_from_fixture_for_fork_claim_test(fixture_for_harness_with_narrowing(
        true, true, false, true,
    ))
}

fn completed_spawn_from_fixture_for_fork_claim_test(
    fixture: Fixture,
) -> (
    Fixture,
    crate::execution::orchestrator_world_dispatch::PreparedOrdinarySpawnWorldWorkerBootstrap,
    transport_api_types::RetainedBootstrapCompletionV1,
) {
    completed_spawn_from_fixture_with_before_native_hook(fixture, |_| {})
}

fn completed_spawn_from_fixture_with_before_native_hook(
    fixture: Fixture,
    before_native: impl FnOnce(
        &crate::execution::orchestrator_world_dispatch::PreparedOrdinarySpawnWorldWorkerBootstrap,
    ),
) -> (
    Fixture,
    crate::execution::orchestrator_world_dispatch::PreparedOrdinarySpawnWorldWorkerBootstrap,
    transport_api_types::RetainedBootstrapCompletionV1,
) {
    use crate::execution::agent_runtime::{dispatch_contract::*, retained_worker_runtime::*};
    use crate::execution::orchestrator_world_dispatch::{
        ordinary_start_selection_carrier, prepare_ordinary_authority_bound_spawn_world_worker,
    };
    use substrate_common::agent_events::{
        AgentEvent, MessageEventKind, RuntimeEventIdentityV1, RuntimeFrameIdentityV1,
        RuntimeTerminalIdentityV1,
    };
    use transport_api_types::{
        NativeProcessTerminationV1, NativeSettlementAcknowledgmentV1, OrdinaryGatewayDisposalV1,
        OrdinaryGatewayRuntimeIdentityV1, RetainedBootstrapCompletionV1,
        RetainedBootstrapObservationV1,
    };
    let selected = Arc::new(
        OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(Arc::clone(
            &fixture.configured,
        ))
        .unwrap()
        .authenticate(&fixture.context.orchestration_session_id)
        .unwrap(),
    );
    let world = fixture.context.original_world_binding.as_ref().unwrap();
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_native_bootstrap_owner".into()),
        idempotency_key: Some("idem_native_bootstrap_owner".into()),
        orchestration_session_id: Some(selected.original_session_id().into()),
        caller_participant_id: Some(fixture.context.original_participant_id.clone()),
        action: WorldDispatchActionV1::SpawnWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(selected.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(world.world_id.clone()),
        world_generation: Some(world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
            prompt: "bootstrap owner unit".into(),
        }),
    }
    .validate()
    .unwrap();
    let prepared = prepare_ordinary_authority_bound_spawn_world_worker(selected, request).unwrap();
    let authority = &prepared.authority;
    let plan = &prepared.admission_plan;
    let proof = &prepared.dispatch_policy_reservation_proof;
    let participant = proof.retained_participant_id();
    let runtime = RetainedWorkerRuntime;
    let registry_path = fixture
        .home
        .join("authority-v1/retained-worker-admission-v1/registry-v1.json");
    let absent_registry = std::fs::read(&registry_path).unwrap();
    assert!(
        serde_json::from_slice::<serde_json::Value>(&absent_registry)
            .unwrap()
            .get("native_lifecycle_v1")
            .is_none()
    );
    runtime
        .claim_admission_transport(authority, plan, participant, Some(proof))
        .unwrap();
    before_native(&prepared);
    let start = RuntimeFrameIdentityV1 {
        schema_version: 1,
        stream_id: "rts_native_bootstrap_owner".into(),
        frame_sequence: 1,
    };
    let span = "native-bootstrap-owner-span";
    runtime
        .observe_admission_transport_start(authority, plan, participant, Some(proof), &start, span)
        .unwrap();
    let registered_frame = RuntimeFrameIdentityV1 {
        frame_sequence: 2,
        ..start.clone()
    };
    let mut registered = AgentEvent::message(
        "ambient-telemetry",
        prepared.selection.original_session_id(),
        proof.bootstrap_run_id(),
        MessageEventKind::Registered,
        "registered",
    );
    registered.participant_id = Some(participant.into());
    registered.backend_id = Some(plan.spawn_request.target_backend_id.clone());
    registered.role = Some("member".into());
    registered.world_id = Some(world.world_id.clone());
    registered.world_generation = Some(world.world_generation);
    registered.span_id = Some(span.into());
    registered.event_identity = Some(RuntimeEventIdentityV1 {
        event_id: "bootstrap-registered-event".into(),
        event_sequence: 1,
    });
    registered.data = serde_json::json!({"schema":"agent_api.session.handle.v1", "session":{"id":"logical-unit-session"}, "internal":{"uaa_session_id":"  native-unit-session  "}});
    let now = || {
        TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            .unwrap()
    };
    let routable = runtime
        .mark_admission_routable(
            authority,
            plan,
            participant,
            Some(proof),
            &registered_frame,
            &registered,
            now(),
        )
        .unwrap();
    let event_identity = RuntimeEventIdentityV1 {
        event_id: "bootstrap-terminal-event".into(),
        event_sequence: 3,
    };
    let report = RetainedBootstrapCompletionV1::from_observation(RetainedBootstrapObservationV1 {
        source_start: ordinary_start_selection_carrier(&prepared.selection).unwrap(),
        runtime: OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id: plan.spawn_request.orchestration_session_id.clone(),
            caller_participant_id: plan.spawn_request.caller_participant_id.clone(),
            participant_id: participant.into(),
            backend_id: plan.spawn_request.target_backend_id.clone(),
            run_id: proof.bootstrap_run_id().into(),
            world_id: world.world_id.clone(),
            world_generation: world.world_generation,
        },
        activation: prepared.e2_launch_activation.clone(),
        native_session_id: "native-unit-session".into(),
        span_id: span.into(),
        frame_identity: RuntimeFrameIdentityV1 {
            frame_sequence: 4,
            ..start.clone()
        },
        terminal_identity: RuntimeTerminalIdentityV1::from(&event_identity),
        event_identity,
        native_settlement: NativeSettlementAcknowledgmentV1::SdkChecked,
        native_termination: NativeProcessTerminationV1::Exited { exit_code: 0 },
        gateway_disposal: OrdinaryGatewayDisposalV1::ProcessWaited {
            runtime_id: "synthetic-unit-gateway".into(),
            pid: 12345,
            termination: NativeProcessTerminationV1::Signaled { signal: 15 },
        },
    })
    .unwrap();

    let recorded = runtime
        .record_native_bootstrap_completion(
            authority,
            &prepared.selection,
            &registered_frame,
            &registered,
            &report,
        )
        .unwrap();
    assert_eq!(recorded, routable);
    (fixture, prepared, report)
}

#[test]
fn test_ordinary_fork_claim_is_durable_and_counts_full_population() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*, retained_worker_runtime::RetainedWorkerRuntime,
        state_store::AgentRuntimeStateStore,
    };
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_fork_world_worker_bootstrap;
    let (fixture, source, _) = completed_spawn_for_fork_claim_test();
    let store = AgentRuntimeStateStore::new().unwrap();
    let mut child_claims = Vec::new();
    for index in 0..4 {
        let request = WorldDispatchRequestV1 {
            request_id: Some(format!("req_durable_fork_{index}")),
            idempotency_key: Some(format!("idem_durable_fork_{index}")),
            orchestration_session_id: Some(source.selection.original_session_id().into()),
            caller_participant_id: Some(source.selection.current_participant_id().into()),
            action: WorldDispatchActionV1::ForkWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(source.descriptor.backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(
                source.e2_launch_activation.retained_participant_id.clone(),
            ),
            world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
            world_generation: Some(source.e2_launch_activation.target_world.world_generation),
            dispatch_policy_narrowing: None,
            payload: WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
                prompt: "never sent by source-owner test".into(),
                fork_reason: None,
                fork_strategy: None,
            }),
        }
        .validate()
        .unwrap();
        let before_preparation = component_tree_snapshot(&fixture.home);
        let preparation = prepare_ordinary_fork_world_worker_bootstrap(
            Arc::clone(&source.selection),
            &store,
            request,
        );
        if index == 3 {
            assert!(preparation
                .err()
                .expect("full durable population must refuse before E2 publication")
                .to_string()
                .contains("retained_worker_cap_reached"));
            assert_eq!(before_preparation, component_tree_snapshot(&fixture.home));
            continue;
        }
        let mut prepared = preparation.unwrap();
        let native_issuance = prepared
            .bootstrap
            .policy_commitment
            .as_mut()
            .unwrap()
            .native_issuance
            .take();
        let activation = &prepared
            .bootstrap
            .policy_commitment
            .as_ref()
            .unwrap()
            .e2_launch_activation;
        let claim = RetainedWorkerRuntime.claim_ordinary_fork_native(
            &source.selection,
            activation,
            &source.admission_plan.descriptor_and_runtime_plan.descriptor,
            native_issuance,
        );
        let claim = claim.unwrap();
        assert!(claim.new_native_claim);
        if index == 0 {
            // The E2 issuance survives loss of a pending admission envelope.
            // Missing ownership may not mint another native-send permission.
            let path = fixture
                .home
                .join("authority-v1/retained-worker-admission-v1/registry-v1.json");
            let original = std::fs::read(&path).unwrap();
            let mut missing: serde_json::Value = serde_json::from_slice(&original).unwrap();
            missing.as_object_mut().unwrap().remove("ordinary_forks_v1");
            std::fs::write(
                &path,
                super::super::host_session_authority::canonical_json::to_vec(&missing).unwrap(),
            )
            .unwrap();
            let lost = RetainedWorkerRuntime.claim_ordinary_fork_native(
                &source.selection,
                activation,
                &source.admission_plan.descriptor_and_runtime_plan.descriptor,
                None,
            );
            std::fs::write(&path, original).unwrap();
            assert!(
                lost.is_err(),
                "missing pending Fork ownership cannot issue another native-send claim"
            );
        }
        assert!(!child_claims.contains(&claim.record.native_claim_id));
        child_claims.push(claim.record.native_claim_id.clone());
        let before = component_tree_snapshot(&fixture.home);
        let joined = RetainedWorkerRuntime
            .claim_ordinary_fork_native(
                &source.selection,
                activation,
                &source.admission_plan.descriptor_and_runtime_plan.descriptor,
                None,
            )
            .unwrap();
        assert!(
            !joined.new_native_claim,
            "exact retry must not acquire a second native-send permit"
        );
        assert_eq!(claim.record.native_claim_id, joined.record.native_claim_id);
        assert_eq!(before, component_tree_snapshot(&fixture.home));
        assert!(
            RetainedWorkerRuntime
                .read_existing_admission_record(
                    &source.authority,
                    source.selection.original_session_id(),
                    &source.e2_launch_activation.retained_participant_id
                )
                .unwrap()
                .is_some(),
            "Fork claim must preserve original Spawn validation"
        );
    }
    assert_eq!(
        child_claims.len(),
        3,
        "one Spawn plus three unresolved Fork claims fills the durable cap"
    );
    let registry_path = fixture
        .home
        .join("authority-v1/retained-worker-admission-v1/registry-v1.json");
    let bytes = std::fs::read(&registry_path).unwrap();
    let mut corrupt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    corrupt["ordinary_forks_v1"]["body"]["revision"] = 99.into();
    std::fs::write(
        &registry_path,
        super::super::host_session_authority::canonical_json::to_vec(&corrupt).unwrap(),
    )
    .unwrap();
    let before = component_tree_snapshot(&fixture.home);
    assert!(RetainedWorkerRuntime
        .read_existing_admission_record(
            &source.authority,
            source.selection.original_session_id(),
            &source.e2_launch_activation.retained_participant_id
        )
        .is_err());
    assert_eq!(before, component_tree_snapshot(&fixture.home));
    std::fs::write(&registry_path, bytes).unwrap();
}

#[test]
fn test_ordinary_fork_completion_adopts_actual_native_r0_and_exact_retry() {
    use super::super::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1;
    use super::super::retained_worker_runtime::{
        set_ordinary_fork_r0_crash_point_for_test, OrdinaryForkR0CrashPointV1,
    };
    use crate::execution::agent_runtime::{
        dispatch_contract::*,
        retained_worker_runtime::RetainedWorkerRuntime,
        state_store::{resolve_canonical_retained_world_dispatch_target, AgentRuntimeStateStore},
    };
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_fork_world_worker_bootstrap;
    use substrate_common::agent_events::{
        AgentEvent, MessageEventKind, RuntimeEventIdentityV1, RuntimeFrameIdentityV1,
        RuntimeTerminalIdentityV1,
    };
    use transport_api_types::{
        OrdinaryForkBootstrapProvenanceV1, OrdinaryGatewayDisposalV1,
        OrdinaryGatewayRuntimeIdentityV1, RetainedForkBootstrapCompletionV1,
    };
    for crash_point in [
        None,
        Some(OrdinaryForkR0CrashPointV1::AfterReservation),
        Some(OrdinaryForkR0CrashPointV1::AfterApplication),
    ] {
        let (fixture, source, source_report) = completed_spawn_for_fork_claim_test();
        fixture._env.install_home(&fixture.home);
        let store = AgentRuntimeStateStore::new().unwrap();
        let request = WorldDispatchRequestV1 {
            request_id: Some("req_fork_r0_owner".into()),
            idempotency_key: Some("idem_fork_r0_owner".into()),
            orchestration_session_id: Some(source.selection.original_session_id().into()),
            caller_participant_id: Some(source.selection.current_participant_id().into()),
            action: WorldDispatchActionV1::ForkWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(source.descriptor.backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(
                source.e2_launch_activation.retained_participant_id.clone(),
            ),
            world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
            world_generation: Some(source.e2_launch_activation.target_world.world_generation),
            dispatch_policy_narrowing: None,
            payload: WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
                prompt: "synthetic completion-owner observation; zero native sends".into(),
                fork_reason: None,
                fork_strategy: None,
            }),
        }
        .validate()
        .unwrap();
        let mut prepared = prepare_ordinary_fork_world_worker_bootstrap(
            Arc::clone(&source.selection),
            &store,
            request,
        )
        .unwrap();
        let native_issuance = prepared
            .bootstrap
            .policy_commitment
            .as_mut()
            .unwrap()
            .native_issuance
            .take();
        let activation = &prepared
            .bootstrap
            .policy_commitment
            .as_ref()
            .unwrap()
            .e2_launch_activation;
        let runtime = RetainedWorkerRuntime;
        let claim = runtime
            .claim_ordinary_fork_native(
                &source.selection,
                activation,
                &source.admission_plan.descriptor_and_runtime_plan.descriptor,
                native_issuance,
            )
            .unwrap();
        assert!(claim.new_native_claim);
        if crash_point.is_none() {
            let transport = crate::execution::routing::MemberDispatchTransportRequest {
                orchestration_session_id: activation.orchestration_session_id.clone(),
                participant_id: activation.retained_participant_id.clone(),
                orchestrator_participant_id: activation.caller_participant_id.clone(),
                parent_participant_id: activation.source_participant_id.clone(),
                resumed_from_participant_id: None,
                backend_id: activation.target_backend_id.clone(),
                protocol: source.descriptor.protocol.clone(),
                run_id: activation.bootstrap_run_id.clone(),
                world_id: activation.target_world.world_id.clone(),
                world_generation: activation.target_world.world_generation,
                initial_prompt: Some(
                    "synthetic completion-owner observation; zero native sends".into(),
                ),
                backend_kind: transport_api_types::MemberRuntimeBackendKindV1::Codex,
                binary_path: source.descriptor.binary_path.display().to_string(),
                retained_worker_launch_authority: None,
                e2_launch_activation: Some(activation.clone()),
                config_projection: None,
                exact_policy_snapshot: Some(
                    prepared
                        .bootstrap
                        .policy_commitment
                        .as_ref()
                        .unwrap()
                        .exact_policy_snapshot
                        .clone(),
                ),
            };
            let (_, execute, _) =
                crate::execution::build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
                    &prepared.selection,
                    &transport,
                    Path::new(prepared.selection.workspace_root()),
                )
                .unwrap();
            let admitted = prepared.selection.prepare_world_execute(&execute).unwrap();
            admitted
                .claim_initial_bootstrap_receiver("spn_fork_receiver_once")
                .unwrap();
            let snapshot = component_tree_snapshot(&fixture.home);
            assert!(admitted
                .claim_initial_bootstrap_receiver("spn_fork_receiver_replay")
                .is_err());
            assert_eq!(snapshot, component_tree_snapshot(&fixture.home));
        }
        let current = source
            .authority
            .resolve_current_exact(source.selection.original_session_id(), None)
            .unwrap();
        assert!(
            resolve_canonical_retained_world_dispatch_target(
                &source.authority,
                &current,
                &activation.retained_participant_id,
                &activation.target_backend_id
            )
            .is_err(),
            "claimed is not R0 routable"
        );
        let mut observed = source_report.observation().clone();
        observed.activation = activation.clone();
        observed.runtime = OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id: activation.orchestration_session_id.clone(),
            caller_participant_id: activation.caller_participant_id.clone(),
            participant_id: activation.retained_participant_id.clone(),
            backend_id: activation.target_backend_id.clone(),
            run_id: activation.bootstrap_run_id.clone(),
            world_id: activation.target_world.world_id.clone(),
            world_generation: activation.target_world.world_generation,
        };
        observed.native_session_id = "actual-fork-owner-native-session".into();
        observed.span_id = "fork-owner-native-span".into();
        observed.frame_identity = RuntimeFrameIdentityV1 {
            schema_version: 1,
            stream_id: "rts_fork_owner".into(),
            frame_sequence: 4,
        };
        observed.event_identity = RuntimeEventIdentityV1 {
            event_id: "fork-owner-exit".into(),
            event_sequence: 3,
        };
        observed.terminal_identity = RuntimeTerminalIdentityV1::from(&observed.event_identity);
        observed.gateway_disposal = OrdinaryGatewayDisposalV1::ClosedBeforeIssue {
            runtime_id: "synthetic-positive-closed-owner".into(),
        };
        // Codec/source-only data: Fork protocol success is independent from its
        // physical MCP child wait. This does not qualify an actual native process.
        observed.native_termination =
            transport_api_types::NativeProcessTerminationV1::Signaled { signal: 9 };
        let report = RetainedForkBootstrapCompletionV1::from_observation(
            observed.clone(),
            OrdinaryForkBootstrapProvenanceV1 {
                source_runtime: source_report.observation().runtime.clone(),
                source_activation: source_report.observation().activation.clone(),
                source_native_session_id: source_report.observation().native_session_id.clone(),
                transferred_child_native_session_id: observed.native_session_id.clone(),
            },
            vec![transport_api_types::NativeProcessWaitObservationV1 {
                pid: 123,
                termination: observed.native_termination.clone(),
                recovery_termination: None,
            }],
        )
        .unwrap();
        let registered_frame = RuntimeFrameIdentityV1 {
            frame_sequence: 2,
            ..observed.frame_identity.clone()
        };
        let mut registered = AgentEvent::message(
            "synthetic-service-observation",
            activation.orchestration_session_id.clone(),
            activation.bootstrap_run_id.clone(),
            MessageEventKind::Registered,
            "registered",
        );
        registered.participant_id = Some(activation.retained_participant_id.clone());
        registered.backend_id = Some(activation.target_backend_id.clone());
        registered.role = Some("member".into());
        registered.world_id = Some(activation.target_world.world_id.clone());
        registered.world_generation = Some(activation.target_world.world_generation);
        registered.parent_participant_id = activation.source_participant_id.clone();
        registered.span_id = Some(observed.span_id.clone());
        registered.event_identity = Some(RuntimeEventIdentityV1 {
            event_id: "fork-owner-registered".into(),
            event_sequence: 1,
        });
        registered.data = serde_json::json!({"schema":"agent_api.session.handle.v1", "session":{"id":"logical-fork-bootstrap-id"}, "internal":{"uaa_session_id": observed.native_session_id}});
        let physical_snapshot = || {
            let mut snapshot = BTreeMap::new();
            let mut pending = vec![fixture.home.clone()];
            while let Some(path) = pending.pop() {
                let metadata = std::fs::symlink_metadata(&path).unwrap();
                assert!(metadata.is_file() || metadata.is_dir());
                let bytes = metadata.is_file().then(|| std::fs::read(&path).unwrap());
                snapshot.insert(
                    path.clone(),
                    (
                        metadata.dev(),
                        metadata.ino(),
                        metadata.uid(),
                        metadata.gid(),
                        metadata.mode(),
                        metadata.nlink(),
                        metadata.mtime(),
                        metadata.mtime_nsec(),
                        metadata.ctime(),
                        metadata.ctime_nsec(),
                        bytes,
                    ),
                );
                if metadata.is_dir() {
                    pending.extend(
                        std::fs::read_dir(&path)
                            .unwrap()
                            .map(|entry| entry.unwrap().path()),
                    );
                }
            }
            snapshot
        };

        let before_translation = physical_snapshot();
        assert!(
            store
                .resolve_hsa_retained_continue_translation_authority(
                    source.selection.original_session_id(),
                    source.selection.current_participant_id(),
                    &activation.retained_participant_id,
                )
                .is_err(),
            "an unadopted Fork must not enter the toolbox retained frontier"
        );
        assert_eq!(before_translation, physical_snapshot());
        let before_start = physical_snapshot();
        assert!(
            runtime
                .record_native_fork_bootstrap_completion(
                    &source.selection,
                    &registered_frame,
                    &registered,
                    &report
                )
                .is_err(),
            "completion cannot substitute for the original observed transport Start"
        );
        assert_eq!(before_start, physical_snapshot());
        let start_frame = RuntimeFrameIdentityV1 {
            frame_sequence: 1,
            ..observed.frame_identity.clone()
        };
        runtime
            .observe_ordinary_fork_transport_start(
                &source.selection,
                activation,
                &start_frame,
                &observed.span_id,
            )
            .unwrap();
        let started = physical_snapshot();
        runtime
            .observe_ordinary_fork_transport_start(
                &source.selection,
                activation,
                &start_frame,
                &observed.span_id,
            )
            .unwrap();
        assert_eq!(
            started,
            physical_snapshot(),
            "exact Start join cannot replace source metadata"
        );
        assert!(runtime
            .observe_ordinary_fork_transport_start(
                &source.selection,
                activation,
                &start_frame,
                "changed-span"
            )
            .is_err());
        assert_eq!(started, physical_snapshot());
        runtime
            .validate_ordinary_fork_registered(
                &source.selection,
                activation,
                &registered_frame,
                &registered,
            )
            .unwrap();
        let mut wrong_readiness = registered.clone();
        wrong_readiness.parent_participant_id = None;
        assert!(runtime
            .validate_ordinary_fork_registered(
                &source.selection,
                activation,
                &registered_frame,
                &wrong_readiness
            )
            .is_err());
        assert_eq!(started, physical_snapshot());
        let before = physical_snapshot();
        let mut wrong_source = report.provenance().clone();
        wrong_source.source_native_session_id = "other-native-source".into();
        let wrong_source = RetainedForkBootstrapCompletionV1::from_observation(
            report.observation().clone(),
            wrong_source,
            report.native_processes().to_vec(),
        )
        .unwrap();
        assert!(runtime
            .record_native_fork_bootstrap_completion(
                &source.selection,
                &registered_frame,
                &registered,
                &wrong_source
            )
            .is_err());
        assert_eq!(before, physical_snapshot());
        if let Some(point) = crash_point {
            set_ordinary_fork_r0_crash_point_for_test(point);
        }
        let adoption = runtime.record_native_fork_bootstrap_completion(
            &source.selection,
            &registered_frame,
            &registered,
            &report,
        );
        let adopted = if let Some(point) = crash_point {
            assert!(adoption
                .unwrap_err()
                .to_string()
                .contains("injected ordinary Fork R0 crash"));
            let pending = runtime
                .read_existing_ordinary_fork_record(
                    &source.authority,
                    &activation.orchestration_session_id,
                    &activation.retained_participant_id,
                )
                .unwrap()
                .unwrap();
            assert!(
                pending.registration.is_none(),
                "crash precedes admission-side R0 adoption"
            );
            assert_eq!(pending.native_claim_id, claim.record.native_claim_id);
            let root = source.authority.read_preserved_start_root_v2().unwrap();
            let requests = root
                .retained_worker_registration_request_index
                .values()
                .filter(|request| {
                    request.retained_participant_id == activation.retained_participant_id
                })
                .collect::<Vec<_>>();
            assert_eq!(
                requests.len(),
                1,
                "the real R0 publisher must retain one exact request"
            );
            assert!(matches!(
                (&point, &requests[0].state),
                (
                    OrdinaryForkR0CrashPointV1::AfterReservation,
                    RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
                ) | (
                    OrdinaryForkR0CrashPointV1::AfterApplication,
                    RetainedWorkerAuthorityRegistrationRequestStateV1::Applied { .. }
                )
            ));
            let before_inspection = component_physical_tree_snapshot(&fixture.home);
            source.selection.with_ordinary_inspection_sources_v1(|_, facts| {
            facts.with_retained_sources(|snapshot, admission, keys, _| -> anyhow::Result<()> {
                use super::super::retained_worker_runtime::{read_ordinary_retained_worker_from_sources_v1, OrdinaryRetainedWorkerReadV1};
                let row = read_ordinary_retained_worker_from_sources_v1(snapshot, admission, keys,
                    &activation.orchestration_session_id, facts.current.authority.world_binding.as_ref().unwrap(),
                    &activation.retained_participant_id, &crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(&source.selection)?)
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
                assert!(matches!(&row, OrdinaryRetainedWorkerReadV1::Fork { native_bootstrap_completed: true, .. }));
                assert_eq!(row.inspection_registration().is_some(), point == OrdinaryForkR0CrashPointV1::AfterApplication,
                    "original HSA Applied truth must be readable before admission adoption; Reserved is not Applied");
                if let Some(registration) = row.inspection_registration() {
                    let graph = facts.retained_target_graph(&registration.registration_id, &activation.retained_participant_id)
                        .map_err(|error| anyhow::anyhow!("{error}"))?;
                    row.validate_inspected_graph(&graph).map_err(|error| anyhow::anyhow!("{error}"))?;
                    assert_eq!(graph.resume_handle.internal_uaa_session_id, observed.native_session_id);
                }
                Ok(())
            }).map_err(|error| anyhow::anyhow!("{error}"))?
        }).unwrap();
            assert_eq!(
                before_inspection,
                component_physical_tree_snapshot(&fixture.home),
                "interrupted R0 inspection must not adopt, reconcile, or rewrite metadata"
            );
            let refreshed = source.selection.refresh_original_selection().unwrap();
            runtime
                .recover_ordinary_fork_registration(&refreshed, &pending)
                .unwrap()
        } else {
            adoption.unwrap()
        };
        assert!(adopted.registration.is_some());
        {
            let held = super::super::host_session_authority::store::hold_existing_accepted_work_authority_snapshot(&source.authority).unwrap();
            let sources = held
                .capture_world_controller_completion_sources(source.selection.original_session_id())
                .unwrap();
            let classified =
                super::super::retained_worker_runtime::decode_world_controller_admission_sources(
                    &held,
                    sources.admission_registry_bytes(),
                    &sources.admission_key_files(),
                    source.selection.original_session_id(),
                    &super::super::host_session_authority::schema::WorldBindingV1 {
                        world_id: activation.target_world.world_id.clone(),
                        world_generation: activation.target_world.world_generation,
                    },
                )
                .unwrap();
            assert_eq!(
                classified.completed_bootstraps.len(),
                2,
                "actual Spawn and Fork owner completions both classify"
            );
            assert!(classified.unresolved_admission_participant_ids.is_empty());
            assert!(
                classified
                    .unclassified_hsa_retained_participant_ids
                    .is_empty(),
                "Fork R0 cannot become an unknown or poison original Spawn coverage"
            );
            sources.verify_sources().unwrap();
        }
        let target = runtime
            .resolve_ordinary_fork_registered_target(&source.authority, &adopted)
            .unwrap();
        assert_eq!(
            target.resume_handle.internal_uaa_session_id,
            observed.native_session_id
        );
        assert_ne!(
            target.resume_handle.internal_uaa_session_id,
            format!("uaa_{}", activation.bootstrap_run_id)
        );
        let current = source
            .authority
            .resolve_current_exact(source.selection.original_session_id(), None)
            .unwrap();
        let resolved = resolve_canonical_retained_world_dispatch_target(
            &source.authority,
            &current,
            &activation.retained_participant_id,
            &activation.target_backend_id,
        )
        .unwrap();
        assert_eq!(resolved.participant_id, activation.retained_participant_id);
        let before_translation = physical_snapshot();
        let translated = store
            .resolve_hsa_retained_continue_translation_authority(
                source.selection.original_session_id(),
                &current.caller.participant_id,
                &activation.retained_participant_id,
            )
            .expect("genuine R0 Fork must pass the actual toolbox translation frontier")
            .expect("genuine HSA authority");
        let translated_target = translated.retained_target.as_ref().unwrap();
        assert_eq!(
            translated_target.participant_id,
            activation.retained_participant_id
        );
        assert_eq!(translated_target.backend_id, activation.target_backend_id);
        assert_eq!(translated.world_id, activation.target_world.world_id);
        assert_eq!(
            translated.world_generation,
            activation.target_world.world_generation
        );
        for (session, caller, participant) in [
            (
                source.selection.original_session_id(),
                "other-caller",
                activation.retained_participant_id.as_str(),
            ),
            (
                "absent-session",
                current.caller.participant_id.as_str(),
                activation.retained_participant_id.as_str(),
            ),
            (
                source.selection.original_session_id(),
                current.caller.participant_id.as_str(),
                "absent-child",
            ),
        ] {
            assert!(store
                .resolve_hsa_retained_continue_translation_authority(session, caller, participant)
                .is_err());
        }
        assert_eq!(
            before_translation,
            physical_snapshot(),
            "translation and refusals must preserve physical authority state"
        );
        assert!(runtime
            .read_existing_admission_record(
                &source.authority,
                source.selection.original_session_id(),
                &source.e2_launch_activation.retained_participant_id
            )
            .unwrap()
            .is_some());
        let published = physical_snapshot();
        let retry = runtime
            .record_native_fork_bootstrap_completion(
                &source.selection,
                &registered_frame,
                &registered,
                &report,
            )
            .unwrap();
        assert_eq!(adopted, retry);
        assert_eq!(
            published,
            physical_snapshot(),
            "exact completion retry must preserve bytes/inodes/modes/times"
        );
        assert!(
            runtime
                .claim_ordinary_fork_native(
                    &source.selection,
                    activation,
                    &source.admission_plan.descriptor_and_runtime_plan.descriptor,
                    None
                )
                .is_err(),
            "pre-registration current authority cannot authorize another operation"
        );
        assert_eq!(published, physical_snapshot());
        let refreshed = source.selection.refresh_original_selection().unwrap();
        assert_eq!(
            refreshed.requested_selection(),
            source.selection.requested_selection()
        );
        assert_eq!(
            refreshed.native_selection(),
            source.selection.native_selection()
        );
        let joined = runtime
            .claim_ordinary_fork_native(
                &refreshed,
                activation,
                &source.admission_plan.descriptor_and_runtime_plan.descriptor,
                None,
            )
            .unwrap();
        assert!(!joined.new_native_claim);
        assert_eq!(published, physical_snapshot());
        let mut wrong_registered = registered.clone();
        wrong_registered.data["changed_observation"] = true.into();
        assert!(runtime
            .record_native_fork_bootstrap_completion(
                &source.selection,
                &registered_frame,
                &wrong_registered,
                &report
            )
            .is_err());
        assert_eq!(published, physical_snapshot());
        let registry_path = fixture
            .home
            .join("authority-v1/retained-worker-admission-v1/registry-v1.json");
        let registry_bytes = std::fs::read(&registry_path).unwrap();
        let mut missing_coverage: serde_json::Value =
            serde_json::from_slice(&registry_bytes).unwrap();
        missing_coverage
            .as_object_mut()
            .unwrap()
            .remove("ordinary_forks_v1");
        std::fs::write(
            &registry_path,
            super::super::host_session_authority::canonical_json::to_vec(&missing_coverage)
                .unwrap(),
        )
        .unwrap();
        let uncovered = physical_snapshot();
        assert!(
            runtime
                .read_existing_admission_record(
                    &source.authority,
                    source.selection.original_session_id(),
                    &source.e2_launch_activation.retained_participant_id
                )
                .is_err(),
            "every original HSA registration must have exact coverage"
        );
        assert_eq!(uncovered, physical_snapshot());
        std::fs::write(&registry_path, registry_bytes).unwrap();
        eprintln!("ordinary Fork R0 source-owner case {crash_point:?}: exact native identity/adoption/retry preserved; zero native sends");
    }
}

#[test]
fn test_ordinary_fork_unowned_e2_children_keep_population_capacity() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*, state_store::AgentRuntimeStateStore,
    };
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_fork_world_worker_bootstrap;
    // These are genuine E2 publisher records. Dropping local preparation does
    // not prove that durable native ownership is absent or release capacity.
    let (fixture, source, _) = completed_spawn_for_fork_claim_test();
    let store = AgentRuntimeStateStore::new().unwrap();
    for index in 0..4 {
        let request = WorldDispatchRequestV1 {
            request_id: Some(format!("req_unowned_fork_{index}")),
            idempotency_key: Some(format!("idem_unowned_fork_{index}")),
            orchestration_session_id: Some(source.selection.original_session_id().into()),
            caller_participant_id: Some(source.selection.current_participant_id().into()),
            action: WorldDispatchActionV1::ForkWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(source.descriptor.backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(
                source.e2_launch_activation.retained_participant_id.clone(),
            ),
            world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
            world_generation: Some(source.e2_launch_activation.target_world.world_generation),
            dispatch_policy_narrowing: None,
            payload: WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
                prompt: "source-owner population test; zero native sends".into(),
                fork_reason: None,
                fork_strategy: None,
            }),
        }
        .validate()
        .unwrap();
        let before = component_tree_snapshot(&fixture.home);
        let result = prepare_ordinary_fork_world_worker_bootstrap(
            Arc::clone(&source.selection),
            &store,
            request,
        );
        if index == 3 {
            assert!(result.is_err(), "one Spawn plus three E2 Fork children without ownership records must fill cap four");
            assert!(result
                .err()
                .unwrap()
                .to_string()
                .contains("retained_worker_cap_reached"));
            assert_eq!(
                before,
                component_tree_snapshot(&fixture.home),
                "cap refusal must not publish another E2 child"
            );
        } else {
            let prepared = result.unwrap();
            assert!(prepared
                .bootstrap
                .policy_commitment
                .as_ref()
                .unwrap()
                .native_issuance
                .is_some());
            drop(prepared);
        }
    }

    let request = WorldDispatchRequestV1 {
        request_id: Some("req_spawn_after_unowned_forks".into()),
        idempotency_key: Some("idem_spawn_after_unowned_forks".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::SpawnWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.descriptor.backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
            prompt: "must not enter native after unresolved Forks".into(),
        }),
    }
    .validate()
    .unwrap();
    let result = crate::execution::orchestrator_world_dispatch::prepare_ordinary_authority_bound_spawn_world_worker(Arc::clone(&source.selection), request);
    assert!(
        result.is_err(),
        "Spawn must count the same durable unresolved Fork population"
    );
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("retained_worker_cap_reached"));
}

#[test]
fn test_ordinary_world_bootstrap_source_cut_uses_completed_native_owner() {
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let held = super::super::host_session_authority::store::hold_existing_accepted_work_authority_snapshot(&source.authority).unwrap();
    let sources = held
        .capture_world_controller_completion_sources(source.selection.original_session_id())
        .unwrap();
    let before = component_tree_snapshot(&fixture.home);
    let cut = super::super::obligation_ledger::world_bound_controller::authenticate_world_controller_work_attention_cut(
        &sources, &super::super::host_session_authority::schema::WorldBindingV1 {
            world_id: source.e2_launch_activation.target_world.world_id.clone(),
            world_generation: source.e2_launch_activation.target_world.world_generation,
        },
    ).unwrap();
    assert!(
        !cut.cut().e2_session_source_ids.is_empty(),
        "genuine bootstrap source must be represented"
    );
    assert!(
        cut.cut().unresolved_e2_source_ids.is_empty(),
        "checked native bootstrap must classify its exact E2 sources"
    );
    assert!(
        cut.cut().unresolved_admission_participant_ids.is_empty(),
        "parked retained worker is not active bootstrap work"
    );
    assert!(cut
        .cut()
        .unclassified_hsa_retained_participant_ids
        .is_empty());
    cut.verify_sources().unwrap();
    assert_eq!(before, component_tree_snapshot(&fixture.home));
}

#[test]
fn test_ordinary_world_held_history_preserves_original_object_and_key_custody() {
    use super::super::host_session_authority::store::hold_existing_accepted_work_authority_snapshot;
    for fault in [
        "equal-object",
        "equal-key",
        "namespace",
        "alter",
        "mode",
        "hardlink",
        "delete",
    ] {
        if fault == "equal-key" {
            // The history edges in this fixture use canonical commitments.
            // E2's actual captured MAC key remains part of the same held source.
            let (fixture, source, _) = completed_spawn_for_fork_claim_test();
            let held = hold_existing_accepted_work_authority_snapshot(&source.authority).unwrap();
            let super::super::host_session_authority::store::DispatchPolicyCommitmentPhysicalReadV1::Present(e2) = held.snapshot().e2() else {
                panic!("genuine completed Spawn must have captured E2 material");
            };
            assert_eq!(
                e2.key_files().len(),
                1,
                "fresh fixture has one actual E2 MAC key"
            );
            let key = fixture
                .home
                .join("authority-v1/dispatch-policy-commitment-v1/keys")
                .join(e2.key_files().iter().next().unwrap().0);
            let root_bytes =
                std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap();
            held.verify_sources().unwrap();
            let next = key.with_extension("replacement");
            std::fs::write(&next, std::fs::read(&key).unwrap()).unwrap();
            std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600)).unwrap();
            std::fs::rename(next, &key).unwrap();
            let after_fault = component_tree_snapshot(&fixture.home);
            assert!(
                held.verify_sources().is_err(),
                "held E2 MAC-key custody cannot be replaced by equal bytes"
            );
            assert_eq!(
                root_bytes,
                std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap()
            );
            assert_eq!(after_fault, component_tree_snapshot(&fixture.home));
            continue;
        }
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        // Genuine World Start enrollment publishes V3 before the held history
        // capture; a plain Applied V2 Start does not use the typed V3 reader.
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap()
            )
            .unwrap()["schema_version"],
            3
        );
        let root = fixture.authority.read_preserved_start_root_v2().unwrap();
        let origin = root
            .transition_intent_map
            .get(&fixture.context.intent_id)
            .unwrap();
        let path = fixture.home.join(format!(
            "authority-v1/objects/host-attach-contract/v1/{}.obj",
            origin.host_attach_contract_ref.ref_id
        ));
        let held = hold_existing_accepted_work_authority_snapshot(&fixture.authority).unwrap();
        held.verify_sources().unwrap();
        let root_bytes =
            std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap();
        match fault {
            "equal-object" | "equal-key" => {
                let next = path.with_extension("replacement");
                std::fs::write(&next, std::fs::read(&path).unwrap()).unwrap();
                std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600)).unwrap();
                std::fs::rename(next, &path).unwrap();
            }
            "namespace" => {
                let parent = path.parent().unwrap();
                let moved = parent.with_extension("retained");
                std::fs::rename(parent, &moved).unwrap();
                std::fs::create_dir(parent).unwrap();
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).unwrap();
                std::fs::copy(moved.join(path.file_name().unwrap()), &path).unwrap();
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
            "alter" => {
                let mut bytes = std::fs::read(&path).unwrap();
                bytes[0] ^= 1;
                std::fs::write(&path, bytes).unwrap();
            }
            "mode" => {
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap()
            }
            "hardlink" => std::fs::hard_link(&path, path.with_extension("alias")).unwrap(),
            "delete" => std::fs::remove_file(&path).unwrap(),
            _ => unreachable!(),
        }
        let after_fault = component_tree_snapshot(&fixture.home);
        assert!(
            held.verify_sources().is_err(),
            "held complete V3 history must retain original custody for {fault}"
        );
        assert_eq!(
            root_bytes,
            std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap()
        );
        assert_eq!(
            after_fault,
            component_tree_snapshot(&fixture.home),
            "physical refusal adds no writes for {fault}"
        );
    }
}

#[derive(Clone, Copy)]
enum OrdinaryPopulationFaultConsumerV1 {
    ForkE2,
    ForkClaim,
    Spawn,
}

fn exercise_ordinary_population_source_fault_before_publication(
    consumer: OrdinaryPopulationFaultConsumerV1,
) {
    let fork_e2_publication = matches!(consumer, OrdinaryPopulationFaultConsumerV1::ForkE2);
    let spawn = matches!(consumer, OrdinaryPopulationFaultConsumerV1::Spawn);
    use super::super::{dispatch_contract::*, retained_worker_runtime::*};
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_fork_world_worker_bootstrap;
    let (fixture, source, _) = completed_spawn_for_fork_claim_test();
    let store = super::super::state_store::AgentRuntimeStateStore::new().unwrap();
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_population_physical_fault".into()),
        idempotency_key: Some("idem_population_physical_fault".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: if spawn {
            WorldDispatchActionV1::SpawnWorldWorker
        } else {
            WorldDispatchActionV1::ForkWorldWorker
        },
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.descriptor.backend_id.clone()),
        task_run_id: None,
        target_participant_id: if spawn {
            None
        } else {
            Some(source.e2_launch_activation.retained_participant_id.clone())
        },
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: if spawn {
            WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
                prompt: "source-guard fault test; zero native sends".into(),
            })
        } else {
            WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
                prompt: "source-guard fault test; zero native sends".into(),
                fork_reason: None,
                fork_strategy: None,
            })
        },
    }
    .validate()
    .unwrap();
    let mut prepared = if fork_e2_publication || spawn {
        None
    } else {
        Some(
            prepare_ordinary_fork_world_worker_bootstrap(
                Arc::clone(&source.selection),
                &store,
                request.clone(),
            )
            .unwrap(),
        )
    };
    let admission_path = fixture
        .home
        .join("authority-v1/retained-worker-admission-v1/registry-v1.json");
    let e2_path = fixture
        .home
        .join("authority-v1/dispatch-policy-commitment-v1/registry-v1.json");
    let (source_path, consuming_path) = if fork_e2_publication {
        (admission_path, e2_path)
    } else {
        (e2_path, admission_path)
    };
    let before_consuming = std::fs::read(&consuming_path).unwrap();
    let fired = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let fault_fired = Arc::clone(&fired);
    set_ordinary_population_before_publication_hook(move || {
        let next = source_path.with_extension("replacement");
        std::fs::write(&next, std::fs::read(&source_path).unwrap()).unwrap();
        std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::rename(next, source_path).unwrap();
        fault_fired.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    let result = if spawn {
        crate::execution::orchestrator_world_dispatch::prepare_ordinary_authority_bound_spawn_world_worker(Arc::clone(&source.selection), request).map(|_| ())
    } else if fork_e2_publication {
        prepare_ordinary_fork_world_worker_bootstrap(Arc::clone(&source.selection), &store, request)
            .map(|_| ())
    } else {
        let policy = prepared
            .as_mut()
            .unwrap()
            .bootstrap
            .policy_commitment
            .as_mut()
            .unwrap();
        RetainedWorkerRuntime
            .claim_ordinary_fork_native(
                &source.selection,
                &policy.e2_launch_activation,
                &source.admission_plan.descriptor_and_runtime_plan.descriptor,
                policy.native_issuance.take(),
            )
            .map(|_| ())
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    };
    assert!(
        fired.load(std::sync::atomic::Ordering::SeqCst),
        "fault must reach the real publication boundary"
    );
    assert!(
        result.is_err(),
        "changed original population custody must refuse"
    );
    assert_eq!(
        before_consuming,
        std::fs::read(&consuming_path).unwrap(),
        "physical source refusal must precede consuming registry publication"
    );
}

#[test]
fn test_ordinary_population_guard_before_fork_e2_publication() {
    exercise_ordinary_population_source_fault_before_publication(
        OrdinaryPopulationFaultConsumerV1::ForkE2,
    );
}

#[test]
fn test_ordinary_population_guard_before_fork_native_claim() {
    exercise_ordinary_population_source_fault_before_publication(
        OrdinaryPopulationFaultConsumerV1::ForkClaim,
    );
}

#[test]
fn test_ordinary_population_guard_before_spawn_admission() {
    exercise_ordinary_population_source_fault_before_publication(
        OrdinaryPopulationFaultConsumerV1::Spawn,
    );
}

type ComponentPhysicalTreeSnapshotV1 = BTreeMap<
    PathBuf,
    (
        u64,
        u64,
        u32,
        u32,
        u32,
        u64,
        u64,
        i64,
        i64,
        i64,
        i64,
        Option<Vec<u8>>,
    ),
>;

fn component_physical_tree_snapshot(root: &Path) -> ComponentPhysicalTreeSnapshotV1 {
    let mut result = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let m = std::fs::symlink_metadata(&path).unwrap();
        let bytes = m.is_file().then(|| std::fs::read(&path).unwrap());
        result.insert(
            path.strip_prefix(root).unwrap().to_path_buf(),
            (
                m.dev(),
                m.ino(),
                m.uid(),
                m.gid(),
                m.mode(),
                m.nlink(),
                m.len(),
                m.mtime(),
                m.mtime_nsec(),
                m.ctime(),
                m.ctime_nsec(),
                bytes,
            ),
        );
        if m.is_dir() {
            pending.extend(
                std::fs::read_dir(&path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
        }
    }
    result
}

#[test]
fn test_ordinary_inspection_sources_read_fresh_r0_without_refreshing_send_authority() {
    use super::super::host_session_authority::facade::RetainedWorkerAuthorityPreconditionV1;
    use super::super::retained_worker_runtime::{
        RetainedWorkerRegistrationPlanV1, RetainedWorkerRuntime,
    };
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let before_r0 = source
        .authority
        .resolve_current_exact(source.selection.original_session_id(), None)
        .unwrap();
    RetainedWorkerRuntime
        .register_retained_target(
            &source.authority,
            &RetainedWorkerRegistrationPlanV1 {
                registration_request_id: "ordinary-inspection-extra-r0".into(),
                orchestration_session_id: source.selection.original_session_id().into(),
                expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                    authority_store_id: before_r0.observation.authority_store_id.clone(),
                    authority_revision: before_r0.observation.authority_revision,
                    authority_record_commitment: before_r0
                        .observation
                        .authority_record_commitment
                        .clone(),
                },
                retained_participant_id: "ordinary-inspection-extra-worker".into(),
                descriptor: source
                    .admission_plan
                    .descriptor_and_runtime_plan
                    .descriptor
                    .clone(),
                internal_uaa_session_id: "synthetic-inspection-extra-native".into(),
            },
        )
        .unwrap();
    assert!(
        source.selection.revalidate().is_err(),
        "old executing snapshot must require a genuine refresh"
    );
    let registration_id = source
        .authority
        .read_preserved_start_root_v2()
        .unwrap()
        .retained_worker_registration_journal
        .values()
        .find(|registration| {
            registration.retained_participant_id
                == source.e2_launch_activation.retained_participant_id
        })
        .unwrap()
        .registration_id
        .clone();
    let original_native = source.selection.native_selection().clone();
    let original_requested = source.selection.requested_selection().clone();
    let inventory_path = fixture
        .context
        .sources
        .inventory
        .get(&original_requested.realized_agent_id)
        .unwrap()
        .path
        .clone();
    assert!(
        inventory_path.starts_with(fixture._temp.path()),
        "live inventory mutation is confined to this owned fixture"
    );
    std::fs::write(inventory_path, b"malformed: [").unwrap();
    std::fs::write(fixture.home.join("config.yaml"), b"malformed: [").unwrap();
    let token = std::env::var_os(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    let before = component_physical_tree_snapshot(&fixture.home);
    let observed = source.selection.with_ordinary_inspection_sources_v1(|context, facts| {
        assert_eq!(context.selections.native, original_native);
        assert_eq!(context.selections.requested, original_requested);
        assert_eq!(facts.current.observation.authority_revision, before_r0.observation.authority_revision + 1);
        assert_eq!(facts.original_start.intent_id, context.intent_id);
        assert_eq!(facts.original_descriptor, context.descriptor);
        assert_eq!(facts.original_policy.schema_version, 1);
        let graph = facts.retained_target_graph(&registration_id, &source.e2_launch_activation.retained_participant_id)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        assert_eq!(graph.registration.registration_id, registration_id);
        assert_eq!(graph.descriptor.backend_id, original_requested.backend_id);
        assert_eq!(graph.retained_worker.participant_id, source.e2_launch_activation.retained_participant_id);
        facts.with_retained_sources(|snapshot, admission, keys, _supervisor| -> anyhow::Result<()> {
            assert_eq!(snapshot.hsa_root_revision_at_read(), facts.current.observation.root_revision);
            assert_eq!(snapshot.authority_store_id(), facts.current.observation.authority_store_id);
            assert!(admission.is_some() && !keys.is_empty(), "actual original admission source remains captured");
            use super::super::retained_worker_runtime::{read_ordinary_retained_worker_from_sources_v1, OrdinaryRetainedWorkerReadV1};
            let world = facts.current.authority.world_binding.as_ref().unwrap();
            let row = read_ordinary_retained_worker_from_sources_v1(snapshot, admission, keys,
                &context.orchestration_session_id, world, &source.e2_launch_activation.retained_participant_id,
                &crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(&source.selection)?)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            let OrdinaryRetainedWorkerReadV1::Spawn { record, native_bootstrap_completed, .. } = row else { panic!("genuine Spawn owner changed family") };
            assert!(native_bootstrap_completed);
            let mut wrong_start = crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(&source.selection)?;
            wrong_start.requested_selection_sha256 = "0".repeat(64);
            assert!(read_ordinary_retained_worker_from_sources_v1(snapshot, admission, keys,
                &context.orchestration_session_id, world, &source.e2_launch_activation.retained_participant_id, &wrong_start).is_err(),
                "native bootstrap source must preserve the original complete requested selection");
            assert_eq!(record.retained_participant_id, graph.registration.retained_participant_id);
            assert!(read_ordinary_retained_worker_from_sources_v1(snapshot, admission, keys,
                &context.orchestration_session_id, world, "ordinary-inspection-extra-worker",
                &crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(&source.selection)?).is_err(),
                "unknown HSA R0 data must not manufacture an admission owner");

            let work = super::super::state_store::authenticate_ordinary_world_work_session_cut_v1(snapshot,
                context.orchestration_session_id.as_str(), facts.current.authority.world_binding.as_ref().unwrap())?;
            assert!(work.proposals().is_empty() && work.acceptances().is_empty(), "source-only R0 metadata cannot fabricate accepted work");
            Ok(())
        }).map_err(|error| anyhow::anyhow!(error.to_string()))??;
        Ok(facts.current.observation.clone())
    }).unwrap();
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    assert_eq!(
        source.selection.current_authority_revision(),
        before_r0.observation.authority_revision,
        "inspection returns data without upgrading the held executing snapshot"
    );
    assert_eq!(
        observed.authority_revision,
        source.selection.current_authority_revision() + 1
    );
    match token {
        Some(value) => std::env::set_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, value),
        None => std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN),
    }
}

#[test]
fn test_ordinary_inspection_sources_refuse_pending_temp_without_reconciliation() {
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let root_bytes = std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap();
    let root: serde_json::Value = serde_json::from_slice(&root_bytes).unwrap();
    let temp = fixture.home.join("authority-v1/tmp").join(format!(
        "root--r{}--{}.tmp",
        root["root_revision"].as_u64().unwrap(),
        "a".repeat(32)
    ));
    std::fs::write(&temp, root_bytes).unwrap();
    std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o600)).unwrap();
    let before = component_physical_tree_snapshot(&fixture.home);
    let mut consumed = false;
    assert!(source
        .selection
        .with_ordinary_inspection_sources_v1(|_, _| {
            consumed = true;
            Ok(())
        })
        .is_err());
    assert!(
        !consumed,
        "unreconciled source cannot reach the inspecting consumer"
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
}

#[test]
fn test_ordinary_inspection_sources_hold_typed_objects_through_success_and_refusal() {
    for semantic_refusal in [false, true] {
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
        let root = source.authority.read_preserved_start_root_v2().unwrap();
        let original = root
            .transition_intent_map
            .get(&fixture.context.intent_id)
            .unwrap();
        let path = fixture.home.join(format!(
            "authority-v1/objects/host-attach-contract/v1/{}.obj",
            original.host_attach_contract_ref.ref_id
        ));
        let original_root =
            std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap();
        let mut after_fault = None;
        let outcome = source
            .selection
            .with_ordinary_inspection_sources_v1(|_, _| {
                let next = path.with_extension("replacement");
                std::fs::write(&next, std::fs::read(&path).unwrap()).unwrap();
                std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600)).unwrap();
                std::fs::rename(next, &path).unwrap();
                after_fault = Some(component_physical_tree_snapshot(&fixture.home));
                if semantic_refusal {
                    anyhow::bail!("inspection-semantic-marker");
                }
                Ok(())
            });
        let error = outcome.unwrap_err().to_string();
        assert!(
            error.contains("inspection source custody refused"),
            "physical original source failure must dominate semantic outcome: {error}"
        );
        assert!(!error.contains("inspection-semantic-marker"));
        assert_eq!(
            original_root,
            std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap()
        );
        assert_eq!(
            after_fault.unwrap(),
            component_physical_tree_snapshot(&fixture.home)
        );
    }
}

#[test]
fn test_ordinary_inspection_sources_compose_registry_and_r0_leaf_custody() {
    for fault in ["e2", "b1", "admission", "r0-descriptor"] {
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
        let root = source.authority.read_preserved_start_root_v2().unwrap();
        let registration = root
            .retained_worker_registration_journal
            .values()
            .find(|registration| {
                registration.retained_participant_id
                    == source.e2_launch_activation.retained_participant_id
            })
            .unwrap();
        let registration_id = registration.registration_id.clone();
        let path = match fault {
            "e2" => fixture
                .home
                .join("authority-v1/dispatch-policy-commitment-v1/registry-v1.json"),
            "b1" => fixture
                .home
                .join("run/agent-hub/world-work-receipt-registry-v1.json"),
            "admission" => fixture
                .home
                .join("authority-v1/retained-worker-admission-v1/registry-v1.json"),
            "r0-descriptor" => fixture.home.join(format!(
                "authority-v1/objects/agent-descriptor/v1/{}.obj",
                registration.descriptor_ref.ref_id
            )),
            _ => unreachable!(),
        };
        assert!(path.is_file(), "fault reaches actual {fault} source");
        let original_root =
            std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap();
        let mut after_fault = None;
        let outcome = source
            .selection
            .with_ordinary_inspection_sources_v1(|_, facts| {
                let graph = facts
                    .retained_target_graph(
                        &registration_id,
                        &source.e2_launch_activation.retained_participant_id,
                    )
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                assert_eq!(graph.registration.registration_id, registration_id);
                facts
                    .with_retained_sources(
                        |_, admission, keys, _supervisor| -> anyhow::Result<()> {
                            assert!(admission.is_some() && !keys.is_empty());
                            let next = path.with_extension("replacement");
                            std::fs::write(&next, std::fs::read(&path).unwrap()).unwrap();
                            std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600))
                                .unwrap();
                            std::fs::rename(next, &path).unwrap();
                            after_fault = Some(component_physical_tree_snapshot(&fixture.home));
                            if fault == "admission" {
                                anyhow::bail!("inspection-composed-semantic-marker");
                            }
                            Ok(())
                        },
                    )
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?
            });
        let error = outcome.unwrap_err().to_string();
        assert!(
            error.contains("custody refused"),
            "original composed custody must dominate for {fault}: {error}"
        );
        assert!(!error.contains("inspection-composed-semantic-marker"));
        assert_eq!(
            original_root,
            std::fs::read(fixture.home.join("authority-v1/state-root-v1.json")).unwrap()
        );
        assert_eq!(
            after_fault.unwrap(),
            component_physical_tree_snapshot(&fixture.home),
            "read refusal must add no writes for {fault}"
        );
    }
}

#[test]
fn test_ordinary_inspection_route_reports_source_facts_and_preserves_read_boundaries() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*, retained_worker_runtime::RetainedWorkerRuntime,
        state_store::AgentRuntimeStateStore,
    };
    use crate::execution::orchestrator_world_dispatch::{
        inspect_ordinary_world_worker_request, prepare_ordinary_fork_world_worker_bootstrap,
    };
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_ordinary_source_inspection".into()),
        idempotency_key: Some("idem_ordinary_source_inspection".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::InspectWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.descriptor.backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(source.e2_launch_activation.retained_participant_id.clone()),
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
    };
    let before = component_physical_tree_snapshot(&fixture.home);
    let denied =
        inspect_ordinary_world_worker_request(&source.selection, request.clone()).unwrap_err();
    assert!(
        denied
            .to_string()
            .contains("does not allow inspect_world_worker"),
        "{denied}"
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap().replace(
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, inspect_world_worker]");
    std::fs::write(&policy_path, policy).unwrap();
    let before = component_physical_tree_snapshot(&fixture.home);
    let WorldDispatchOutcomeV1::InspectWorldWorker(inspect) =
        inspect_ordinary_world_worker_request(&source.selection, request.clone()).unwrap()
    else {
        panic!("ordinary inspection changed outcome family")
    };
    assert!(inspect
        .summary
        .contains("R0 registered, native bootstrap checked complete"));
    let bootstrap = inspect.bootstrap_snapshot.as_ref().unwrap();
    assert_eq!(bootstrap.schema_version, 1);
    assert_eq!(bootstrap.bootstrap_kind, RetainedBootstrapKindV1::Spawn);
    assert!(bootstrap.registration_id.is_some() && bootstrap.native_bootstrap_completed);
    let normalized =
        super::super::tool_invocation_contract::normalize_inspect_world_worker_outcome_v1(&inspect)
            .unwrap();
    assert_eq!(normalized.bootstrap_snapshot, inspect.bootstrap_snapshot);
    let mut unsupported = inspect.clone();
    unsupported
        .bootstrap_snapshot
        .as_mut()
        .unwrap()
        .schema_version = 2;
    assert!(
        super::super::tool_invocation_contract::normalize_inspect_world_worker_outcome_v1(
            &unsupported
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<RetainedBootstrapInspectSnapshotV1>(
            serde_json::to_value(unsupported.bootstrap_snapshot.unwrap()).unwrap()
        )
        .is_err()
    );
    let mut unknown = serde_json::to_value(bootstrap).unwrap();
    unknown["invented_native_liveness"] = true.into();
    assert!(serde_json::from_value::<RetainedBootstrapInspectSnapshotV1>(unknown).is_err());

    assert!(
        inspect.snapshot.is_none(),
        "registration/closed bootstrap cannot invent current participant liveness"
    );
    assert!(
        inspect.terminal.is_none() && inspect.terminal_ref.is_none(),
        "bootstrap Exit cannot certify an accepted turn"
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    for fault in ["caller", "backend", "world", "target"] {
        let mut wrong = request.clone();
        match fault {
            "caller" => wrong.caller_participant_id = Some("other-caller".into()),
            "backend" => wrong.target_backend_id = Some("cli:other-world".into()),
            "world" => {
                wrong.world_generation = wrong.world_generation.map(|generation| generation + 1)
            }
            "target" => wrong.target_participant_id = Some("unknown-worker".into()),
            _ => unreachable!(),
        }
        let before = component_physical_tree_snapshot(&fixture.home);
        assert!(
            inspect_ordinary_world_worker_request(&source.selection, wrong).is_err(),
            "{fault}"
        );
        assert_eq!(
            before,
            component_physical_tree_snapshot(&fixture.home),
            "failed {fault} read wrote state"
        );
    }
    let fork_request = WorldDispatchRequestV1 {
        request_id: Some("req_inspection_pending_fork".into()),
        idempotency_key: Some("idem_inspection_pending_fork".into()),
        action: WorldDispatchActionV1::ForkWorldWorker,
        payload: WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
            prompt: "source-only inspection claim; never sent".into(),
            fork_reason: None,
            fork_strategy: None,
        }),
        ..request.clone()
    }
    .validate()
    .unwrap();
    let mut fork = prepare_ordinary_fork_world_worker_bootstrap(
        Arc::clone(&source.selection),
        &AgentRuntimeStateStore::new().unwrap(),
        fork_request,
    )
    .unwrap();
    let cap = fork.bootstrap.policy_commitment.as_mut().unwrap();
    let issuance = cap.native_issuance.take();
    let activation = cap.e2_launch_activation.clone();
    let claim = RetainedWorkerRuntime
        .claim_ordinary_fork_native(
            &source.selection,
            &activation,
            &source.admission_plan.descriptor_and_runtime_plan.descriptor,
            issuance,
        )
        .unwrap();
    assert!(claim.new_native_claim);
    drop(claim); // This source test never consumes a native-send permission.
    let mut pending_request = request.clone();
    pending_request.target_participant_id = Some(activation.retained_participant_id.clone());
    let before = component_physical_tree_snapshot(&fixture.home);
    let WorldDispatchOutcomeV1::InspectWorldWorker(pending) =
        inspect_ordinary_world_worker_request(&source.selection, pending_request.clone()).unwrap()
    else {
        panic!("pending Fork inspection changed outcome family")
    };
    assert!(pending
        .summary
        .contains("R0 pending, native bootstrap unresolved"));
    assert!(
        pending.snapshot.is_none()
            && pending.pending_admission.is_none()
            && pending.terminal.is_none(),
        "Fork must not borrow Spawn authority revision or synthesize logical/native liveness"
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    let bootstrap = pending.bootstrap_snapshot.as_ref().unwrap();
    assert_eq!(bootstrap.bootstrap_kind, RetainedBootstrapKindV1::Fork);
    assert!(bootstrap.registration_id.is_none() && !bootstrap.native_bootstrap_completed);
    let exact = WorldDispatchControlTargetV1::PendingRetainedAdmission {
        issuer_request_id: activation.request_id.clone(),
        target_participant_id: activation.retained_participant_id.clone(),
    };
    pending_request.target_participant_id = Some(format!(
        "substrate_target_v1:{}",
        serde_json::to_string(&exact).unwrap()
    ));
    let before = component_physical_tree_snapshot(&fixture.home);
    let WorldDispatchOutcomeV1::InspectWorldWorker(exact_pending) =
        inspect_ordinary_world_worker_request(&source.selection, pending_request.clone()).unwrap()
    else {
        panic!("exact pending Fork inspection changed outcome family")
    };
    assert_eq!(
        exact_pending.target_participant_id,
        activation.retained_participant_id
    );
    assert_eq!(exact_pending.exact_target, Some(exact));
    assert_eq!(exact_pending.bootstrap_snapshot, pending.bootstrap_snapshot);
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    let wrong = WorldDispatchControlTargetV1::PendingRetainedAdmission {
        issuer_request_id: "req_other_fork".into(),
        target_participant_id: activation.retained_participant_id.clone(),
    };
    pending_request.target_participant_id = Some(format!(
        "substrate_target_v1:{}",
        serde_json::to_string(&wrong).unwrap()
    ));
    assert!(inspect_ordinary_world_worker_request(&source.selection, pending_request).is_err());
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    let absent_acceptance = WorldDispatchControlTargetV1::AcceptedRetainedTurn {
        acceptance_record_id: format!("wwa_{}", uuid::Uuid::now_v7()),
        active_run_id: "absent-active-run".into(),
        message_id: "absent-message".into(),
        target_participant_id: source.e2_launch_activation.retained_participant_id.clone(),
    };
    let mut exact_turn_request = request.clone();
    exact_turn_request.target_participant_id = None;
    exact_turn_request.task_run_id = Some(format!(
        "substrate_target_v1:{}",
        serde_json::to_string(&absent_acceptance).unwrap()
    ));
    let before = component_physical_tree_snapshot(&fixture.home);
    let absent = inspect_ordinary_world_worker_request(&source.selection, exact_turn_request)
        .unwrap_err()
        .to_string();
    assert!(
        absent.contains("B1 source refused"),
        "exact accepted-turn carrier must reach strict B1 authentication: {absent}"
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    let inventory_path = fixture
        .context
        .sources
        .inventory
        .get(&source.selection.requested_selection().realized_agent_id)
        .unwrap()
        .path
        .clone();
    assert!(inventory_path.starts_with(fixture._temp.path()));
    std::fs::write(inventory_path, b"malformed: [").unwrap();
    std::fs::write(fixture.home.join("config.yaml"), b"malformed: [").unwrap();
    let token = std::env::var_os(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    let before = component_physical_tree_snapshot(&fixture.home);
    let result = inspect_ordinary_world_worker_request(&source.selection, request);
    match token {
        Some(value) => std::env::set_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, value),
        None => std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN),
    }
    result.unwrap();
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home), "Inspect must retain original config/inventory and acquire no current account/send permission");
}

#[test]
fn test_ordinary_inspection_sources_hold_supervisor_through_refusal() {
    for fault in [
        "same-bytes-replacement",
        "same-inode-rewrite",
        "absent-to-present",
        "unsafe-temporary",
    ] {
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
        let path = fixture
            .home
            .join("run/agent-hub/world-work-execution-supervisor-v1.json");
        assert!(!path.exists());
        // Deliberately malformed hostile fixture bytes exercise physical custody
        // only. They never become supervisor, receipt or terminal truth.
        if matches!(fault, "same-bytes-replacement" | "same-inode-rewrite") {
            std::fs::write(&path, b"malformed supervisor negative fixture").unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        if fault == "unsafe-temporary" {
            std::fs::write(
                path.with_file_name("world-work-execution-supervisor-v1-hostile.tmp"),
                b"malformed negative fixture",
            )
            .unwrap();
        }
        let before = component_physical_tree_snapshot(&fixture.home);
        let mut after_fault = None;
        let mut entered = false;
        let outcome = source
            .selection
            .with_ordinary_inspection_sources_v1(|_, facts| {
                facts
                    .with_retained_sources(|_, _, _, supervisor| -> anyhow::Result<()> {
                        entered = true;
                        match fault {
                            "same-bytes-replacement" => {
                                let next = path.with_extension("replacement");
                                std::fs::write(&next, supervisor.unwrap()).unwrap();
                                std::fs::set_permissions(
                                    &next,
                                    std::fs::Permissions::from_mode(0o600),
                                )
                                .unwrap();
                                std::fs::rename(next, &path).unwrap();
                            }
                            "same-inode-rewrite" => {
                                std::fs::write(&path, b"different malformed negative fixture")
                                    .unwrap()
                            }
                            "absent-to-present" => {
                                assert!(supervisor.is_none());
                                std::fs::write(&path, b"malformed negative fixture").unwrap();
                            }
                            _ => panic!("unsafe supervisor temporary reached consumer"),
                        }
                        after_fault = Some(component_physical_tree_snapshot(&fixture.home));
                        anyhow::bail!("supervisor-semantic-refusal-marker")
                    })
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?
            });
        let error = outcome.unwrap_err().to_string();
        assert!(error.contains("custody refused"), "{fault}: {error}");
        assert!(
            !error.contains("supervisor-semantic-refusal-marker"),
            "physical failure takes precedence"
        );
        if fault == "unsafe-temporary" {
            assert!(!entered);
            assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
        } else {
            assert!(entered);
            assert_eq!(
                after_fault.unwrap(),
                component_physical_tree_snapshot(&fixture.home)
            );
        }
    }
}

#[test]
fn test_ordinary_control_source_scope_preserves_generic_read_custody() {
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let before = component_physical_tree_snapshot(&fixture.home);
    let control = source
        .authority
        .capture_ordinary_supervisor_control_source_v1(source.selection.original_session_id())
        .unwrap();
    assert_eq!(
        control.facts.current.observation.root_revision,
        control.snapshot().hsa_root_revision_at_read()
    );
    assert_eq!(
        control.facts.original_start.intent_id,
        source.selection.original_start_intent_id()
    );
    assert_eq!(
        control.facts.original_descriptor,
        fixture.context.descriptor
    );
    assert_eq!(
        control.facts.original_contract.backend_id,
        source.selection.native_selection().backend_id
    );
    assert_eq!(control.facts.original_policy.schema_version, 1);
    assert!(
        control.admission_registry_bytes().is_some() && !control.admission_key_files().is_empty()
    );
    control.verify_sources().unwrap();
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    // A new owner can acquire Root: the source holder keeps descriptors, not a
    // lock across a future supervisor mutation or IPC.
    source.authority.read_preserved_start_root_v2().unwrap();
    let generic = super::super::host_session_authority::store::hold_existing_accepted_work_authority_snapshot(&source.authority).unwrap();
    let path = fixture
        .home
        .join("run/agent-hub/world-work-execution-supervisor-v1.json");
    assert!(!path.exists());
    // Negative fixture bytes prove the source-scope distinction only. The
    // supervisor owner must separately validate its state and authorize a write;
    // this source holder never grants that permission or interprets these bytes.
    std::fs::write(path, b"deliberately malformed supervisor scope fixture").unwrap();
    let changed = component_physical_tree_snapshot(&fixture.home);
    control.verify_sources().unwrap();
    assert!(
        generic.verify_sources().is_err(),
        "generic capture must retain its exact original namespace predicate"
    );
    assert_eq!(changed, component_physical_tree_snapshot(&fixture.home));
}

#[test]
fn test_ordinary_control_source_retains_original_root_receipt_and_key_guards() {
    for fault in [
        "root",
        "receipt",
        "e2",
        "e2-key",
        "admission",
        "typed-object",
        "other-hub-file",
        "hub-mode",
    ] {
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
        let control = source
            .authority
            .capture_ordinary_supervisor_control_source_v1(source.selection.original_session_id())
            .unwrap();
        let path = match fault {
            "root" => fixture.home.join("authority-v1/state-root-v1.json"),
            "receipt" => fixture
                .home
                .join("run/agent-hub/world-work-receipt-registry-v1.json"),
            "e2" => fixture
                .home
                .join("authority-v1/dispatch-policy-commitment-v1/registry-v1.json"),
            "e2-key" => {
                let super::super::host_session_authority::store::DispatchPolicyCommitmentPhysicalReadV1::Present(e2) = control.snapshot().e2() else { panic!("genuine E2 source missing") };
                fixture
                    .home
                    .join("authority-v1/dispatch-policy-commitment-v1/keys")
                    .join(e2.key_files().iter().next().unwrap().0)
            }
            "admission" => fixture
                .home
                .join("authority-v1/retained-worker-admission-v1/registry-v1.json"),
            "typed-object" => fixture.home.join(format!(
                "authority-v1/objects/host-attach-contract/v1/{}.obj",
                control.facts.original_start.host_attach_contract_ref.ref_id
            )),
            "other-hub-file" => fixture.home.join("run/agent-hub/uncovered-control-source"),
            "hub-mode" => fixture.home.join("run/agent-hub"),
            _ => unreachable!(),
        };
        if fault == "other-hub-file" {
            std::fs::write(&path, b"negative unrelated-source fixture").unwrap();
        } else if fault == "hub-mode" {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o750)).unwrap();
        } else {
            assert!(path.is_file(), "actual {fault} source is captured");
            let next = path.with_extension("replacement");
            std::fs::write(&next, std::fs::read(&path).unwrap()).unwrap();
            std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600)).unwrap();
            std::fs::rename(next, &path).unwrap();
        }
        let after_fault = component_physical_tree_snapshot(&fixture.home);
        assert!(
            control.verify_sources().is_err(),
            "control source must retain original {fault} custody"
        );
        assert_eq!(after_fault, component_physical_tree_snapshot(&fixture.home));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_ordinary_cancel_route_denials_and_empty_receipt_scope_are_zero_effect() {
    use crate::execution::agent_runtime::dispatch_contract::*;
    use crate::execution::orchestrator_world_dispatch::cancel_ordinary_world_work_request;
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    // An unissued exact target cannot initialize admission storage during
    // translation, even with a real active HSA caller and World.
    {
        use crate::execution::agent_runtime::tool_invocation_contract::{
            translate_host_tool_invocation_request_to_internal_dispatch_request_v1,
            HostToolInvocationRequestEnvelopeV1, HostToolRuntimeDispatchMetadataV1,
            HostToolRuntimeWorldBindingV1,
        };
        let store =
            crate::execution::agent_runtime::state_store::AgentRuntimeStateStore::new().unwrap();
        let world = fixture.context.original_world_binding.as_ref().unwrap();
        let before = component_physical_tree_snapshot(&fixture.home);
        assert!(translate_host_tool_invocation_request_to_internal_dispatch_request_v1(
            &store, &HostToolRuntimeDispatchMetadataV1 {
                request_id: "req_absent_r0_cancel_lookup".into(), idempotency_key: "idem_absent_r0_cancel_lookup".into(),
                orchestration_session_id: fixture.context.orchestration_session_id.clone(),
                caller_participant_id: fixture.context.original_participant_id.clone(),
            }, &HostToolRuntimeWorldBindingV1 { world_id: world.world_id.clone(), world_generation: world.world_generation },
            HostToolInvocationRequestEnvelopeV1 { version: 1, tool_name: "cancel_world_work".into(), tool_call_id: None,
                arguments: serde_json::json!({"exact_target":{"target_kind":"accepted_retained_turn",
                    "acceptance_record_id":"unissued-acceptance", "active_run_id":"unissued-run", "message_id":"unissued-message",
                    "target_participant_id":"unissued-retained-worker"},"payload":{}}),
            }).is_err());
        assert_eq!(
            before,
            component_physical_tree_snapshot(&fixture.home),
            "missing canonical target lookup must not initialize or reconcile admission storage"
        );
    }
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let socket = fixture._temp.path().join("unused-cancel.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket);
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_ordinary_cancel_empty_scope".into()),
        idempotency_key: Some("idem_ordinary_cancel_empty_scope".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::CancelWorldWork,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.descriptor.backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(source.e2_launch_activation.retained_participant_id.clone()),
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
    };
    let before = component_physical_tree_snapshot(&fixture.home);
    let WorldDispatchOutcomeV1::CancelWorldWork(denied) =
        cancel_ordinary_world_work_request(&source.selection, request.clone())
            .await
            .unwrap()
    else {
        panic!("ordinary Cancel changed outcome family")
    };
    assert_eq!(denied.state, CancelWorldWorkTerminalStateV1::PolicyDenied);
    assert!(denied.cancel_request_id.is_none() && denied.terminal.is_none());
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap().replace(
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, cancel_world_work]");
    std::fs::write(&policy_path, policy).unwrap();
    let inventory_path = fixture
        .context
        .sources
        .inventory
        .get(&source.selection.requested_selection().realized_agent_id)
        .unwrap()
        .path
        .clone();
    assert!(inventory_path.starts_with(fixture._temp.path()));
    std::fs::write(inventory_path, b"malformed: [").unwrap();
    std::fs::write(fixture.home.join("config.yaml"), b"malformed: [").unwrap();
    let token = std::env::var_os(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    let before = component_physical_tree_snapshot(&fixture.home);
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        cancel_ordinary_world_work_request(&source.selection, request.clone()),
    )
    .await;
    match token {
        Some(value) => std::env::set_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, value),
        None => std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN),
    }
    let WorldDispatchOutcomeV1::CancelWorldWork(empty) =
        result.expect("source-only Cancel test hang bound").unwrap()
    else {
        panic!("ordinary Cancel changed outcome family")
    };
    assert_eq!(
        empty.state,
        CancelWorldWorkTerminalStateV1::NoActiveCancelableWork
    );
    assert!(
        empty.cancel_request_id.is_none() && empty.terminal.is_none() && empty.closeout.is_none(),
        "no logical worker retirement or accepted cancellation is inferred"
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
        "empty receipt scope must send no service cancellation"
    );
    assert!(
        !fixture
            .home
            .join("run/agent-hub/world-work-execution-supervisor-v1.json")
            .exists(),
        "read/open cannot initialize supervisor mutation state"
    );
    for fault in ["caller", "world", "exact-acceptance"] {
        let mut wrong = request.clone();
        let expected = match fault {
            "caller" => {
                wrong.caller_participant_id = Some("foreign-current-caller".into());
                CancelWorldWorkTerminalStateV1::InvalidTarget
            }
            "world" => {
                wrong.world_generation = wrong.world_generation.map(|generation| generation + 1);
                CancelWorldWorkTerminalStateV1::WorldBindingMismatch
            }
            "exact-acceptance" => {
                let exact = WorldDispatchControlTargetV1::AcceptedRetainedTurn {
                    acceptance_record_id: format!("wwa_{}", uuid::Uuid::now_v7()),
                    active_run_id: "unissued-run".into(),
                    message_id: "unissued-message".into(),
                    target_participant_id: source
                        .e2_launch_activation
                        .retained_participant_id
                        .clone(),
                };
                wrong.target_participant_id = None;
                wrong.task_run_id = Some(format!(
                    "substrate_target_v1:{}",
                    serde_json::to_string(&exact).unwrap()
                ));
                CancelWorldWorkTerminalStateV1::InvalidTarget
            }
            _ => unreachable!(),
        };
        let before = component_physical_tree_snapshot(&fixture.home);
        if fault == "exact-acceptance" {
            use crate::execution::agent_runtime::tool_invocation_contract::{
                translate_host_tool_invocation_request_to_internal_dispatch_request_v1,
                HostToolInvocationRequestEnvelopeV1, HostToolRuntimeDispatchMetadataV1,
                HostToolRuntimeWorldBindingV1,
            };
            let store = crate::execution::agent_runtime::state_store::AgentRuntimeStateStore::new()
                .unwrap();
            assert!(
                store
                    .load_session(source.selection.original_session_id())
                    .unwrap()
                    .is_none(),
                "canonical ordinary control must not require a legacy session projection"
            );
            let exact: WorldDispatchControlTargetV1 = serde_json::from_str(
                wrong
                    .task_run_id
                    .as_ref()
                    .unwrap()
                    .strip_prefix("substrate_target_v1:")
                    .unwrap(),
            )
            .unwrap();
            let translated =
                translate_host_tool_invocation_request_to_internal_dispatch_request_v1(
                    &store,
                    &HostToolRuntimeDispatchMetadataV1 {
                        request_id: wrong.request_id.clone().unwrap(),
                        idempotency_key: wrong.idempotency_key.clone().unwrap(),
                        orchestration_session_id: source.selection.original_session_id().into(),
                        caller_participant_id: source.selection.current_participant_id().into(),
                    },
                    &HostToolRuntimeWorldBindingV1 {
                        world_id: source.e2_launch_activation.target_world.world_id.clone(),
                        world_generation: source.e2_launch_activation.target_world.world_generation,
                    },
                    HostToolInvocationRequestEnvelopeV1 {
                        version: 1,
                        tool_name: "cancel_world_work".into(),
                        tool_call_id: Some("unissued-accepted-control-input".into()),
                        arguments: serde_json::json!({"exact_target":exact,"payload":{}}),
                    },
                )
                .expect("canonical R0 supplies routing without a legacy projection");
            assert_eq!(
                translated.dispatch_request.task_run_id, wrong.task_run_id,
                "translation must preserve exact unissued IDs as input, not manufacture acceptance"
            );
            assert_eq!(
                translated.dispatch_request.target_backend_id,
                wrong.target_backend_id
            );
            assert!(translated.dispatch_request.target_participant_id.is_none());
            wrong = translated.dispatch_request;
        }
        let WorldDispatchOutcomeV1::CancelWorldWork(refused) =
            cancel_ordinary_world_work_request(&source.selection, wrong)
                .await
                .unwrap()
        else {
            panic!("ordinary refusal changed outcome family")
        };
        assert_eq!(refused.state, expected, "{fault}");
        assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
}

#[test]
fn test_ordinary_cancel_uses_exact_published_ephemeral_receipt() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*,
        state_store::AgentRuntimeStateStore,
        world_work_execution_supervisor::{
            OrdinaryCancellationDecisionV1, WorldWorkExecutionSupervisor,
        },
    };
    use crate::execution::orchestrator_world_dispatch::publish_ordinary_task_source_fixture_for_test;
    for fault in ["none", "root", "receipt", "e2", "e2-key", "policy"] {
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        let policy_path = fixture.home.join("policy.yaml");
        let policy = std::fs::read_to_string(&policy_path).unwrap()
        .replace("allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
            "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, run_world_task, cancel_world_work, inspect_world_worker]")
        .replace("allowed_modes: [retained]", "allowed_modes: [retained, ephemeral]\n    max_concurrent_ephemeral: 2");
        std::fs::write(&policy_path, policy).unwrap();
        let selected = Arc::new(
            OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(Arc::clone(
                &fixture.configured,
            ))
            .unwrap()
            .authenticate(&fixture.context.orchestration_session_id)
            .unwrap(),
        );
        let world = fixture.context.original_world_binding.as_ref().unwrap();
        let request = WorldDispatchRequestV1 {
            request_id: Some("req_cancel_ephemeral_source_fixture".into()),
            idempotency_key: Some("idem_cancel_ephemeral_source_fixture".into()),
            orchestration_session_id: Some(selected.original_session_id().into()),
            caller_participant_id: Some(selected.current_participant_id().into()),
            action: WorldDispatchActionV1::RunWorldTask,
            mode: WorldDispatchModeV1::Ephemeral,
            target_backend_id: Some(selected.requested_selection().backend_id.clone()),
            task_run_id: None,
            target_participant_id: None,
            world_id: Some(world.world_id.clone()),
            world_generation: Some(world.world_generation),
            dispatch_policy_narrowing: None,
            payload: WorldDispatchPayloadV1::Task(TaskPayloadV1 {
                prompt: "source publisher fixture; no native send".into(),
            }),
        };
        let (record, supervisor, claim, admission) = publish_ordinary_task_source_fixture_for_test(
            &AgentRuntimeStateStore::new().unwrap(),
            Arc::clone(&selected),
            request.clone().validate().unwrap(),
        )
        .unwrap();
        assert_eq!(record.runtime_acceptance.stream_id, claim.stream_id);
        let super::super::state_store::AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id } =
            &record.work_identity
        else {
            panic!("source family changed")
        };
        let mut cancel = request;
        cancel.request_id = Some("req_cancel_actual_source_owner".into());
        cancel.idempotency_key = Some("idem_cancel_actual_source_owner".into());
        cancel.action = WorldDispatchActionV1::CancelWorldWork;
        cancel.task_run_id = Some(task_run_id.clone());
        cancel.payload = WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default());
        let raw_cancel = cancel.clone();
        let cancel = cancel.validate().unwrap();
        if fault != "none" {
            use super::super::host_session_authority::store;
            let supervisor_path = fixture
                .home
                .join("run/agent-hub/world-work-execution-supervisor-v1.json");
            let before_bytes = std::fs::read(&supervisor_path).unwrap();
            let before = std::fs::metadata(&supervisor_path).unwrap();
            let fault_path = match fault {
                "root" => fixture.home.join("authority-v1/state-root-v1.json"),
                "receipt" => fixture
                    .home
                    .join("run/agent-hub/world-work-receipt-registry-v1.json"),
                "e2" => fixture
                    .home
                    .join("authority-v1/dispatch-policy-commitment-v1/registry-v1.json"),
                "e2-key" => {
                    let control = fixture
                        .authority
                        .capture_ordinary_supervisor_control_source_v1(
                            selected.original_session_id(),
                        )
                        .unwrap();
                    let store::DispatchPolicyCommitmentPhysicalReadV1::Present(e2) =
                        control.snapshot().e2()
                    else {
                        panic!("genuine E2 source absent")
                    };
                    fixture
                        .home
                        .join("authority-v1/dispatch-policy-commitment-v1/keys")
                        .join(e2.key_files().iter().next().unwrap().0)
                }
                "policy" => policy_path.clone(),
                _ => unreachable!(),
            };
            let fired = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let fired_by_hook = Arc::clone(&fired);
            store::set_ordinary_supervisor_before_publication_hook(move || {
                if fault == "policy" {
                    let policy = std::fs::read_to_string(&fault_path)
                        .unwrap()
                        .replace(", cancel_world_work", "");
                    std::fs::write(&fault_path, policy).unwrap();
                } else {
                    let next = fault_path.with_extension("replacement");
                    std::fs::write(&next, std::fs::read(&fault_path).unwrap()).unwrap();
                    std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600))
                        .unwrap();
                    std::fs::rename(next, fault_path).unwrap();
                }
                fired_by_hook.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            assert!(
                WorldWorkExecutionSupervisor::accept_or_join_ordinary_cancellation(
                    &selected, &cancel
                )
                .is_err(),
                "{fault} must refuse"
            );
            assert!(
                fired.load(std::sync::atomic::Ordering::SeqCst),
                "{fault} must reach actual prepublication boundary"
            );
            assert_eq!(
                before_bytes,
                std::fs::read(&supervisor_path).unwrap(),
                "{fault} cannot publish cancellation lease"
            );
            let after = std::fs::metadata(&supervisor_path).unwrap();
            assert_eq!(
                (
                    before.dev(),
                    before.ino(),
                    before.mtime(),
                    before.mtime_nsec(),
                    before.ctime(),
                    before.ctime_nsec()
                ),
                (
                    after.dev(),
                    after.ino(),
                    after.mtime(),
                    after.mtime_nsec(),
                    after.ctime(),
                    after.ctime_nsec()
                ),
                "{fault} cannot replace or rewrite consuming supervisor"
            );
            // The deliberate race writer and the owner's unpublished temporary are
            // distinct measured effects; this is not a zero-metadata-write claim.
            continue;
        }
        let OrdinaryCancellationDecisionV1::Accepted(prepared) =
            WorldWorkExecutionSupervisor::accept_or_join_ordinary_cancellation(&selected, &cancel)
                .unwrap()
        else {
            panic!("genuine accepted source must reach cancellation owner")
        };
        let (_, accepted_record, outcome) = prepared.into_parts();
        assert_eq!(accepted_record, record);
        assert_eq!(outcome.execution_claim, claim);
        assert!(outcome.should_deliver_transport && outcome.delivery_claim_id.is_some());
        assert!(outcome.terminal.is_none() && outcome.cancellation_pending);
        let before_retry = component_physical_tree_snapshot(&fixture.home);
        let OrdinaryCancellationDecisionV1::Accepted(retry) =
            WorldWorkExecutionSupervisor::accept_or_join_ordinary_cancellation(&selected, &cancel)
                .unwrap()
        else {
            panic!("exact retry must join existing cancellation")
        };
        let (_, retry_record, retry) = retry.into_parts();
        assert_eq!(retry_record, record);
        assert_eq!(
            retry.accepted_cancel_request_id,
            outcome.accepted_cancel_request_id
        );
        assert!(!retry.should_deliver_transport && retry.delivery_claim_id.is_none());
        assert_eq!(
            before_retry,
            component_physical_tree_snapshot(&fixture.home)
        );
        let observed = fixture
            .authority
            .observe_existing_ordinary_world_work_v1(&record)
            .unwrap();
        assert_eq!(observed.claim, claim);
        assert!(
            observed.terminal.is_none(),
            "delivery lease alone is never native terminal evidence"
        );
        assert!(supervisor
            .inspect_observation_by_acceptance_id(&record.acceptance_record_id)
            .unwrap()
            .unwrap()
            .terminal
            .is_none());
        let mut inspect = raw_cancel.clone();
        inspect.action = WorldDispatchActionV1::InspectWorldWorker;
        inspect.payload = WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default());
        let before_read = component_physical_tree_snapshot(&fixture.home);
        let WorldDispatchOutcomeV1::InspectWorldWorker(read) =
            crate::execution::orchestrator_world_dispatch::inspect_ordinary_world_worker_request(
                &selected,
                inspect.clone(),
            )
            .unwrap()
        else {
            panic!("Inspect family changed")
        };
        assert!(read.ephemeral_snapshot.as_ref().unwrap().authoritative_live);
        assert!(
            read.terminal.is_none() && read.bootstrap_snapshot.is_none() && read.snapshot.is_none()
        );
        assert_eq!(before_read, component_physical_tree_snapshot(&fixture.home));
        // The direct owner check above claimed, but did not send, transport. Return
        // that genuine claim through its existing confirmed-not-delivered producer.
        supervisor.record_cancellation_delivery_result(&claim, outcome.delivery_claim_id.as_deref().unwrap(),
        super::super::world_work_execution_supervisor::WorldWorkCancellationDeliveryResultV1::ConfirmedNotDelivered).unwrap();
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        use crate::execution::orchestrator_world_dispatch::cancel_ordinary_world_work_request;
        let socket = fixture._temp.path().join("cancel-source-transport.sock");
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let no_second_connection = listener.try_clone().unwrap();
        let listener = tokio::net::UnixListener::from_std(listener).unwrap();
        std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket);
        let server = async {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut wire = Vec::new();
            let header_end = loop {
                let mut byte = [0];
                stream.read_exact(&mut byte).await.unwrap();
                wire.push(byte[0]);
                assert!(wire.len() < 8192, "bounded private request header");
                if wire.ends_with(b"\r\n\r\n") { break wire.len(); }
            };
            let header = std::str::from_utf8(&wire).unwrap();
            assert!(header.starts_with("POST /v1/execute/cancel "));
            let length: usize = header.lines().find_map(|line| {
                let (key, value) = line.split_once(':')?;
                key.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().unwrap())
            }).unwrap();
            assert!(length < 8192);
            wire.resize(header_end + length, 0);
            stream.read_exact(&mut wire[header_end..]).await.unwrap();
            let sent: transport_api_types::ExecuteCancelRequestV1 = serde_json::from_slice(&wire[header_end..]).unwrap();
            assert_eq!(sent.span_id, claim.runtime_submission_id, "actual accepted service span, never bootstrap/run label");
            assert_eq!(sent.sig, "INT");
            // Acquiring actual Root while the request is at the peer proves
            // the control source scope did not carry its flock across IPC.
            let authority = Arc::clone(&fixture.authority);
            tokio::task::spawn_blocking(move || authority.read_preserved_start_root_v2()).await.unwrap().unwrap();
            let body = r#"{"schema_version":1,"delivered":true}"#;
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.shutdown().await.unwrap();
        };
        let route = crate::execution::orchestrator_world_dispatch::cancel_ordinary_world_work_request_using(
            &selected, raw_cancel.clone(), |wire| async {
                let transport_api_types::PlatformPrincipalV1::Unix { uid, .. } = selected.native_principal() else { panic!("fixture principal not Unix") };
                let before = component_physical_tree_snapshot(&fixture.home);
                for fault in ["peer", "lease", "acceptance", "span", "signal"] {
                    let mut changed = wire.clone();
                    let peer = if fault == "peer" { uid + 1 } else { *uid };
                    match fault {
                        "lease" => changed.delivery_claim_id = "unissued-lease".into(),
                        "acceptance" => changed.acceptance_record_id = "unissued-acceptance".into(),
                        "span" => changed.runtime_submission_id = "different-span".into(),
                        "signal" => changed.signal = "KILL".into(),
                        _ => {},
                    }
                    assert!(admission.with_ordinary_accepted_cancellation(&changed, peer,
                        || panic!("{fault} reached synthetic ephemeral SDK callback")).is_err(), "{fault}");
                }
                let called = std::cell::Cell::new(false);
                assert!(!admission.with_ordinary_accepted_cancellation(&wire, *uid, || {
                    called.set(true);
                    let lock = std::fs::OpenOptions::new().read(true).write(true)
                        .open(fixture.home.join("authority-v1/lock/root.lock"))?;
                    assert_eq!(unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&lock), libc::LOCK_EX | libc::LOCK_NB) }, -1);
                    assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::EWOULDBLOCK));
                    Ok(false) // Actual admission; no synthetic fixture SDK handle.
                }).expect("genuine ephemeral admission source refused"));
                assert!(called.get());
                assert_eq!(before, component_physical_tree_snapshot(&fixture.home), "ephemeral service source admission writes nothing");
                // Injected private HTTP peer, not the production Root connector.
                transport_api_client::AgentClient::unix_socket(&socket)?.cancel_execute(
                    transport_api_types::ExecuteCancelRequestV1 { span_id: wire.runtime_submission_id, sig: wire.signal }).await
            });
        let (_, outcome) = tokio::time::timeout(Duration::from_secs(30), async { tokio::join!(server, route) })
            .await.expect("private IPC/lease test hang bound");
        let WorldDispatchOutcomeV1::CancelWorldWork(delivered) = outcome.unwrap() else { panic!("cancel family changed") };
        assert_eq!(delivered.state, CancelWorldWorkTerminalStateV1::CancelAcceptedPendingCloseout);
        assert!(delivered.terminal.is_none() && delivered.closeout.is_none(), "HTTP delivery cannot manufacture checked native closeout");
        assert_eq!(delivered.cancel_request_id, Some(cancel.request_id.clone()));
        let WorldDispatchOutcomeV1::CancelWorldWork(retry) = cancel_ordinary_world_work_request(&selected, raw_cancel.clone()).await.unwrap()
            else { panic!("cancel family changed") };
        assert_eq!(retry.state, CancelWorldWorkTerminalStateV1::CancelAcceptedPendingCloseout);
        assert_eq!(retry.cancel_request_id, delivered.cancel_request_id);
        assert!(retry.terminal.is_none());
        assert_eq!(no_second_connection.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
        // Synthetic terminal observation, published only through the real
        // strict journal. This is source/transport classification proof only.
        let event_identity = substrate_common::agent_events::RuntimeEventIdentityV1 {
            event_id: "source_fixture_cancelled_terminal".into(), event_sequence: 1,
        };
        let terminal = transport_api_types::ExecuteStreamFrame::Exit {
            frame_identity: transport_api_types::RuntimeFrameIdentityV1 { schema_version: 1, stream_id: claim.stream_id.clone(), frame_sequence: 2 },
            terminal_identity: substrate_common::agent_events::RuntimeTerminalIdentityV1::from(&event_identity),
            event_identity, exit: 130, span_id: claim.runtime_submission_id.clone(), scopes_used: Vec::new(), fs_diff: None,
            process_telemetry: Default::default(),
        };
        supervisor.journal_frame(&claim, &terminal, &terminal.canonical_ndjson_bytes().unwrap()).unwrap();
        let WorldDispatchOutcomeV1::CancelWorldWork(terminal) = cancel_ordinary_world_work_request(&selected, raw_cancel).await.unwrap()
            else { panic!("cancel family changed") };
        assert_eq!(terminal.state, CancelWorldWorkTerminalStateV1::AlreadyTerminal);
        assert!(terminal.terminal.is_some() && terminal.closeout.is_none(), "accepted turn terminal is not worker retirement");
        assert_eq!(no_second_connection.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
        let before_read = component_physical_tree_snapshot(&fixture.home);
        let WorldDispatchOutcomeV1::InspectWorldWorker(read) = crate::execution::orchestrator_world_dispatch::inspect_ordinary_world_worker_request(&selected, inspect).unwrap()
            else { panic!("Inspect family changed") };
        assert!(!read.ephemeral_snapshot.as_ref().unwrap().authoritative_live);
        assert!(read.terminal.is_some() && read.bootstrap_snapshot.is_none() && read.snapshot.is_none());
        assert_eq!(before_read, component_physical_tree_snapshot(&fixture.home));
        // The accepted-control scope must expose its original Supervisor guard
        // before any callback effect. Equal bytes on a replacement inode cannot
        // substitute for that original physical capture.
        let fired = std::cell::Cell::new(false);
        let refused = selected.with_ordinary_accepted_control_sources_v1(|_, _, _, _, verify| {
            let path = fixture.home.join("run/agent-hub/world-work-execution-supervisor-v1.json");
            let replacement = path.with_extension("source-control-replacement");
            std::fs::write(&replacement, std::fs::read(&path).unwrap()).unwrap();
            std::fs::set_permissions(&replacement, std::fs::Permissions::from_mode(0o600)).unwrap();
            std::fs::rename(replacement, path).unwrap();
            fired.set(true);
            assert!(verify().is_err(), "same-byte Supervisor substitution must fail before an accepted-control effect");
            Ok(())
        });
        assert!(fired.get() && refused.is_err(), "semantic success still runs the original physical finalizers");
    });
    }
}

async fn read_private_ordinary_http_request_for_test(
    stream: &mut tokio::net::UnixStream,
) -> (String, Vec<u8>) {
    use tokio::io::AsyncReadExt as _;
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        stream.read_exact(&mut byte).await.unwrap();
        header.push(byte[0]);
        assert!(header.len() < 8192);
    }
    let header = String::from_utf8(header).unwrap();
    let length: usize = header
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().unwrap())
        })
        .unwrap();
    assert!(length < 1024 * 1024);
    let mut body = vec![0; length];
    stream.read_exact(&mut body).await.unwrap();
    (header, body)
}

#[tokio::test(flavor = "current_thread")]
async fn test_ordinary_retained_acceptance_inspect_cancel_uses_actual_producers() {
    ordinary_retained_producer_control_case(false).await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_ordinary_stop_cancellation_uses_existing_supervisor_owner() {
    ordinary_retained_producer_control_case(true).await;
}

async fn ordinary_retained_producer_control_case(stop_first: bool) {
    use crate::execution::agent_runtime::{
        dispatch_contract::*, state_store::AgentRuntimeStateStore,
    };
    use crate::execution::orchestrator_world_dispatch::{
        cancel_ordinary_world_work_request, continue_ordinary_world_worker,
        inspect_ordinary_world_worker_request,
    };
    use tokio::io::AsyncWriteExt as _;
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap().replace(
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, inspect_world_worker, cancel_world_work, stop_world_worker]");
    std::fs::write(&policy_path, policy).unwrap();
    let socket = fixture._temp.path().join("retained-source-peer.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let no_second_connection = listener.try_clone().unwrap();
    let listener = tokio::net::UnixListener::from_std(listener).unwrap();
    std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket);
    let target = source.e2_launch_activation.retained_participant_id.clone();
    let session = source.selection.original_session_id().to_string();
    let caller = source.selection.current_participant_id().to_string();
    let backend = source.selection.requested_selection().backend_id.clone();
    let world = source.e2_launch_activation.target_world.clone();
    let expected_target = target.clone();
    let expected_session = session.clone();
    let (release_terminal, terminal_release) = tokio::sync::oneshot::channel();
    let peer = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let (header, body) = read_private_ordinary_http_request_for_test(&mut stream).await;
        assert!(header.starts_with("POST /v1/ordinary/member_turn/stream "));
        let submitted: transport_api_types::OrdinaryMemberTurnSubmitRequestV1 =
            serde_json::from_slice(&body).unwrap();
        let submitted = submitted.into_request();
        assert_eq!(submitted.participant_id, expected_target);
        assert_eq!(submitted.orchestration_session_id, expected_session);
        assert!(
            submitted.acceptance_context.is_some() && submitted.policy_snapshot_carrier.is_some()
        );
        let span = "source_fixture_retained_actual_service_span";
        let stream_id = "rts_source_fixture_retained_actual_stream";
        let start = transport_api_types::ExecuteStreamFrame::Start {
            frame_identity: transport_api_types::RuntimeFrameIdentityV1 {
                schema_version: 1,
                stream_id: stream_id.into(),
                frame_sequence: 1,
            },
            span_id: span.into(),
        }
        .canonical_ndjson_bytes()
        .unwrap();
        let event_identity = substrate_common::agent_events::RuntimeEventIdentityV1 {
            event_id: "source_fixture_retained_cancelled_event".into(),
            event_sequence: 1,
        };
        let terminal = transport_api_types::ExecuteStreamFrame::Exit {
            frame_identity: transport_api_types::RuntimeFrameIdentityV1 {
                schema_version: 1,
                stream_id: stream_id.into(),
                frame_sequence: 2,
            },
            terminal_identity: substrate_common::agent_events::RuntimeTerminalIdentityV1::from(
                &event_identity,
            ),
            event_identity,
            exit: 130,
            span_id: span.into(),
            scopes_used: Vec::new(),
            fs_diff: None,
            process_telemetry: Default::default(),
        }
        .canonical_ndjson_bytes()
        .unwrap();
        let headers = format!("HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", start.len() + terminal.len());
        stream.write_all(headers.as_bytes()).await.unwrap();
        stream.write_all(&start).await.unwrap();
        stream.flush().await.unwrap();
        let (mut cancel, _) = listener.accept().await.unwrap();
        let (header, body) = read_private_ordinary_http_request_for_test(&mut cancel).await;
        assert!(header.starts_with("POST /v1/execute/cancel "));
        let request: transport_api_types::ExecuteCancelRequestV1 =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(request.span_id, span);
        let body = r#"{"schema_version":1,"delivered":true}"#;
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        cancel.write_all(response.as_bytes()).await.unwrap();
        cancel.shutdown().await.unwrap();
        terminal_release.await.unwrap();
        stream.write_all(&terminal).await.unwrap();
        stream.shutdown().await.unwrap();
    });
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_actual_retained_acceptance_source".into()),
        idempotency_key: Some("idem_actual_retained_acceptance_source".into()),
        orchestration_session_id: Some(session),
        caller_participant_id: Some(caller),
        action: WorldDispatchActionV1::ContinueWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(backend),
        task_run_id: None,
        target_participant_id: Some(target),
        world_id: Some(world.world_id),
        world_generation: Some(world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
            prompt: "source observer fixture; no native runtime".into(),
            thread_id: None,
        }),
    };
    let carrier: transport_api_types::InstallBootstrapContextCarrierV1 =
        serde_json::from_slice(&std::fs::read(fixture._temp.path().join("carrier.json")).unwrap())
            .unwrap();
    let result = tokio::time::timeout(
        Duration::from_secs(45),
        continue_ordinary_world_worker(
            &AgentRuntimeStateStore::new().unwrap(),
            Arc::clone(&source.selection),
            carrier,
            request.clone(),
        ),
    )
    .await
    .expect("source producer/peer test hang bound")
    .unwrap();
    let WorldDispatchOutcomeV1::AcceptedForeground(receipt) = result else {
        panic!("Continue did not produce actual B1/E2 acceptance")
    };
    let AcceptedForegroundReceiptV1::Retained(receipt) = *receipt else {
        panic!("Continue acceptance changed family")
    };
    let exact = WorldDispatchControlTargetV1::AcceptedRetainedTurn {
        acceptance_record_id: receipt.acceptance_record_id.clone(),
        active_run_id: receipt.active_run_id.clone(),
        message_id: receipt.message_id.clone(),
        target_participant_id: receipt.target_participant_id.clone(),
    };
    let mut stop = request.clone();
    stop.request_id = Some("req_stop_genuine_accepted_cut".into());
    stop.idempotency_key = Some("idem_stop_genuine_accepted_cut".into());
    stop.action = WorldDispatchActionV1::StopWorldWorker;
    stop.payload = WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default());
    let stop = stop.validate().unwrap();
    let prepared_stop = super::super::retained_worker_runtime::RetainedWorkerRuntime
        .prepare_ordinary_stop_intent_v1(&source.selection, &stop)
        .unwrap();
    assert_eq!(
        prepared_stop
            .active_acceptance_record()
            .unwrap()
            .acceptance_record_id,
        receipt.acceptance_record_id,
        "Stop intent must capture actual accepted-turn producer identity"
    );
    let reader = crate::OrdinaryRetainedStopControlReaderV1::from_bootstrap(
        Arc::clone(&source.selection),
        source.e2_launch_activation.clone(),
    )
    .unwrap();
    let transport_api_types::PlatformPrincipalV1::Unix { uid, .. } =
        source.selection.native_principal()
    else {
        panic!("native principal not Unix")
    };
    let before = component_physical_tree_snapshot(&fixture.home);
    let observed = reader
        .with_locked_retirement(prepared_stop.request(), *uid, |_| {
            panic!("active accepted turn reached retirement callback")
        })
        .unwrap();
    assert!(matches!(
        observed,
        transport_api_types::OrdinaryRetainedWorkerStopResponseV1::NotQuiescent {
            reason: transport_api_types::OrdinaryRetainedWorkerStopBusyReasonV1::AcceptedTurn
        }
    ));
    assert_eq!(
        before,
        component_physical_tree_snapshot(&fixture.home),
        "busy Stop read changes no durable source"
    );
    let mut control = request;
    control.target_participant_id = None;
    control.task_run_id = Some(format!(
        "substrate_target_v1:{}",
        serde_json::to_string(&exact).unwrap()
    ));
    control.action = WorldDispatchActionV1::InspectWorldWorker;
    control.payload = WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default());
    let before = component_physical_tree_snapshot(&fixture.home);
    let WorldDispatchOutcomeV1::InspectWorldWorker(inspect) =
        inspect_ordinary_world_worker_request(&source.selection, control.clone()).unwrap()
    else {
        panic!("Inspect changed family")
    };
    assert!(inspect.terminal.is_none() && inspect.snapshot.is_none());
    let stop_read = inspect
        .stop_snapshot
        .as_ref()
        .expect("actual MACed Stop intent must be visible independently from bootstrap");
    assert_eq!(&stop_read.request, prepared_stop.request());
    assert_eq!(
        stop_read.accepted_work_ids_at_admission,
        vec![receipt.acceptance_record_id.clone()]
    );
    assert!(
        stop_read.acknowledgment.is_none() && stop_read.completed_at.is_none(),
        "bootstrap completion cannot imply logical Stop ACK"
    );
    let normalized =
        super::super::tool_invocation_contract::normalize_inspect_world_worker_outcome_v1(&inspect)
            .unwrap();
    assert_eq!(
        normalized.stop_snapshot.as_ref().unwrap().as_ref(),
        stop_read.as_ref()
    );
    for fault in ["schema", "extra"] {
        let mut wire = serde_json::to_value(stop_read).unwrap();
        if fault == "schema" {
            wire["schema_version"] = 2.into();
        } else {
            wire["native_live"] = true.into();
        }
        assert!(
            serde_json::from_value::<RetainedStopInspectSnapshotV1>(wire).is_err(),
            "{fault}"
        );
    }
    let mut wrong_scope = inspect.clone();
    wrong_scope
        .stop_snapshot
        .as_mut()
        .unwrap()
        .request
        .world_generation += 1;
    assert!(
        super::super::tool_invocation_contract::normalize_inspect_world_worker_outcome_v1(
            &wrong_scope
        )
        .is_err()
    );

    assert_eq!(
        inspect
            .bootstrap_snapshot
            .as_ref()
            .unwrap()
            .registration_id
            .as_deref(),
        source
            .authority
            .read_a12b_root()
            .unwrap()
            .retained_worker_registration_journal
            .values()
            .next()
            .map(|row| row.registration_id.as_str())
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    control.request_id = Some("req_exact_retained_cancel_source".into());
    control.idempotency_key = Some("idem_exact_retained_cancel_source".into());
    control.action = WorldDispatchActionV1::CancelWorldWork;
    control.payload = WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default());
    let actual_runtime_fixture =
        transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id: receipt.orchestration_session_id.clone(),
            caller_participant_id: prepared_stop
                .active_acceptance_record()
                .unwrap()
                .caller_participant_id
                .clone(),
            participant_id: receipt.target_participant_id.clone(),
            backend_id: receipt.target_backend_id.clone(),
            run_id: receipt.active_run_id.clone(),
            world_id: receipt.world_id.clone(),
            world_generation: receipt.world_generation,
        };
    if stop_first {
        use super::super::world_work_execution_supervisor::{
            with_ordinary_retained_cancel_admission_v1, OrdinaryCancellationDecisionV1,
            WorldWorkCancellationDeliveryResultV1, WorldWorkExecutionSupervisor,
        };
        let allowed = std::fs::read_to_string(&policy_path).unwrap();
        std::fs::write(&policy_path, allowed.replace(", cancel_world_work", "")).unwrap();
        let OrdinaryCancellationDecisionV1::Accepted(accepted) =
            WorldWorkExecutionSupervisor::accept_or_join_ordinary_stop_cancellation(
                &source.selection,
                &prepared_stop,
            )
            .expect("actual Stop producer must not require Cancel permission")
        else {
            panic!("actual Stop failed to produce existing Supervisor lease")
        };
        let (supervisor, acceptance, cancellation) = accepted.into_parts();
        assert_eq!(
            cancellation.accepted_cancel_request_id.as_deref(),
            Some(prepared_stop.request().stop_request_id.as_str())
        );
        let wire = transport_api_types::OrdinaryAcceptedWorkCancelRequestV1 {
            schema_version: 1,
            authority_store_id: acceptance.authority_store_id.clone(),
            orchestration_session_id: acceptance.orchestration_session_id.clone(),
            acceptance_record_id: acceptance.acceptance_record_id.clone(),
            cancel_request_id: cancellation.accepted_cancel_request_id.clone().unwrap(),
            delivery_claim_id: cancellation
                .delivery_claim_id
                .clone()
                .expect("fresh Stop must claim delivery"),
            runtime_submission_id: cancellation.execution_claim.runtime_submission_id.clone(),
            target_backend_id: acceptance.target_backend_id.clone(),
            world_id: acceptance.world_id.clone(),
            world_generation: acceptance.world_generation,
            signal: "INT".into(),
            control: transport_api_types::OrdinaryAcceptedWorkCancelControlV1::StopWorker {
                stop: Box::new(prepared_stop.request().clone()),
            },
        };
        let before = component_physical_tree_snapshot(&fixture.home);
        assert!(!with_ordinary_retained_cancel_admission_v1(
            &source.selection,
            &wire,
            &actual_runtime_fixture,
            *uid,
            || Ok(false)
        )
        .unwrap());
        assert_eq!(
            before,
            component_physical_tree_snapshot(&fixture.home),
            "Stop service source read cannot publish or assert native delivery"
        );
        let OrdinaryCancellationDecisionV1::Accepted(joined) =
            WorldWorkExecutionSupervisor::accept_or_join_ordinary_stop_cancellation(
                &source.selection,
                &prepared_stop,
            )
            .unwrap()
        else {
            panic!("Stop retry did not join original owner")
        };
        let (_, _, joined) = joined.into_parts();
        assert_eq!(
            joined.accepted_cancel_request_id,
            cancellation.accepted_cancel_request_id
        );
        assert!(
            joined.delivery_claim_id.is_none(),
            "Stop retry cannot duplicate a live delivery lease"
        );
        assert_eq!(
            before,
            component_physical_tree_snapshot(&fixture.home),
            "exact Stop lease join writes nothing"
        );
        // No native delivery occurred in this source probe. Return its genuine
        // lease through the existing owner before the synthetic transport case.
        supervisor
            .record_cancellation_delivery_result(
                &cancellation.execution_claim,
                &wire.delivery_claim_id,
                WorldWorkCancellationDeliveryResultV1::ConfirmedNotDelivered,
            )
            .unwrap();
        std::fs::write(&policy_path, allowed).unwrap();
    }
    // Genuine B1/E2/Supervisor producer sources, with synthetic service/native
    // delivery. This does not exercise the installed Root connector or SDK.
    let WorldDispatchOutcomeV1::CancelWorldWork(cancel) = tokio::time::timeout(Duration::from_secs(30),
        crate::execution::orchestrator_world_dispatch::cancel_ordinary_world_work_request_using(
            &source.selection, control.clone(), |wire| async {
                use super::super::world_work_execution_supervisor::with_ordinary_retained_cancel_admission_v1 as admit;
                let before = component_physical_tree_snapshot(&fixture.home);
                for fault in ["peer", "lease", "cancel", "span", "acceptance"] {
                    let mut changed = wire.clone();
                    let peer = if fault == "peer" { uid + 1 } else { *uid };
                    match fault {
                        "lease" => changed.delivery_claim_id = "unissued-lease".into(),
                        "cancel" => changed.cancel_request_id = "unissued-cancel".into(),
                        "span" => changed.runtime_submission_id = "different-span".into(),
                        "acceptance" => changed.acceptance_record_id = "unissued-acceptance".into(),
                        _ => {},
                    }
                    assert!(admit(&source.selection, &changed, &actual_runtime_fixture, peer,
                        || panic!("{fault} reached synthetic cancellation callback")).is_err(), "{fault}");
                }
                let calls = std::cell::Cell::new(0);
                assert!(!admit(&source.selection, &wire, &actual_runtime_fixture, *uid, || {
                    calls.set(calls.get() + 1);
                    let lock = std::fs::OpenOptions::new().read(true).write(true)
                        .open(fixture.home.join("authority-v1/lock/root.lock"))?;
                    assert_eq!(unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&lock), libc::LOCK_EX | libc::LOCK_NB) }, -1);
                    assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::EWOULDBLOCK));
                    Ok(false) // Source probe has no actual native handle.
                }).expect("genuine service source admission refused"));
                assert_eq!(calls.get(), 1);
                assert_eq!(before, component_physical_tree_snapshot(&fixture.home), "service admission changes no durable source");
                let allowed = std::fs::read_to_string(&policy_path)?;
                std::fs::write(&policy_path, allowed.replace(", cancel_world_work", ""))?;
                assert!(admit(&source.selection, &wire, &actual_runtime_fixture, *uid,
                    || panic!("revoked Cancel reached synthetic callback")).is_err());
                let mut stop_wire = wire.clone();
                stop_wire.control = transport_api_types::OrdinaryAcceptedWorkCancelControlV1::StopWorker {
                    stop: Box::new(prepared_stop.request().clone()),
                };
                let stop_result = admit(&source.selection, &stop_wire, &actual_runtime_fixture, *uid, || Ok(false));
                std::fs::write(&policy_path, &allowed)?;
                assert!(!stop_result.expect("genuine Stop cancellation source admission refused"), "admitted Stop uses Stop policy independently from Cancel policy");
                // Explicit source-fixture transport injection. The public route
                // uses the strict installed Root connector and cannot use this.
                transport_api_client::AgentClient::unix_socket(&socket)?.cancel_execute(
                    transport_api_types::ExecuteCancelRequestV1 { span_id: wire.runtime_submission_id, sig: wire.signal }).await
            })).await.expect("cancel source peer hang bound").unwrap()
        else { panic!("Cancel changed family") };
    assert_eq!(
        cancel.state,
        CancelWorldWorkTerminalStateV1::CancelAcceptedPendingCloseout
    );
    assert!(cancel.terminal.is_none() && cancel.closeout.is_none());
    let WorldDispatchOutcomeV1::CancelWorldWork(retry) =
        cancel_ordinary_world_work_request(&source.selection, control.clone())
            .await
            .unwrap()
    else {
        panic!("Cancel changed family")
    };
    assert_eq!(retry.cancel_request_id, cancel.cancel_request_id);
    assert_eq!(
        no_second_connection.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    release_terminal.send(()).unwrap();
    peer.await.unwrap();
    source
        .selection
        .join_issued_world_observers()
        .await
        .unwrap();
    let WorldDispatchOutcomeV1::CancelWorldWork(terminal) =
        cancel_ordinary_world_work_request(&source.selection, control.clone())
            .await
            .unwrap()
    else {
        panic!("Cancel changed family")
    };
    assert_eq!(
        terminal.state,
        CancelWorldWorkTerminalStateV1::AlreadyTerminal
    );
    assert!(terminal.terminal.is_some() && terminal.closeout.is_none());
    let before = component_physical_tree_snapshot(&fixture.home);
    let observed = reader
        .with_locked_retirement(prepared_stop.request(), *uid, |admission| {
            admission.revalidate_before_retirement()?;
            assert!(admission.existing_acknowledgment().is_none());
            Ok(transport_api_types::OrdinaryRetainedWorkerStopResponseV1::UnknownOwner {})
        })
        .unwrap();
    assert!(
        matches!(
            observed,
            transport_api_types::OrdinaryRetainedWorkerStopResponseV1::UnknownOwner {}
        ),
        "checked accepted terminal permits owner observation, never invents native retirement"
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    control.action = WorldDispatchActionV1::InspectWorldWorker;
    control.payload = WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default());
    let before = component_physical_tree_snapshot(&fixture.home);
    let WorldDispatchOutcomeV1::InspectWorldWorker(terminal) =
        inspect_ordinary_world_worker_request(&source.selection, control).unwrap()
    else {
        panic!("Inspect changed family")
    };
    assert!(
        terminal.terminal.is_some()
            && terminal
                .bootstrap_snapshot
                .as_ref()
                .unwrap()
                .native_bootstrap_completed
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    assert_eq!(
        no_second_connection.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    // HTTP frames are synthetic: this qualifies source producers and routing,
    // never native execution, settlement acknowledgment, kernel or installed use.
}

#[test]
fn test_ordinary_admission_control_source_allows_only_owned_publication() {
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let before = component_physical_tree_snapshot(&fixture.home);
    let held = source
        .authority
        .capture_ordinary_retained_lifecycle_control_source_v1(
            source.selection.original_session_id(),
            &source.e2_launch_activation.retained_participant_id,
        )
        .unwrap();
    assert_eq!(
        held.facts.current.caller.participant_id,
        source.selection.current_participant_id()
    );
    assert_eq!(
        held.target_graph
            .as_ref()
            .unwrap()
            .registration
            .retained_participant_id,
        source.e2_launch_activation.retained_participant_id
    );
    assert_eq!(
        held.snapshot().authority_store_id(),
        source.selection.authority_store_id()
    );
    assert!(held.supervisor_bytes().is_none());
    held.verify_before_publication(None).unwrap();
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
    // The data holder releases Root and never grants this fixture mutation.
    source.authority.read_preserved_start_root_v2().unwrap();
    let strict_admission_reader = source
        .authority
        .capture_ordinary_supervisor_control_source_v1(source.selection.original_session_id())
        .unwrap();
    let directory = fixture
        .home
        .join("authority-v1/retained-worker-admission-v1");
    let temp_name = format!("admission-registry--{}.tmp", "a".repeat(32));
    let temp = directory.join("tmp").join(&temp_name);
    let original = held.admission_registry_bytes().to_vec();
    // Deliberate fixture-owned publication of unchanged existing data, not an
    // issued Stop, retirement acknowledgment, policy grant or native effect.
    std::fs::write(&temp, &original).unwrap();
    std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o600)).unwrap();
    let staged = component_physical_tree_snapshot(&fixture.home);
    assert!(held.verify_before_publication(None).is_err());
    assert!(held
        .verify_before_publication(Some(&format!("admission-registry--{}.tmp", "b".repeat(32))))
        .is_err());
    held.verify_before_publication(Some(&temp_name)).unwrap();
    assert_eq!(staged, component_physical_tree_snapshot(&fixture.home));
    std::fs::rename(&temp, directory.join("registry-v1.json")).unwrap();
    let published = component_physical_tree_snapshot(&fixture.home);
    assert!(
        held.verify_before_publication(None).is_err(),
        "original input predicate cannot be rebound after replacement"
    );
    held.verify_after_publication(&original).unwrap();
    assert!(held.verify_after_publication(b"different output").is_err());
    assert!(
        strict_admission_reader.verify_sources().is_err(),
        "existing admission-consuming read custody must still reject admission namespace change"
    );
    assert_eq!(
        published,
        component_physical_tree_snapshot(&fixture.home),
        "verification performs no writes"
    );
}

#[test]
fn test_ordinary_admission_control_source_retains_original_inputs() {
    for fault in [
        "root",
        "receipt",
        "e2",
        "e2-key",
        "admission",
        "admission-key",
        "supervisor",
        "typed-object",
    ] {
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
        let supervisor_path = fixture
            .home
            .join("run/agent-hub/world-work-execution-supervisor-v1.json");
        if fault == "supervisor" {
            // Negative physical-source fixture only. Its deliberately malformed
            // bytes grant no supervisor truth; an operation must validate them.
            std::fs::write(&supervisor_path, b"negative physical source fixture only").unwrap();
            std::fs::set_permissions(&supervisor_path, std::fs::Permissions::from_mode(0o600))
                .unwrap();
        }
        let held = source
            .authority
            .capture_ordinary_retained_lifecycle_control_source_v1(
                source.selection.original_session_id(),
                &source.e2_launch_activation.retained_participant_id,
            )
            .unwrap();
        let path = match fault {
            "root" => fixture.home.join("authority-v1/state-root-v1.json"),
            "receipt" => fixture
                .home
                .join("run/agent-hub/world-work-receipt-registry-v1.json"),
            "e2" => fixture
                .home
                .join("authority-v1/dispatch-policy-commitment-v1/registry-v1.json"),
            "e2-key" => {
                let super::super::host_session_authority::store::DispatchPolicyCommitmentPhysicalReadV1::Present(e2) = held.snapshot().e2() else { panic!("actual E2 missing") };
                fixture
                    .home
                    .join("authority-v1/dispatch-policy-commitment-v1/keys")
                    .join(e2.key_files().iter().next().unwrap().0)
            }
            "admission" => fixture
                .home
                .join("authority-v1/retained-worker-admission-v1/registry-v1.json"),
            "admission-key" => fixture
                .home
                .join("authority-v1/retained-worker-admission-v1/keys")
                .join(&held.admission_key_files()[0].0),
            "supervisor" => {
                assert_eq!(
                    held.supervisor_bytes(),
                    Some(b"negative physical source fixture only".as_slice())
                );
                supervisor_path
            }
            "typed-object" => fixture.home.join(format!(
                "authority-v1/objects/host-attach-contract/v1/{}.obj",
                held.facts.original_start.host_attach_contract_ref.ref_id
            )),
            _ => unreachable!(),
        };
        let next = path.with_extension("replacement");
        std::fs::write(&next, std::fs::read(&path).unwrap()).unwrap();
        std::fs::set_permissions(&next, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::rename(next, &path).unwrap();
        let after_fault = component_physical_tree_snapshot(&fixture.home);
        assert!(
            held.verify_before_publication(None).is_err(),
            "original {fault} source must refuse"
        );
        assert_eq!(after_fault, component_physical_tree_snapshot(&fixture.home));
    }
}

#[test]
fn test_ordinary_existing_admission_control_storage_never_reconciles() {
    use super::super::host_session_authority::store;
    for fault in ["none", "admission-temp", "key-temp", "authority-temp"] {
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
        let held = source
            .authority
            .capture_ordinary_retained_lifecycle_control_source_v1(
                source.selection.original_session_id(),
                &source.e2_launch_activation.retained_participant_id,
            )
            .unwrap();
        let directory = fixture
            .home
            .join("authority-v1/retained-worker-admission-v1");
        let temp_name = format!("admission-registry--{}.tmp", "c".repeat(32));
        let injected = match fault {
            "none" => None,
            "admission-temp" => Some(directory.join("tmp").join(&temp_name)),
            "key-temp" => Some(
                directory
                    .join("keys")
                    .join(format!("admission-key--{}.tmp", "d".repeat(32))),
            ),
            "authority-temp" => Some(
                fixture
                    .home
                    .join("authority-v1/tmp/unrecognized-control-source"),
            ),
            _ => unreachable!(),
        };
        if let Some(path) = &injected {
            std::fs::write(path, b"deliberate pending source fixture; never promoted").unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        let before = component_physical_tree_snapshot(&fixture.home);
        let storage =
            store::retained_worker_admission_storage_existing_for_authority(&source.authority);
        assert_eq!(
            before,
            component_physical_tree_snapshot(&fixture.home),
            "{fault}: storage bind cannot repair/create/reconcile"
        );
        if fault == "none" {
            let storage = storage.unwrap();
            let original = held.admission_registry_bytes().to_vec();
            // Actual existing-only storage publisher, with deliberately unchanged
            // data in a fixture. This is no Stop grant or retirement producer.
            storage
                .transaction_existing_control(
                    &|| {
                        held.verify_before_publication(None)
                            .map_err(|_| store::BootstrapError::ordinary_control_source_guard())
                    },
                    |transaction| {
                        assert_eq!(
                            transaction.authority_root().authority_store_id(),
                            held.snapshot().authority_store_id()
                        );
                        assert_eq!(
                            transaction.read_registry()?.as_deref(),
                            Some(original.as_slice())
                        );
                        transaction.replace_registry_with_source_guard(
                            &temp_name,
                            &original,
                            &|| {
                                held.verify_before_publication(Some(&temp_name))
                                    .map_err(|_| {
                                        store::BootstrapError::ordinary_control_source_guard()
                                    })
                            },
                        )
                    },
                )
                .unwrap();
            held.verify_after_publication(&original).unwrap();
            assert!(!directory.join("tmp").join(&temp_name).exists());
        } else {
            if let Ok(storage) = storage {
                // Probe the storage refusal itself as well as source guards:
                // no caller operation or mutation may run on pending storage.
                let entered = std::cell::Cell::new(false);
                let result = storage.transaction_existing_control(&|| Ok(()), |_| {
                    entered.set(true);
                    Err::<(), _>(store::BootstrapError::ordinary_control_source_guard())
                });
                assert!(
                    result.is_err() && !entered.get(),
                    "{fault}: existing-only storage must refuse before operation"
                );
            }
            assert_eq!(
                before,
                component_physical_tree_snapshot(&fixture.home),
                "{fault}: refusal must preserve all pending evidence"
            );
            assert!(injected.unwrap().exists());
        }
    }
}

#[test]
fn test_ordinary_stop_intent_uses_genuine_sources_and_fresh_policy() {
    use super::super::{dispatch_contract::*, retained_worker_runtime::*};
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let socket = fixture._temp.path().join("unused-stop-intent.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, report) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_ordinary_stop_actual_intent".into()),
        idempotency_key: Some("idem_ordinary_stop_actual_intent".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::StopWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.selection.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(source.e2_launch_activation.retained_participant_id.clone()),
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
    }
    .validate()
    .unwrap();
    let before_denial = component_physical_tree_snapshot(&fixture.home);
    assert!(RetainedWorkerRuntime
        .prepare_ordinary_stop_intent_v1(&source.selection, &request)
        .is_err());
    assert_eq!(
        before_denial,
        component_physical_tree_snapshot(&fixture.home),
        "read custody cannot grant Stop"
    );
    let policy_path = fixture.home.join("policy.yaml");
    let original_policy = std::fs::read_to_string(&policy_path).unwrap();
    std::fs::write(&policy_path, original_policy.replace(
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, stop_world_worker]")).unwrap();
    let inventory_path = fixture
        .context
        .sources
        .inventory
        .get(&source.selection.requested_selection().realized_agent_id)
        .unwrap()
        .path
        .clone();
    let inventory_bytes = std::fs::read(&inventory_path).unwrap();
    let config_path = fixture.home.join("config.yaml");
    let config_bytes = std::fs::read(&config_path).unwrap();
    std::fs::write(&inventory_path, b"malformed: [").unwrap();
    std::fs::write(&config_path, b"malformed: [").unwrap();
    let guarded_paths = [
        "authority-v1/state-root-v1.json",
        "authority-v1/dispatch-policy-commitment-v1/registry-v1.json",
        "run/agent-hub/world-work-receipt-registry-v1.json",
    ];
    let guarded_bytes = guarded_paths
        .iter()
        .map(|path| std::fs::read(fixture.home.join(path)).unwrap())
        .collect::<Vec<_>>();
    let token = std::env::var_os(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    let result = RetainedWorkerRuntime.prepare_ordinary_stop_intent_v1(&source.selection, &request);
    match token {
        Some(value) => std::env::set_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, value),
        None => std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN),
    }
    let prepared = result.unwrap();
    assert_eq!(prepared.request().stop_request_id, request.request_id);
    assert_eq!(
        prepared.request().native_session_id,
        report.observation().native_session_id
    );
    assert_eq!(
        prepared.request().worker_cap_ref,
        source.e2_launch_activation.immutable_worker_cap_ref
    );
    assert!(
        prepared.acknowledgment().is_none() && prepared.active_acceptance_record().is_none(),
        "intent is neither service ACK nor active/native evidence"
    );
    for (path, before) in guarded_paths.iter().zip(&guarded_bytes) {
        assert_eq!(
            before,
            &std::fs::read(fixture.home.join(path)).unwrap(),
            "Stop intent cannot alter {path}"
        );
    }
    assert!(std::fs::read_dir(
        fixture
            .home
            .join("authority-v1/retained-worker-admission-v1/tmp")
    )
    .unwrap()
    .next()
    .is_none());
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
        "intent preparation makes no service/native request"
    );
    let reader = OrdinaryRetainedStopControlReaderV1::from_bootstrap(
        Arc::clone(&source.selection),
        source.e2_launch_activation.clone(),
    )
    .unwrap();
    let transport_api_types::PlatformPrincipalV1::Unix { uid, .. } =
        source.selection.native_principal()
    else {
        panic!("fixture principal not Unix")
    };
    let before_control = component_physical_tree_snapshot(&fixture.home);
    let control_called = std::cell::Cell::new(false);
    let control = reader
        .with_locked_retirement(prepared.request(), *uid, |admission| {
            control_called.set(true);
            admission.revalidate_before_retirement()?;
            assert_eq!(admission.request(), prepared.request());
            assert!(admission.existing_acknowledgment().is_none());
            let lock = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(fixture.home.join("authority-v1/lock/root.lock"))
                .unwrap();
            // SAFETY: flock borrows a valid fixture-owned descriptor and attempts
            // a nonblocking lock. Failure must be the already-held Root lock.
            let result = unsafe {
                libc::flock(
                    std::os::fd::AsRawFd::as_raw_fd(&lock),
                    libc::LOCK_EX | libc::LOCK_NB,
                )
            };
            assert_eq!(
                result, -1,
                "retirement callback must retain the Root publication boundary"
            );
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::EWOULDBLOCK)
            );
            Ok(transport_api_types::OrdinaryRetainedWorkerStopResponseV1::UnknownOwner {})
        })
        .unwrap();
    assert!(control_called.get());
    assert!(
        matches!(
            control,
            transport_api_types::OrdinaryRetainedWorkerStopResponseV1::UnknownOwner {}
        ),
        "source admission cannot manufacture actual native owner/retirement"
    );
    assert_eq!(
        before_control,
        component_physical_tree_snapshot(&fixture.home),
        "locked Stop control read writes nothing"
    );
    for fault in ["peer", "native", "intent"] {
        let mut changed = prepared.request().clone();
        let peer_uid = if fault == "peer" { uid + 1 } else { *uid };
        if fault == "native" {
            changed.native_session_id = "different-native-session".into();
        }
        if fault == "intent" {
            changed.stop_request_id = "different-unissued-stop".into();
        }
        let before = component_physical_tree_snapshot(&fixture.home);
        assert!(
            reader
                .with_locked_retirement(&changed, peer_uid, |_| panic!(
                    "unadmitted Stop reached member owner"
                ))
                .is_err(),
            "{fault}"
        );
        assert_eq!(
            before,
            component_physical_tree_snapshot(&fixture.home),
            "{fault}: failed Stop control read writes nothing"
        );
    }
    let after = component_physical_tree_snapshot(&fixture.home);
    let retry = RetainedWorkerRuntime
        .prepare_ordinary_stop_intent_v1(&source.selection, &request)
        .unwrap();
    assert_eq!(retry.request(), prepared.request());
    assert!(retry.acknowledgment().is_none());
    assert_eq!(
        after,
        component_physical_tree_snapshot(&fixture.home),
        "immutable intent retry creates no new publication"
    );
    let read = source
        .authority
        .capture_ordinary_retained_lifecycle_control_source_v1(
            source.selection.original_session_id(),
            &source.e2_launch_activation.retained_participant_id,
        )
        .unwrap();
    let row = read_ordinary_retained_worker_from_sources_v1(
        read.snapshot(),
        Some(read.admission_registry_bytes()),
        &read.admission_key_files(),
        source.selection.original_session_id(),
        &super::super::host_session_authority::schema::WorldBindingV1 {
            world_id: request.world_id.clone(),
            world_generation: request.world_generation,
        },
        &source.e2_launch_activation.retained_participant_id,
        &crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
            &source.selection,
        )
        .unwrap(),
    )
    .unwrap();
    assert!(matches!(row, OrdinaryRetainedWorkerReadV1::Spawn { record, .. } if matches!(record.state, RetainedWorkerAdmissionStateV1::Routable { .. })),
        "a pending Stop does not relabel bootstrap completion as logical terminal or release capacity");
    for fault in ["idempotency", "caller", "world", "target"] {
        let mut changed = request.clone();
        match fault {
            "idempotency" => changed.idempotency_key = "different-idempotency".into(),
            "caller" => changed.caller_participant_id = "different-caller".into(),
            "world" => changed.world_generation += 1,
            "target" => changed.target_participant_id = Some("unissued-worker".into()),
            _ => unreachable!(),
        }
        let before = component_physical_tree_snapshot(&fixture.home);
        assert!(
            RetainedWorkerRuntime
                .prepare_ordinary_stop_intent_v1(&source.selection, &changed)
                .is_err(),
            "{fault}"
        );
        assert_eq!(
            before,
            component_physical_tree_snapshot(&fixture.home),
            "{fault}: refusal creates no publication"
        );
    }
    std::fs::write(&policy_path, original_policy).unwrap();
    let before_revocation = component_physical_tree_snapshot(&fixture.home);
    assert!(
        RetainedWorkerRuntime
            .prepare_ordinary_stop_intent_v1(&source.selection, &request)
            .is_err(),
        "stored intent never substitutes for current Stop permission"
    );
    assert!(
        reader
            .with_locked_retirement(prepared.request(), *uid, |_| panic!(
                "revoked Stop reached member owner"
            ))
            .is_err(),
        "service control source must independently enforce fresh Stop policy"
    );
    assert_eq!(
        before_revocation,
        component_physical_tree_snapshot(&fixture.home)
    );
    std::fs::write(inventory_path, inventory_bytes).unwrap();
    std::fs::write(config_path, config_bytes).unwrap();
    // All native completion data in this component preamble are synthetic.
    // The actual Stop intent publisher runs, but no service retirement/ACK,
    // native cancellation, kernel, installed or provider qualification occurs.
}

#[test]
fn test_ordinary_stop_intent_late_refusal_keeps_no_owned_staging() {
    use super::super::{dispatch_contract::*, retained_worker_runtime::*};
    for fault in ["policy", "root", "e2-key", "candidate-substitution"] {
        let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
        let _listener = enroll_component_public_start_source(&fixture).unwrap();
        let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
        let policy_path = fixture.home.join("policy.yaml");
        let allowed_policy = std::fs::read_to_string(&policy_path).unwrap().replace(
            "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
            "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, stop_world_worker]");
        std::fs::write(&policy_path, &allowed_policy).unwrap();
        let request = WorldDispatchRequestV1 {
            request_id: Some("req_stop_late_refusal".into()),
            idempotency_key: Some("idem_stop_late_refusal".into()),
            orchestration_session_id: Some(source.selection.original_session_id().into()),
            caller_participant_id: Some(source.selection.current_participant_id().into()),
            action: WorldDispatchActionV1::StopWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(source.selection.requested_selection().backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(
                source.e2_launch_activation.retained_participant_id.clone(),
            ),
            world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
            world_generation: Some(source.e2_launch_activation.target_world.world_generation),
            dispatch_policy_narrowing: None,
            payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
        }
        .validate()
        .unwrap();
        let captured = source
            .authority
            .capture_ordinary_retained_lifecycle_control_source_v1(
                source.selection.original_session_id(),
                &source.e2_launch_activation.retained_participant_id,
            )
            .unwrap();
        let fault_path = match fault {
            "policy" => policy_path.clone(),
            "root" => fixture.home.join("authority-v1/state-root-v1.json"),
            "e2-key" => {
                let super::super::host_session_authority::store::DispatchPolicyCommitmentPhysicalReadV1::Present(e2) = captured.snapshot().e2() else { panic!("actual E2 key missing") };
                fixture
                    .home
                    .join("authority-v1/dispatch-policy-commitment-v1/keys")
                    .join(e2.key_files().iter().next().unwrap().0)
            }
            "candidate-substitution" => fixture
                .home
                .join("authority-v1/retained-worker-admission-v1/tmp"),
            _ => unreachable!(),
        };
        drop(captured);
        let registry_path = fixture
            .home
            .join("authority-v1/retained-worker-admission-v1/registry-v1.json");
        let before_registry = std::fs::read(&registry_path).unwrap();
        let fired = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let fired_by_hook = Arc::clone(&fired);
        let tmp = fixture
            .home
            .join("authority-v1/retained-worker-admission-v1/tmp");
        let hook_tmp = tmp.clone();
        let after_substitution = std::rc::Rc::new(std::cell::RefCell::new(None));
        let capture_substitution = after_substitution.clone();
        let hook_home = fixture.home.clone();
        set_ordinary_population_before_publication_hook(move || {
            assert_eq!(
                std::fs::read_dir(&hook_tmp).unwrap().count(),
                1,
                "fault must reach actual staged candidate, before rename"
            );
            if fault == "candidate-substitution" {
                let own = std::fs::read_dir(&hook_tmp)
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path();
                std::fs::rename(&own, hook_tmp.join("moved-original-owned-candidate")).unwrap();
                std::fs::write(&own, b"foreign substituted candidate; no deletion grant").unwrap();
                std::fs::set_permissions(&own, std::fs::Permissions::from_mode(0o600)).unwrap();
                *capture_substitution.borrow_mut() =
                    Some(component_physical_tree_snapshot(&hook_home));
            } else if fault == "policy" {
                let policy = std::fs::read_to_string(&fault_path)
                    .unwrap()
                    .replace(", stop_world_worker", "");
                std::fs::write(&fault_path, policy).unwrap();
            } else {
                let replacement = fault_path.with_extension("replacement");
                std::fs::write(&replacement, std::fs::read(&fault_path).unwrap()).unwrap();
                std::fs::set_permissions(&replacement, std::fs::Permissions::from_mode(0o600))
                    .unwrap();
                std::fs::rename(replacement, fault_path).unwrap();
            }
            fired_by_hook.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        assert!(
            RetainedWorkerRuntime
                .prepare_ordinary_stop_intent_v1(&source.selection, &request)
                .is_err(),
            "{fault}"
        );
        assert!(fired.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(
            before_registry,
            std::fs::read(&registry_path).unwrap(),
            "{fault}: no Stop intent may publish after source refusal"
        );
        if fault == "candidate-substitution" {
            assert!(after_substitution.borrow().as_ref().unwrap() == &component_physical_tree_snapshot(&fixture.home),
                "a substituted name must refuse cleanup without deleting the replacement or searching for the moved original");
            continue;
        }
        assert!(
            std::fs::read_dir(&tmp).unwrap().next().is_none(),
            "{fault}: rejected owner must settle its own unpublished candidate"
        );
        std::fs::write(&policy_path, &allowed_policy).unwrap();
        let retry = RetainedWorkerRuntime
            .prepare_ordinary_stop_intent_v1(&source.selection, &request)
            .unwrap();
        assert_eq!(retry.request().stop_request_id, request.request_id);
        assert!(
            retry.acknowledgment().is_none(),
            "metadata retry cannot claim native/service retirement"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_ordinary_pending_stop_blocks_new_work_without_closing_bootstrap() {
    use super::super::{dispatch_contract::*, retained_worker_runtime::RetainedWorkerRuntime};
    use crate::execution::orchestrator_world_dispatch::{
        continue_ordinary_world_worker, prepare_ordinary_fork_world_worker_bootstrap,
    };
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _enrollment = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&path).unwrap().replace(
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, stop_world_worker]");
    std::fs::write(path, policy).unwrap();
    let stop = WorldDispatchRequestV1 {
        request_id: Some("req_stop_blocks_new_work".into()),
        idempotency_key: Some("idem_stop_blocks_new_work".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::StopWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.selection.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(source.e2_launch_activation.retained_participant_id.clone()),
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
    };
    let mut fork = stop.clone();
    fork.request_id = Some("req_new_fork_after_stop".into());
    fork.idempotency_key = Some("idem_new_fork_after_stop".into());
    fork.action = WorldDispatchActionV1::ForkWorldWorker;
    fork.payload = WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
        prompt: "must not issue after Stop".into(),
        fork_reason: None,
        fork_strategy: None,
    });
    let store = super::super::state_store::AgentRuntimeStateStore::new().unwrap();
    let mut original_fork = fork.clone();
    original_fork.request_id = Some("req_fork_before_source_stop".into());
    original_fork.idempotency_key = Some("idem_fork_before_source_stop".into());
    let issued = prepare_ordinary_fork_world_worker_bootstrap(
        Arc::clone(&source.selection),
        &store,
        original_fork.clone().validate().unwrap(),
    )
    .unwrap();
    assert!(issued
        .bootstrap
        .policy_commitment
        .as_ref()
        .unwrap()
        .native_issuance
        .is_some());
    drop(issued);
    for index in 0..2 {
        let mut another = original_fork.clone();
        another.request_id = Some(format!("req_unclaimed_before_stop_{index}"));
        another.idempotency_key = Some(format!("idem_unclaimed_before_stop_{index}"));
        let issued = prepare_ordinary_fork_world_worker_bootstrap(
            Arc::clone(&source.selection),
            &store,
            another.validate().unwrap(),
        )
        .unwrap();
        assert!(issued
            .bootstrap
            .policy_commitment
            .as_ref()
            .unwrap()
            .native_issuance
            .is_some());
        drop(issued);
    }
    RetainedWorkerRuntime
        .prepare_ordinary_stop_intent_v1(&source.selection, &stop.clone().validate().unwrap())
        .unwrap();
    let read = source
        .authority
        .capture_ordinary_retained_lifecycle_control_source_v1(
            source.selection.original_session_id(),
            &source.e2_launch_activation.retained_participant_id,
        )
        .unwrap();
    let population =
        super::super::dispatch_policy_commitment::decode_ordinary_bootstrap_population(
            read.snapshot().e2(),
            read.snapshot().authority_store_id(),
        )
        .unwrap();
    let root = super::super::host_session_authority::store_schema::VersionedStateRoot::decode(
        read.snapshot().hsa_state_root_bytes(),
    )
    .unwrap();
    assert_eq!(
        super::super::retained_worker_runtime::count_existing_ordinary_retained_population(
            &root,
            read.admission_registry_bytes(),
            &read.admission_key_files(),
            &population
        )
        .unwrap(),
        4,
        "pending Stop and unclaimed genuine E2 Forks must retain the entire population cap"
    );
    read.verify_before_publication(None).unwrap();
    drop(read);
    let before = component_physical_tree_snapshot(&fixture.home);
    let retry = prepare_ordinary_fork_world_worker_bootstrap(
        Arc::clone(&source.selection),
        &store,
        original_fork.validate().unwrap(),
    )
    .unwrap();
    assert!(
        retry
            .bootstrap
            .policy_commitment
            .as_ref()
            .unwrap()
            .native_issuance
            .is_none(),
        "exact Fork join after source Stop intent cannot mint another native claim"
    );
    drop(retry);
    assert_eq!(
        before,
        component_physical_tree_snapshot(&fixture.home),
        "immutable Fork retry after source Stop writes nothing"
    );
    let held = super::super::host_session_authority::store::hold_existing_accepted_work_authority_snapshot(&source.authority).unwrap();
    let sources = held
        .capture_world_controller_completion_sources(source.selection.original_session_id())
        .unwrap();
    let cut = super::super::obligation_ledger::world_bound_controller::authenticate_world_controller_work_attention_cut(
        &sources, &WorldBindingV1 { world_id: stop.world_id.clone().unwrap(), world_generation: stop.world_generation.unwrap() }).unwrap();
    assert!(
        cut.cut()
            .unresolved_admission_participant_ids
            .contains(&source.e2_launch_activation.retained_participant_id),
        "checked bootstrap cannot close admitted Stop attention before service retirement ACK"
    );
    cut.verify_sources().unwrap();
    assert_eq!(
        before,
        component_physical_tree_snapshot(&fixture.home),
        "World attention read has zero durable effects"
    );
    drop(cut);
    drop(sources);
    drop(held);
    let before = component_physical_tree_snapshot(&fixture.home);
    let result = prepare_ordinary_fork_world_worker_bootstrap(
        Arc::clone(&source.selection),
        &store,
        fork.validate().unwrap(),
    );
    assert!(
        result.is_err(),
        "a pending Stop must block NEW Fork issuance from its source"
    );
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("ordinary_retained_stop_pending_or_closed"));
    assert_eq!(
        before,
        component_physical_tree_snapshot(&fixture.home),
        "Fork refusal must not issue E2/admission/R0 work"
    );
    let socket = fixture._temp.path().join("stop-refusal-no-peer.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket);
    let carrier: transport_api_types::InstallBootstrapContextCarrierV1 =
        serde_json::from_slice(&std::fs::read(fixture._temp.path().join("carrier.json")).unwrap())
            .unwrap();
    let mut next = stop;
    next.request_id = Some("req_new_continue_after_stop".into());
    next.idempotency_key = Some("idem_new_continue_after_stop".into());
    next.action = WorldDispatchActionV1::ContinueWorldWorker;
    next.payload = WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
        prompt: "must not accept after Stop".into(),
        thread_id: None,
    });
    let before = component_physical_tree_snapshot(&fixture.home);
    let result = tokio::time::timeout(
        Duration::from_secs(45),
        continue_ordinary_world_worker(&store, Arc::clone(&source.selection), carrier, next),
    )
    .await
    .unwrap();
    assert!(
        result.is_err(),
        "a pending Stop must block a NEW retained-turn proposal"
    );
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("ordinary_retained_stop_pending_or_closed"));
    assert_eq!(
        before,
        component_physical_tree_snapshot(&fixture.home),
        "Continue refusal must not publish B1/E2 or make a service request"
    );
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    // Real source publishers with synthetic bootstrap terminal: no retirement,
    // ACK, capacity release, native process, kernel or provider claim.
}

#[tokio::test(flavor = "current_thread")]
async fn test_ordinary_stop_source_dispatch_refuses_unqualified_transport_without_sending() {
    use super::super::dispatch_contract::*;
    use crate::execution::orchestrator_world_dispatch::{
        finish_ordinary_world_worker_stop, prepare_ordinary_world_worker_stop,
    };
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _enrollment = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap().replace(
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, stop_world_worker]");
    std::fs::write(policy_path, policy).unwrap();
    let socket = fixture._temp.path().join("unqualified-stop-peer.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    std::env::set_var("SUBSTRATE_WORLD_SOCKET", &socket);
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_source_public_stop_refusal".into()),
        idempotency_key: Some("idem_source_public_stop_refusal".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::StopWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.selection.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(source.e2_launch_activation.retained_participant_id.clone()),
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
    };
    let inventory_path = fixture
        .context
        .sources
        .inventory
        .get(&source.selection.requested_selection().realized_agent_id)
        .unwrap()
        .path
        .clone();
    let config_path = fixture.home.join("config.yaml");
    let config_bytes = std::fs::read(&config_path).unwrap();
    let inventory_bytes = std::fs::read(&inventory_path).unwrap();
    std::fs::write(&config_path, b"malformed: [").unwrap();
    std::fs::write(&inventory_path, b"malformed: [").unwrap();
    let token = std::env::var_os(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);
    let prepared = prepare_ordinary_world_worker_stop(&source.selection, request.clone()).unwrap();
    assert!(prepared
        .scope()
        .observes_active_controller(source.selection.current_participant_id()));
    assert!(!prepared
        .scope()
        .observes_active_controller("different-host"));
    assert!(prepared.scope().acknowledgment().is_none());
    let after_intent = component_physical_tree_snapshot(&fixture.home);
    let refused = finish_ordinary_world_worker_stop(&source.selection, prepared).await;
    match token {
        Some(value) => std::env::set_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, value),
        None => std::env::remove_var(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN),
    }
    assert!(refused
        .err()
        .unwrap()
        .to_string()
        .starts_with("missing_transport:"));
    assert_eq!(after_intent, component_physical_tree_snapshot(&fixture.home), "missing qualified endpoint leaves only the original intent; no closeout/default/repair/allocation");
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
        "no unqualified socket, generic cancel or legacy Stop fallback may receive a request"
    );
    let retry = prepare_ordinary_world_worker_stop(&source.selection, request).unwrap();
    assert!(retry.scope().acknowledgment().is_none());
    assert_eq!(
        after_intent,
        component_physical_tree_snapshot(&fixture.home),
        "immutable intent retry is read-only"
    );
    std::fs::write(config_path, config_bytes).unwrap();
    std::fs::write(inventory_path, inventory_bytes).unwrap();
    // Genuine source producers, synthetic bootstrap. No valid request to the
    // installed service, native retirement, Closed publication, kernel or model.
}

#[test]
fn test_ordinary_stop_original_grant_survives_world_start_settlement() {
    use super::super::{
        dispatch_contract::*,
        host_session_authority::{schema::*, start_continuity::*, store},
        retained_worker_runtime::RetainedWorkerRuntime,
    };
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let (_listener, helper, controller) =
        enroll_component_public_start_source_with_plan(&fixture).unwrap();
    let key = helper
        .startup_prompt
        .as_ref()
        .unwrap()
        .request_key_sha256
        .as_ref()
        .unwrap();
    let transaction = fixture
        .authority
        .retryable_start_transaction(key)
        .unwrap()
        .unwrap();
    let original = fixture
        .authority
        .read_preserved_start_root_v2()
        .unwrap()
        .transition_intent_map[&transaction.start_intent_id]
        .clone();
    let now = || {
        TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            .unwrap()
    };
    fixture
        .authority
        .mark_start_prompt_submission_no_replay_barrier(&transaction.transaction_id, key, now())
        .unwrap();
    // These native protocol observations are synthetic component inputs. All
    // continuation, completion, work-cut and posture publications are genuine.
    let registration = EstablishStartContinuationRequestV1 {
        start_transaction_id: transaction.transaction_id.clone(),
        request_key_sha256: key.clone(),
        authority_store_id: transaction.authority_store_id.clone(),
        orchestration_session_id: transaction.orchestration_session_id.clone(),
        intent_id: original.intent_id.clone(),
        issuer_request_id: original.issuer_request_id.clone(),
        payload_commitment: original.payload_commitment.clone(),
        application_result_ref: transaction.start_application_result_ref.clone(),
        run_id: original.run_id.clone(),
        expected_authority_revision: transaction.start_authority_revision,
        authoritative_participant_id: original.target_authoritative_participant_id.clone(),
        backend_id: helper.descriptor.backend_id.clone(),
        protocol: helper.descriptor.protocol.clone(),
        exchange_id: "synthetic-stop-host-exchange".into(),
        exchange_sequence: 1,
        provider_event_kind: "turn.exchange_opened".into(),
        exchange_evidence_sha256: "a".repeat(64),
        internal_uaa_session_id: "synthetic-stop-host-native-session".into(),
        observed_at: now(),
    };
    let StartContinuationOutcomeV1::Applied(registered) = fixture
        .authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("genuine continuation did not apply")
    };
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap();
    std::fs::write(&policy_path, policy.replace(
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
        "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, stop_world_worker]")).unwrap();
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_stop_before_genuine_host_settlement".into()),
        idempotency_key: Some("idem_stop_before_genuine_host_settlement".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::StopWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.selection.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(source.e2_launch_activation.retained_participant_id.clone()),
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
    }
    .validate()
    .unwrap();
    let prepared = RetainedWorkerRuntime
        .prepare_ordinary_stop_intent_v1(&source.selection, &request)
        .unwrap();
    let settlement = SettleStartTurnRequestV1 {
        start_transaction_id: registration.start_transaction_id.clone(),
        request_key_sha256: registration.request_key_sha256.clone(),
        authority_store_id: registration.authority_store_id.clone(),
        orchestration_session_id: registration.orchestration_session_id.clone(),
        intent_id: registration.intent_id.clone(),
        issuer_request_id: registration.issuer_request_id.clone(),
        payload_commitment: registration.payload_commitment.clone(),
        application_result_ref: registration.application_result_ref.clone(),
        run_id: registration.run_id.clone(),
        authoritative_participant_id: registration.authoritative_participant_id.clone(),
        backend_id: registration.backend_id.clone(),
        protocol: registration.protocol.clone(),
        registered_resume_handle_ref: registered.resume_handle_ref,
        expected_authority_revision: fixture
            .authority
            .resolve_current_exact(&registration.orchestration_session_id, None)
            .unwrap()
            .observation
            .authority_revision,
        protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: registration.authoritative_participant_id.clone(),
        },
        event_id: "synthetic-stop-host-completed".into(),
        event_sequence: 2,
        provider_event_kind: "turn.completed".into(),
        thread_id: registration.internal_uaa_session_id.clone(),
        turn_id: "synthetic-stop-host-turn".into(),
        completion_evidence_sha256: "b".repeat(64),
        kind: StartTurnCompletionKindV1::ResumableClean,
        obligation_ledger_read: None,
        completed_at: now(),
    };
    let held = store::hold_existing_accepted_work_authority_snapshot(&fixture.authority).unwrap();
    let sources = held
        .capture_world_controller_completion_sources(source.selection.original_session_id())
        .unwrap();
    let cut = super::super::obligation_ledger::world_bound_controller::authenticate_world_controller_work_attention_cut(&sources, controller.world_binding()).unwrap();
    let StartTurnSettlementOutcomeV1::Applied(settled) = fixture
        .authority
        .settle_ordinary_world_controller_start(&settlement, &controller, &cut)
        .unwrap()
    else {
        panic!("genuine World Start settlement did not apply")
    };
    assert_eq!(
        settled.resulting_posture,
        HostSessionPostureV1::AwaitingAttention,
        "pending Stop must keep genuine World attention open"
    );
    drop(sources);
    drop(held);
    let before = component_physical_tree_snapshot(&fixture.home);
    assert!(
        RetainedWorkerRuntime
            .prepare_ordinary_stop_intent_v1(&source.selection, &request)
            .is_err(),
        "closed Host posture cannot admit fresh ordinary control"
    );
    let resumed = RetainedWorkerRuntime
        .resume_ordinary_stop_intent_v1(&source.selection, &prepared)
        .unwrap();
    assert_eq!(resumed.request(), prepared.request());
    assert!(resumed.acknowledgment().is_none() && resumed.active_acceptance_record().is_none());
    assert!(!resumed.observes_active_controller(&request.caller_participant_id));
    assert_eq!(
        before,
        component_physical_tree_snapshot(&fixture.home),
        "admitted cleanup source observation writes nothing after Host settlement"
    );
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap();
    std::fs::write(&policy_path, policy.replace(", stop_world_worker", "")).unwrap();
    let before = component_physical_tree_snapshot(&fixture.home);
    assert!(
        RetainedWorkerRuntime
            .resume_ordinary_stop_intent_v1(&source.selection, &prepared)
            .is_err(),
        "stored grant cannot bypass fresh Stop revocation"
    );
    assert_eq!(before, component_physical_tree_snapshot(&fixture.home));
}

#[test]
fn test_ordinary_bootstrap_receiver_claim_survives_fresh_admission() {
    use crate::execution::agent_runtime::dispatch_contract::*;
    use crate::execution::build_ordinary_agent_client_and_member_dispatch_request_for_cwd;
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_authority_bound_spawn_world_worker;
    // Real source-owner publishers; the completed source's native observations
    // are synthetic. No service/native process, kernel or provider effect here.
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap();
    std::fs::write(
        &policy_path,
        policy.replace(
            "fork_world_worker]",
            "fork_world_worker, cancel_world_work, stop_world_worker, inspect_world_worker]",
        ),
    )
    .unwrap();
    let _enrollment = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let selected = Arc::new(source.selection.refresh_original_selection().unwrap());
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_receiver_once".into()),
        idempotency_key: Some("idem_receiver_once".into()),
        orchestration_session_id: Some(selected.original_session_id().into()),
        caller_participant_id: Some(selected.current_participant_id().into()),
        action: WorldDispatchActionV1::SpawnWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(selected.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(source.request.world_id.clone()),
        world_generation: Some(source.request.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerSpawn(WorkerSpawnPayloadV1 {
            prompt: "receiver once input".into(),
        }),
    }
    .validate()
    .unwrap();
    let prepared = prepare_ordinary_authority_bound_spawn_world_worker(selected, request).unwrap();
    let transport = crate::execution::routing::MemberDispatchTransportRequest {
        orchestration_session_id: prepared.request.orchestration_session_id.clone(),
        participant_id: prepared
            .launch_authority_proof
            .retained_participant_id
            .clone(),
        orchestrator_participant_id: prepared.request.caller_participant_id.clone(),
        parent_participant_id: None,
        resumed_from_participant_id: None,
        backend_id: prepared.descriptor.backend_id.clone(),
        protocol: prepared.descriptor.protocol.clone(),
        run_id: prepared.launch_authority_proof.bootstrap_run_id.clone(),
        world_id: prepared.request.world_id.clone(),
        world_generation: prepared.request.world_generation,
        initial_prompt: Some("receiver once input".into()),
        backend_kind: transport_api_types::MemberRuntimeBackendKindV1::Codex,
        binary_path: prepared.descriptor.binary_path.display().to_string(),
        retained_worker_launch_authority: Some(prepared.launch_authority_proof.clone()),
        e2_launch_activation: Some(prepared.e2_launch_activation.clone()),
        config_projection: None,
        exact_policy_snapshot: Some(prepared.exact_policy_snapshot.clone()),
    };
    let (_, execute, _) = build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
        &prepared.selection,
        &transport,
        Path::new(prepared.selection.workspace_root()),
    )
    .unwrap();
    let admission = prepared.selection.prepare_world_execute(&execute).unwrap();
    let registry_path = fixture
        .home
        .join("authority-v1/retained-worker-admission-v1/registry-v1.json");
    let original = std::fs::read(&registry_path).unwrap();
    let record_path = format!(
        "/records_by_session/{}/{}",
        prepared.request.orchestration_session_id,
        prepared.launch_authority_proof.retained_participant_id
    );
    // Negative source substitution only. Never upgrade a pre-protocol claim
    // merely because its receiver receipt is absent.
    for version in [None, Some(2)] {
        let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
        let record = changed
            .pointer_mut(&record_path)
            .unwrap()
            .as_object_mut()
            .unwrap();
        assert_eq!(
            record.get("bootstrap_receiver_protocol"),
            Some(&serde_json::json!(1))
        );
        if let Some(version) = version {
            record.insert("bootstrap_receiver_protocol".into(), version.into());
        } else {
            record.remove("bootstrap_receiver_protocol");
        }
        std::fs::write(&registry_path, serde_json::to_vec(&changed).unwrap()).unwrap();
        let denied = component_tree_snapshot(&fixture.home);
        assert!(admission
            .claim_initial_bootstrap_receiver("spn_unmarked_receiver")
            .is_err());
        assert_eq!(denied, component_tree_snapshot(&fixture.home));
    }
    std::fs::write(&registry_path, &original).unwrap();
    // Before an actual receiver exists, admit only intent. No fabricated span,
    // receiver UUID, Execute digest, SDK delivery or retirement is permitted.
    let early_request = |action, payload, name: &str| {
        WorldDispatchRequestV1 {
            request_id: Some(name.into()),
            idempotency_key: Some(name.into()),
            orchestration_session_id: Some(prepared.request.orchestration_session_id.clone()),
            caller_participant_id: Some(prepared.request.caller_participant_id.clone()),
            action,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(prepared.request.target_backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(
                prepared
                    .launch_authority_proof
                    .retained_participant_id
                    .clone(),
            ),
            world_id: Some(prepared.request.world_id.clone()),
            world_generation: Some(prepared.request.world_generation),
            dispatch_policy_narrowing: None,
            payload,
        }
        .validate()
        .unwrap()
    };
    let early_cancel = early_request(
        WorldDispatchActionV1::CancelWorldWork,
        WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
        "req_initial_cancel",
    );
    let early_stop = early_request(
        WorldDispatchActionV1::StopWorldWorker,
        WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
        "req_initial_stop",
    );
    let pending_intent =
        crate::execution::agent_runtime::retained_worker_runtime::RetainedWorkerRuntime
            .prepare_initial_bootstrap_control_before_receiver_v1(
                &prepared.selection,
                &early_cancel,
            )
            .unwrap()
            .unwrap();
    assert_eq!(pending_intent.control_request_id, early_cancel.request_id);
    let early_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let early_outcome = early_runtime.block_on(crate::execution::orchestrator_world_dispatch::control_ordinary_initial_bootstrap_request_using(
        &prepared.selection, &early_cancel, |_| async { anyhow::bail!("no actual receiver exists for Root delivery") })).unwrap().unwrap();
    let WorldDispatchOutcomeV1::CancelWorldWork(early_outcome) = early_outcome else {
        panic!("wrong pre-receiver intent outcome");
    };
    assert_eq!(
        early_outcome.state,
        CancelWorldWorkTerminalStateV1::CancelAcceptedPendingCloseout
    );
    assert!(early_outcome.closeout.is_none() && early_outcome.terminal_ref.is_none());
    assert!(early_runtime.block_on(crate::execution::orchestrator_world_dispatch::control_ordinary_initial_bootstrap_request_using(
        &prepared.selection, &early_stop, |_| async { anyhow::bail!("no actual receiver exists for Root delivery") })).unwrap_err().to_string().contains("pending"));
    let pending_source = component_tree_snapshot(&fixture.home);
    assert!(
        crate::execution::agent_runtime::retained_worker_runtime::RetainedWorkerRuntime
            .prepare_initial_bootstrap_control_before_receiver_v1(&prepared.selection, &early_stop)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        pending_source,
        component_tree_snapshot(&fixture.home),
        "pending exact retry cannot republish or deliver"
    );
    let pending_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&registry_path).unwrap()).unwrap();
    let body = &pending_json["ordinary_bootstrap_receivers_v1"]["body"];
    assert!(body["by_activation"].as_object().unwrap().is_empty());
    let intents = body["pending_controls_v1"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|value| value.as_array().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(intents.len(), 2);
    for intent in intents {
        for absent in [
            "receiver_claim_id",
            "execute_sha256",
            "service_span_id",
            "retirement_v1",
        ] {
            assert!(
                intent.get(absent).is_none(),
                "pending intent cannot manufacture {absent}"
            );
        }
    }
    assert!(super::super::host_session_authority::store::with_existing_retained_worker_admission_read(
        prepared.selection.held_authority(), |root, bytes, keys| {
            crate::execution::agent_runtime::retained_worker_runtime::require_ordinary_retained_target_open_from_sources(
                root, bytes.unwrap(), &keys, &prepared.request.orchestration_session_id,
                &prepared.launch_authority_proof.retained_participant_id).is_err()
        }).unwrap(), "pre-receiver Stop must already fence new work without claiming retirement");
    let before = component_tree_snapshot(&fixture.home);
    admission
        .claim_initial_bootstrap_receiver("spn_receiver_once")
        .unwrap();
    let claimed = component_tree_snapshot(&fixture.home);
    assert_ne!(before, claimed);
    assert!(admission
        .claim_initial_bootstrap_receiver("spn_receiver_once")
        .is_err());
    assert!(admission
        .claim_initial_bootstrap_receiver("spn_other_receiver")
        .is_err());
    // A raw transport Exit is not the consumed receiver's native/gateway/member
    // closeout, even when its stream/event/span identities are well formed.
    let frame = substrate_common::agent_events::RuntimeFrameIdentityV1 {
        schema_version: 1,
        stream_id: "receiver-raw-exit".into(),
        frame_sequence: 1,
    };
    let event = substrate_common::agent_events::RuntimeEventIdentityV1 {
        event_id: "receiver-raw-exit-event".into(),
        event_sequence: 1,
    };
    let terminal = substrate_common::agent_events::RuntimeTerminalIdentityV1::from(&event);
    assert!(crate::execution::agent_runtime::retained_worker_runtime::RetainedWorkerRuntime.mark_admission_terminal_for_transport(
        prepared.selection.held_authority(), &prepared.admission_plan,
        &prepared.launch_authority_proof.retained_participant_id,
        Some(&prepared.dispatch_policy_reservation_proof), &frame, &event, &terminal,
        "spn_receiver_once", 130,
        TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)).unwrap(),
    ).is_err(), "consumed native receiver requires its checked physical retirement report before capacity release");
    assert_eq!(claimed, component_tree_snapshot(&fixture.home));
    // Fresh caller/receiver objects see the durable consumption, including when
    // the original receiver has not produced Start/Registered/terminal frames.
    drop(admission);
    let fresh = prepared.selection.prepare_world_execute(&execute).unwrap();
    assert!(fresh
        .claim_initial_bootstrap_receiver("spn_restarted_receiver")
        .is_err());
    assert_eq!(claimed, component_tree_snapshot(&fixture.home));
    let controls_seen = std::cell::Cell::new(0);
    fresh
        .with_stored_initial_bootstrap_controls("spn_receiver_once", |wire| {
            assert_eq!(wire.signal, "INT");
            assert_eq!(wire.service_span_id, "spn_receiver_once");
            controls_seen.set(controls_seen.get() + 1);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        controls_seen.get(),
        2,
        "actual reception must adopt both prior source intents"
    );
    assert_eq!(
        claimed,
        component_tree_snapshot(&fixture.home),
        "control adoption is not another source publication or native send"
    );

    // Source-control admission only: callback invocation is not SDK cancellation
    // delivery, physical settlement or retirement proof.
    let control_request = |action, payload, name: &str| {
        WorldDispatchRequestV1 {
            request_id: Some(name.into()),
            idempotency_key: Some(name.into()),
            orchestration_session_id: Some(prepared.request.orchestration_session_id.clone()),
            caller_participant_id: Some(prepared.request.caller_participant_id.clone()),
            action,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(prepared.request.target_backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(
                prepared
                    .launch_authority_proof
                    .retained_participant_id
                    .clone(),
            ),
            world_id: Some(prepared.request.world_id.clone()),
            world_generation: Some(prepared.request.world_generation),
            dispatch_policy_narrowing: None,
            payload,
        }
        .validate()
        .unwrap()
    };
    let cancel_request = control_request(
        WorldDispatchActionV1::CancelWorldWork,
        WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
        "req_initial_cancel",
    );
    let stop_request = control_request(
        WorldDispatchActionV1::StopWorldWorker,
        WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
        "req_initial_stop",
    );
    let runtime = crate::execution::agent_runtime::retained_worker_runtime::RetainedWorkerRuntime;
    let cancel = runtime
        .prepare_initial_bootstrap_control_if_pending_v1(&prepared.selection, &cancel_request)
        .unwrap()
        .expect("genuine consumed initial owner");
    let stop = runtime
        .prepare_initial_bootstrap_control_if_pending_v1(&prepared.selection, &stop_request)
        .unwrap()
        .expect("genuine consumed initial owner");
    let admitted = component_tree_snapshot(&fixture.home);
    assert_eq!(
        stop,
        runtime
            .prepare_initial_bootstrap_control_v1(&prepared.selection, &stop_request)
            .unwrap()
    );
    assert_eq!(admitted, component_tree_snapshot(&fixture.home));
    let encoded = serde_json::to_vec(&stop).unwrap();
    assert_eq!(
        stop,
        serde_json::from_slice::<transport_api_types::OrdinaryInitialBootstrapControlRequestV1>(
            &encoded
        )
        .unwrap()
    );
    let mut extra = serde_json::to_value(&stop).unwrap();
    extra["native_terminal"] = true.into();
    assert!(
        serde_json::from_value::<transport_api_types::OrdinaryInitialBootstrapControlRequestV1>(
            extra
        )
        .is_err()
    );
    let socket_path = fixture._temp.path().join("untrusted-control.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();
    listener.set_nonblocking(true).unwrap();
    let runtime_io = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime_io.block_on(async {
        for client in [
            transport_api_client::AgentClient::unix_socket(&socket_path).unwrap(),
            transport_api_client::AgentClient::tcp("127.0.0.1", 9).unwrap(),
        ] {
            assert_eq!(
                client
                    .control_ordinary_initial_bootstrap(&stop)
                    .await
                    .err()
                    .unwrap()
                    .to_string(),
                "ordinary_initial_control_unsupported_configuration"
            );
        }
        let mut invalid = stop.clone();
        invalid.signal = "KILL".into();
        assert_eq!(
            transport_api_client::AgentClient::unix_socket("/run/substrate.sock")
                .unwrap()
                .control_ordinary_initial_bootstrap(&invalid)
                .await
                .err()
                .unwrap()
                .to_string(),
            "ordinary_initial_control_invalid_request"
        );
    });
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    use crate::execution::orchestrator_world_dispatch::control_ordinary_initial_bootstrap_request_using;
    use transport_api_types::{
        OrdinaryInitialBootstrapControlDeliveryV1 as Delivery,
        OrdinaryInitialBootstrapControlResponseV1 as Response,
    };
    let WorldDispatchOutcomeV1::CancelWorldWork(pending) = runtime_io
        .block_on(control_ordinary_initial_bootstrap_request_using(
            &prepared.selection,
            &cancel_request,
            |wire| {
                assert_eq!(wire, cancel);
                async {
                    Ok(Response {
                        schema_version: 1,
                        delivery: Delivery::Latched,
                    })
                }
            },
        ))
        .unwrap()
        .expect("initial cancellation routed")
    else {
        panic!("wrong initial control outcome");
    };
    assert_eq!(
        pending.state,
        CancelWorldWorkTerminalStateV1::CancelAcceptedPendingCloseout
    );
    assert!(pending.closeout.is_none() && pending.terminal_ref.is_none());
    let WorldDispatchOutcomeV1::CancelWorldWork(unknown) = runtime_io
        .block_on(control_ordinary_initial_bootstrap_request_using(
            &prepared.selection,
            &cancel_request,
            |_| async {
                Ok(Response {
                    schema_version: 1,
                    delivery: Delivery::UnknownOwner,
                })
            },
        ))
        .unwrap()
        .unwrap()
    else {
        panic!("wrong initial control outcome");
    };
    assert_eq!(
        unknown.state,
        CancelWorldWorkTerminalStateV1::OwnerUnreachable
    );
    assert!(unknown.closeout.is_none() && unknown.terminal_ref.is_none());
    let stopped = runtime_io.block_on(control_ordinary_initial_bootstrap_request_using(
        &prepared.selection,
        &stop_request,
        |_| async {
            Ok(Response {
                schema_version: 1,
                delivery: Delivery::Delivered,
            })
        },
    ));
    assert!(
        stopped.unwrap_err().to_string().contains("pending"),
        "delivery cannot report logical Stopped without actual retirement proof"
    );
    let fenced = super::super::host_session_authority::store::with_existing_retained_worker_admission_read(
        prepared.selection.held_authority(), |root, bytes, keys| {
            crate::execution::agent_runtime::retained_worker_runtime::require_ordinary_retained_target_open_from_sources(
                root, bytes.unwrap(), &keys, &prepared.request.orchestration_session_id,
                &prepared.launch_authority_proof.retained_participant_id).is_err()
        }).unwrap();
    assert!(
        fenced,
        "an admitted initial Stop must fence new Resume/Fork before delivery and retirement"
    );
    let inspect_request = control_request(
        WorldDispatchActionV1::InspectWorldWorker,
        WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        "req_initial_control_inspect",
    );
    let inspect_request = serde_json::from_value::<WorldDispatchRequestV1>(
        serde_json::to_value(&inspect_request).unwrap(),
    )
    .unwrap();
    let WorldDispatchOutcomeV1::InspectWorldWorker(inspected) =
        crate::execution::orchestrator_world_dispatch::inspect_ordinary_world_worker_request(
            &prepared.selection,
            inspect_request,
        )
        .unwrap()
    else {
        panic!("wrong inspection outcome");
    };
    let bootstrap = inspected.bootstrap_snapshot.unwrap();
    assert!(!bootstrap.native_bootstrap_completed);
    let original_registration_id = bootstrap.registration_id.clone();
    let controls = bootstrap.initial_control.unwrap();
    assert_eq!(
        controls.cancel_request_id.as_deref(),
        Some(cancel.control_request_id.as_str())
    );
    assert_eq!(
        controls.stop_request_id.as_deref(),
        Some(stop.control_request_id.as_str())
    );
    assert!(
        inspected.stop_snapshot.is_none(),
        "initial intent cannot manufacture a parked-member retirement grant or ACK"
    );
    let mut completed_request = stop_request.clone();
    completed_request.target_participant_id = Some(
        source
            .launch_authority_proof
            .retained_participant_id
            .clone(),
    );
    assert!(
        runtime
            .prepare_initial_bootstrap_control_if_pending_v1(
                &prepared.selection,
                &completed_request
            )
            .unwrap()
            .is_none(),
        "a genuine completed bootstrap retains its existing parked/accepted control path"
    );
    let called = std::cell::Cell::new(false);
    let uid = unsafe { libc::geteuid() };
    assert!(fresh
        .with_initial_bootstrap_control(&cancel, uid, || {
            called.set(true);
            Ok(true)
        })
        .unwrap());
    assert!(called.replace(false));
    assert!(fresh
        .with_initial_bootstrap_control(&stop, 0, || {
            called.set(true);
            Ok(true)
        })
        .is_err());
    assert!(!called.get());
    let mut forged = stop.clone();
    forged.service_span_id = "spn_foreign_owner".into();
    assert!(fresh
        .with_initial_bootstrap_control(&forged, uid, || {
            called.set(true);
            Ok(true)
        })
        .is_err());
    assert!(!called.get());
    let mut unissued = stop.clone();
    unissued.control_request_id = "req_unissued_control".into();
    assert!(fresh
        .with_initial_bootstrap_control(&unissued, uid, || {
            called.set(true);
            Ok(true)
        })
        .is_err());
    assert!(!called.get());
    assert_eq!(admitted, component_tree_snapshot(&fixture.home));
    // Stop retains its own operation permission after Cancel/Spawn revocation.
    let current_policy = std::fs::read_to_string(&policy_path).unwrap();
    std::fs::write(
        &policy_path,
        current_policy
            .replace("spawn_world_worker, ", "")
            .replace("cancel_world_work, ", ""),
    )
    .unwrap();
    let revoked = component_tree_snapshot(&fixture.home);
    assert!(fresh
        .with_initial_bootstrap_control(&cancel, uid, || {
            called.set(true);
            Ok(true)
        })
        .is_err());
    assert!(!called.get());
    assert!(fresh
        .with_initial_bootstrap_control(&stop, uid, || {
            called.set(true);
            Ok(true)
        })
        .unwrap());
    assert!(called.get());
    assert_eq!(revoked, component_tree_snapshot(&fixture.home));
    // Codec and authenticated Source publication only. These deliberately
    // synthetic closeout facts are not Root, native, kernel or retirement proof.
    let observation = transport_api_types::OrdinaryInitialBootstrapRetirementObservationV1 {
        source_start: stop.source_start.clone(),
        activation: stop.activation.clone(),
        original_sender_claim_id: stop.original_sender_claim_id.clone(),
        execute_sha256: stop.execute_sha256.clone(),
        receiver_claim_id: stop.receiver_claim_id.clone(),
        service_span_id: stop.service_span_id.clone(),
        retirement_id: format!("obrt_{}", uuid::Uuid::now_v7()),
        native_processes:
            transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1::ClosedBeforeAnySpawn {},
        gateway_disposal: transport_api_types::OrdinaryGatewayDisposalV1::ProcessWaited {
            runtime_id: "synthetic_initial_gateway".into(),
            pid: 41,
            termination: transport_api_types::NativeProcessTerminationV1::Signaled { signal: 15 },
        },
        frame_identity: frame,
        event_identity: event,
    };
    let reader = fresh
        .initial_bootstrap_closeout_reader("spn_receiver_once")
        .unwrap();
    assert!(fresh
        .initial_bootstrap_closeout_reader("spn_foreign_receiver")
        .is_err());
    called.set(false);
    assert!(reader
        .with_control(&cancel, uid, || {
            called.set(true);
            Ok(true)
        })
        .is_err());
    assert!(!called.get());
    assert!(reader.with_control(&stop, uid, || Ok(true)).unwrap());
    assert!(reader.with_control(&stop, 0, || Ok(true)).is_err());
    let report = reader
        .build_report(
            observation.retirement_id.clone(),
            observation.native_processes.clone(),
            observation.gateway_disposal.clone(),
            observation.frame_identity.clone(),
            observation.event_identity.clone(),
        )
        .unwrap();
    assert_eq!(report.observation(), &observation, "the actual original reader copies scope without retaining Execute input, World or SDK custody");
    assert!(report.matches_control(&stop));
    let retired_response = Response {
        schema_version: 1,
        delivery: Delivery::Retired {
            report: Box::new(report.clone()),
        },
    };
    assert_eq!(
        serde_json::from_value::<Response>(serde_json::to_value(&retired_response).unwrap())
            .unwrap(),
        retired_response
    );
    let mut extra = serde_json::to_value(&retired_response).unwrap();
    extra["delivery"]["retired"]["terminal"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<Response>(extra).is_err(),
        "retired response cannot accept an invented terminal field"
    );
    assert_eq!(
        serde_json::from_value::<transport_api_types::OrdinaryInitialBootstrapRetirementV1>(
            serde_json::to_value(&report).unwrap()
        )
        .unwrap(),
        report
    );
    let population_count = || {
        prepared.selection.with_ordinary_lifecycle_control_sources_v1(
        &stop.activation.retained_participant_id, |_, source, verify, _| {
            let population = super::super::dispatch_policy_commitment::decode_ordinary_bootstrap_population(
                source.snapshot().e2(), prepared.selection.authority_store_id())?;
            let root = super::super::host_session_authority::store_schema::VersionedStateRoot::decode(source.snapshot().hsa_state_root_bytes())?;
            let count = crate::execution::agent_runtime::retained_worker_runtime::count_existing_ordinary_retained_population(
                &root, source.admission_registry_bytes(), &source.admission_key_files(), &population)?;
            verify(None)?;
            Ok(count)
        }).unwrap()
    };
    let count_before = population_count();
    let mut wrong = observation;
    wrong.receiver_claim_id = format!("obr_{}", uuid::Uuid::now_v7());
    let wrong =
        transport_api_types::OrdinaryInitialBootstrapRetirementV1::from_observation(wrong).unwrap();
    assert!(runtime
        .record_initial_bootstrap_retirement_v1(&prepared.selection, &wrong)
        .is_err());
    assert_eq!(revoked, component_tree_snapshot(&fixture.home));
    let rejected = runtime_io.block_on(control_ordinary_initial_bootstrap_request_using(
        &prepared.selection,
        &stop_request,
        |_| async {
            Ok(Response {
                schema_version: 1,
                delivery: Delivery::Retired {
                    report: Box::new(wrong),
                },
            })
        },
    ));
    assert!(rejected
        .unwrap_err()
        .to_string()
        .contains("changed original receiver"));
    assert_eq!(revoked, component_tree_snapshot(&fixture.home));
    let WorldDispatchOutcomeV1::StopWorldWorker(delivered) = runtime_io
        .block_on(control_ordinary_initial_bootstrap_request_using(
            &prepared.selection,
            &stop_request,
            |_| async { Ok(retired_response) },
        ))
        .unwrap()
        .unwrap()
    else {
        panic!("matching reply did not record initial retirement");
    };
    assert_eq!(
        delivered.closeout.participant_state,
        crate::execution::agent_runtime::AgentRuntimeSessionState::Stopped
    );
    assert_eq!(
        population_count(),
        count_before - 1,
        "only the exact checked owner retirement releases its original cap"
    );
    let stopped = runtime_io
        .block_on(control_ordinary_initial_bootstrap_request_using(
            &prepared.selection,
            &stop_request,
            |_| async {
                anyhow::bail!("checked stored retirement must not contact Root or resend")
            },
        ))
        .unwrap()
        .unwrap();
    let WorldDispatchOutcomeV1::StopWorldWorker(stopped) = stopped else {
        panic!("checked initial retirement must close original Stop");
    };
    assert_eq!(
        stopped.closeout.participant_state,
        crate::execution::agent_runtime::AgentRuntimeSessionState::Stopped
    );
    assert_eq!(
        stopped.closeout.session_state,
        crate::execution::agent_runtime::orchestration_session::OrchestrationSessionState::Active
    );
    let held = super::super::host_session_authority::store::hold_existing_accepted_work_authority_snapshot(prepared.selection.held_authority()).unwrap();
    let sources = held
        .capture_world_controller_completion_sources(prepared.selection.original_session_id())
        .unwrap();
    let cut = super::super::obligation_ledger::world_bound_controller::authenticate_world_controller_work_attention_cut(
        &sources, &super::super::host_session_authority::schema::WorldBindingV1 {
            world_id: stop.activation.target_world.world_id.clone(), world_generation: stop.activation.target_world.world_generation,
        }).unwrap();
    assert!(
        cut.cut().unresolved_e2_source_ids.is_empty(),
        "checked initial retirement closes original E2 reservation and commitment attention"
    );
    assert!(
        cut.cut().unresolved_admission_participant_ids.is_empty(),
        "checked failed bootstrap is no longer pending Stop/bootstrap work"
    );
    cut.verify_sources().unwrap();
    drop(cut);
    drop(sources);
    drop(held);
    let inspect_request = control_request(
        WorldDispatchActionV1::InspectWorldWorker,
        WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        "req_initial_retired_inspect",
    );
    let inspect_request = serde_json::from_value::<WorldDispatchRequestV1>(
        serde_json::to_value(&inspect_request).unwrap(),
    )
    .unwrap();
    let WorldDispatchOutcomeV1::InspectWorldWorker(inspected) =
        crate::execution::orchestrator_world_dispatch::inspect_ordinary_world_worker_request(
            &prepared.selection,
            inspect_request,
        )
        .unwrap()
    else {
        panic!("wrong retirement inspection");
    };
    assert!(inspected.summary.contains("retired before completion"));
    let bootstrap = inspected.bootstrap_snapshot.unwrap();
    assert!(!bootstrap.native_bootstrap_completed);
    assert_eq!(bootstrap.registration_id, original_registration_id, "initial retirement must preserve the actual prior R0 source without manufacturing registration");
    assert_eq!(
        bootstrap.initial_control.unwrap().retirement_id.as_deref(),
        Some(report.observation().retirement_id.as_str())
    );
    let retired = component_tree_snapshot(&fixture.home);
    runtime
        .record_initial_bootstrap_retirement_v1(&prepared.selection, &report)
        .unwrap();
    assert_eq!(
        retired,
        component_tree_snapshot(&fixture.home),
        "exact receipt joins without new publication or resend"
    );
    assert!(fresh
        .claim_initial_bootstrap_receiver("spn_after_retirement")
        .is_err());
    assert_eq!(retired, component_tree_snapshot(&fixture.home));
}

#[test]
fn test_ordinary_fork_initial_stop_adopts_original_receiver_and_retirement() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*, retained_worker_runtime::RetainedWorkerRuntime,
        state_store::AgentRuntimeStateStore,
    };
    use crate::execution::orchestrator_world_dispatch::prepare_ordinary_fork_world_worker_bootstrap;
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap().replace(
        "fork_world_worker]",
        "fork_world_worker, stop_world_worker, inspect_world_worker]",
    );
    std::fs::write(&policy_path, policy).unwrap();
    let _enrollment = enroll_component_public_start_source(&fixture).unwrap();
    let (fixture, source, _) = completed_spawn_from_fixture_for_fork_claim_test(fixture);
    let store = AgentRuntimeStateStore::new().unwrap();
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_fork_initial_control".into()),
        idempotency_key: Some("idem_fork_initial_control".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::ForkWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.descriptor.backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(source.e2_launch_activation.retained_participant_id.clone()),
        world_id: Some(source.e2_launch_activation.target_world.world_id.clone()),
        world_generation: Some(source.e2_launch_activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerFork(WorkerForkPayloadV1 {
            prompt: "synthetic completion-owner observation; zero native sends".into(),
            fork_reason: None,
            fork_strategy: None,
        }),
    }
    .validate()
    .unwrap();
    let mut prepared = prepare_ordinary_fork_world_worker_bootstrap(
        Arc::clone(&source.selection),
        &store,
        request,
    )
    .unwrap();
    let native_issuance = prepared
        .bootstrap
        .policy_commitment
        .as_mut()
        .unwrap()
        .native_issuance
        .take();
    let activation = &prepared
        .bootstrap
        .policy_commitment
        .as_ref()
        .unwrap()
        .e2_launch_activation;
    let runtime = RetainedWorkerRuntime;
    let claim = runtime
        .claim_ordinary_fork_native(
            &source.selection,
            activation,
            &source.admission_plan.descriptor_and_runtime_plan.descriptor,
            native_issuance,
        )
        .unwrap();
    assert!(claim.new_native_claim);
    let stop = WorldDispatchRequestV1 {
        request_id: Some("req_fork_initial_stop".into()),
        idempotency_key: Some("idem_fork_initial_stop".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::StopWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(activation.target_backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(activation.retained_participant_id.clone()),
        world_id: Some(activation.target_world.world_id.clone()),
        world_generation: Some(activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
    }
    .validate()
    .unwrap();
    let pending = runtime
        .prepare_initial_bootstrap_control_before_receiver_v1(&source.selection, &stop)
        .unwrap()
        .unwrap();
    assert_eq!(pending.control_request_id, stop.request_id);
    let transport = crate::execution::routing::MemberDispatchTransportRequest {
        orchestration_session_id: activation.orchestration_session_id.clone(),
        participant_id: activation.retained_participant_id.clone(),
        orchestrator_participant_id: activation.caller_participant_id.clone(),
        parent_participant_id: activation.source_participant_id.clone(),
        resumed_from_participant_id: None,
        backend_id: activation.target_backend_id.clone(),
        protocol: source.descriptor.protocol.clone(),
        run_id: activation.bootstrap_run_id.clone(),
        world_id: activation.target_world.world_id.clone(),
        world_generation: activation.target_world.world_generation,
        initial_prompt: Some("synthetic completion-owner observation; zero native sends".into()),
        backend_kind: transport_api_types::MemberRuntimeBackendKindV1::Codex,
        binary_path: source.descriptor.binary_path.display().to_string(),
        retained_worker_launch_authority: None,
        e2_launch_activation: Some(activation.clone()),
        config_projection: None,
        exact_policy_snapshot: Some(
            prepared
                .bootstrap
                .policy_commitment
                .as_ref()
                .unwrap()
                .exact_policy_snapshot
                .clone(),
        ),
    };
    let (_, execute, _) =
        crate::execution::build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
            &prepared.selection,
            &transport,
            Path::new(prepared.selection.workspace_root()),
        )
        .unwrap();
    let admitted = prepared.selection.prepare_world_execute(&execute).unwrap();
    admitted
        .claim_initial_bootstrap_receiver("spn_fork_receiver_once")
        .unwrap();
    let mut controls = Vec::new();
    admitted
        .with_stored_initial_bootstrap_controls("spn_fork_receiver_once", |wire| {
            controls.push(wire.clone());
            Ok(())
        })
        .unwrap();
    assert_eq!(controls.len(), 1);
    let wire = &controls[0];
    assert_eq!(wire.original_sender_claim_id, claim.record.native_claim_id);
    assert_eq!(wire.activation, *activation);
    assert_eq!(wire.control_request_id, pending.control_request_id);
    let held = component_tree_snapshot(&fixture.home);
    assert!(admitted
        .claim_initial_bootstrap_receiver("spn_fork_replay")
        .is_err());
    assert_eq!(held, component_tree_snapshot(&fixture.home));
    // Actual Source/codec only: these physical closeout facts are synthetic.
    // No native launch, Root retirement or checked process observation is claimed.
    let reader = admitted
        .initial_bootstrap_closeout_reader("spn_fork_receiver_once")
        .unwrap();
    let report = reader
        .build_report(
            format!("obrt_{}", uuid::Uuid::now_v7()),
            transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1::ClosedBeforeAnySpawn {},
            transport_api_types::OrdinaryGatewayDisposalV1::ClosedBeforeIssue {
                runtime_id: "synthetic_fork_gateway".into(),
            },
            transport_api_types::RuntimeFrameIdentityV1 {
                schema_version: 1,
                stream_id: "synthetic-fork-closeout".into(),
                frame_sequence: 1,
            },
            transport_api_types::RuntimeEventIdentityV1 {
                event_id: "synthetic-fork-closeout-event".into(),
                event_sequence: 1,
            },
        )
        .unwrap();
    assert!(report.matches_control(wire));
    runtime
        .record_initial_bootstrap_retirement_v1(&source.selection, &report)
        .unwrap();
    let closed = component_tree_snapshot(&fixture.home);
    runtime
        .record_initial_bootstrap_retirement_v1(&source.selection, &report)
        .unwrap();
    assert_eq!(closed, component_tree_snapshot(&fixture.home));
    let inspect = WorldDispatchRequestV1 {
        request_id: Some("req_fork_initial_inspect".into()),
        action: WorldDispatchActionV1::InspectWorldWorker,
        payload: WorldDispatchPayloadV1::WorkerInspect(WorkerInspectPayloadV1::default()),
        ..serde_json::from_value(serde_json::to_value(&stop).unwrap()).unwrap()
    };
    let WorldDispatchOutcomeV1::InspectWorldWorker(inspect) =
        crate::execution::orchestrator_world_dispatch::inspect_ordinary_world_worker_request(
            &source.selection,
            inspect,
        )
        .unwrap()
    else {
        panic!("wrong initial Fork inspect");
    };
    let bootstrap = inspect.bootstrap_snapshot.unwrap();
    assert!(!bootstrap.native_bootstrap_completed);
    assert!(
        bootstrap.registration_id.is_none(),
        "failed initial Fork cannot fabricate R0 or a native-session identity"
    );
    assert_eq!(
        bootstrap.initial_control.unwrap().retirement_id.as_deref(),
        Some(report.observation().retirement_id.as_str())
    );
}

#[test]
fn test_ordinary_initial_stop_grant_survives_successful_bootstrap_without_regrant() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*, retained_worker_runtime::RetainedWorkerRuntime,
    };
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap().replace(
        "fork_world_worker]",
        "fork_world_worker, stop_world_worker]",
    );
    std::fs::write(&policy_path, policy).unwrap();
    let _enrollment = enroll_component_public_start_source(&fixture).unwrap();
    let grant = std::cell::RefCell::new(None);
    let (fixture, source, _) =
        completed_spawn_from_fixture_with_before_native_hook(fixture, |prepared| {
            let WorldDispatchPayloadV1::WorkerSpawn(payload) = &prepared.request.payload else {
                panic!("wrong bootstrap family");
            };
            let transport = crate::execution::routing::MemberDispatchTransportRequest {
                orchestration_session_id: prepared.request.orchestration_session_id.clone(),
                participant_id: prepared
                    .launch_authority_proof
                    .retained_participant_id
                    .clone(),
                orchestrator_participant_id: prepared.request.caller_participant_id.clone(),
                parent_participant_id: None,
                resumed_from_participant_id: None,
                backend_id: prepared.descriptor.backend_id.clone(),
                protocol: prepared.descriptor.protocol.clone(),
                run_id: prepared.launch_authority_proof.bootstrap_run_id.clone(),
                world_id: prepared.request.world_id.clone(),
                world_generation: prepared.request.world_generation,
                initial_prompt: Some(payload.prompt.clone()),
                backend_kind: transport_api_types::MemberRuntimeBackendKindV1::Codex,
                binary_path: prepared.descriptor.binary_path.display().to_string(),
                retained_worker_launch_authority: Some(prepared.launch_authority_proof.clone()),
                e2_launch_activation: Some(prepared.e2_launch_activation.clone()),
                config_projection: None,
                exact_policy_snapshot: Some(prepared.exact_policy_snapshot.clone()),
            };
            let (_, execute, _) =
                crate::execution::build_ordinary_agent_client_and_member_dispatch_request_for_cwd(
                    &prepared.selection,
                    &transport,
                    Path::new(prepared.selection.workspace_root()),
                )
                .unwrap();
            let admission = prepared.selection.prepare_world_execute(&execute).unwrap();
            admission
                .claim_initial_bootstrap_receiver("native-bootstrap-owner-span")
                .unwrap();
            let request = WorldDispatchRequestV1 {
                request_id: Some("req_initial_stop_before_clean_completion".into()),
                idempotency_key: Some("idem_initial_stop_before_clean_completion".into()),
                orchestration_session_id: Some(prepared.request.orchestration_session_id.clone()),
                caller_participant_id: Some(prepared.request.caller_participant_id.clone()),
                action: WorldDispatchActionV1::StopWorldWorker,
                mode: WorldDispatchModeV1::Retained,
                target_backend_id: Some(prepared.request.target_backend_id.clone()),
                task_run_id: None,
                target_participant_id: Some(
                    prepared
                        .launch_authority_proof
                        .retained_participant_id
                        .clone(),
                ),
                world_id: Some(prepared.request.world_id.clone()),
                world_generation: Some(prepared.request.world_generation),
                dispatch_policy_narrowing: None,
                payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
            }
            .validate()
            .unwrap();
            grant.replace(Some(
                RetainedWorkerRuntime
                    .prepare_initial_bootstrap_control_v1(&prepared.selection, &request)
                    .unwrap(),
            ));
            // The actual Source grant won before the synthetic native completion,
            // while no service/SDK delivery is claimed by this fixture.
        });
    let grant = grant.into_inner().unwrap();
    let policy = std::fs::read_to_string(&policy_path).unwrap();
    std::fs::write(
        &policy_path,
        policy
            .replace("spawn_world_worker, ", "")
            .replace("continue_world_worker, ", "")
            .replace("fork_world_worker, ", ""),
    )
    .unwrap();
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_resume_original_initial_stop".into()),
        idempotency_key: Some("idem_resume_original_initial_stop".into()),
        orchestration_session_id: Some(source.selection.original_session_id().into()),
        caller_participant_id: Some(source.selection.current_participant_id().into()),
        action: WorldDispatchActionV1::StopWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(source.request.target_backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(
            source
                .launch_authority_proof
                .retained_participant_id
                .clone(),
        ),
        world_id: Some(source.request.world_id.clone()),
        world_generation: Some(source.request.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let bypassed = runtime.block_on(crate::execution::orchestrator_world_dispatch::control_ordinary_initial_bootstrap_request_using(
        &source.selection, &request.clone().validate().unwrap(), |_| async { anyhow::bail!("completed bootstrap has no initial SDK owner") })).unwrap();
    assert!(
        bypassed.is_none(),
        "a completed bootstrap must move the old Stop grant to parked-member cleanup"
    );
    let prepared =
        crate::execution::orchestrator_world_dispatch::prepare_ordinary_world_worker_stop(
            &source.selection,
            request.clone(),
        )
        .unwrap();
    assert_eq!(
        prepared.scope().request().stop_request_id,
        grant.control_request_id
    );
    assert_eq!(
        prepared.scope().request().bootstrap_run_id,
        grant.activation.bootstrap_run_id
    );
    assert!(
        prepared.scope().acknowledgment().is_none(),
        "grant conversion is not a retirement receipt"
    );
    let held = component_tree_snapshot(&fixture.home);
    let mut exact = request;
    let selector = |issuer: &str| {
        format!(
            "substrate_target_v1:{}",
            serde_json::to_string(&WorldDispatchControlTargetV1::PendingRetainedAdmission {
                issuer_request_id: issuer.into(),
                target_participant_id: source
                    .launch_authority_proof
                    .retained_participant_id
                    .clone(),
            })
            .unwrap()
        )
    };
    exact.target_participant_id = Some(selector("req_forged_initial_stop_issuer"));
    assert!(
        crate::execution::orchestrator_world_dispatch::prepare_ordinary_world_worker_stop(
            &source.selection,
            exact.clone()
        )
        .is_err()
    );
    assert_eq!(
        held,
        component_tree_snapshot(&fixture.home),
        "a foreign issuer cannot rewrite the original Stop grant"
    );
    exact.target_participant_id = Some(selector(&source.request.request_id));
    assert!(
        crate::execution::orchestrator_world_dispatch::prepare_ordinary_world_worker_stop(
            &source.selection,
            exact
        )
        .is_err(),
        "the public Stop contract requires a participant ID, not an exact pending-admission handle"
    );
    assert_eq!(
        held,
        component_tree_snapshot(&fixture.home),
        "unsupported handles must not rewrite the original grant"
    );
    let _ = fixture;
}

#[test]
fn test_ordinary_ephemeral_initial_receiver_is_once_and_has_no_retained_dependency() {
    use crate::execution::agent_runtime::{
        dispatch_contract::*, state_store::AgentRuntimeStateStore,
    };
    use crate::execution::orchestrator_world_dispatch::publish_ordinary_task_source_fixture_with_receiver_for_test;
    let fixture = fixture_for_harness_with_narrowing(true, true, false, true);
    let _listener = enroll_component_public_start_source(&fixture).unwrap();
    let policy_path = fixture.home.join("policy.yaml");
    let policy = std::fs::read_to_string(&policy_path).unwrap()
        .replace("allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker]",
            "allowed_actions: [spawn_world_worker, continue_world_worker, fork_world_worker, run_world_task]")
        .replace("allowed_modes: [retained]", "allowed_modes: [retained, ephemeral]\n    max_concurrent_ephemeral: 2");
    std::fs::write(&policy_path, policy).unwrap();
    let selected = Arc::new(
        OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(Arc::clone(
            &fixture.configured,
        ))
        .unwrap()
        .authenticate(&fixture.context.orchestration_session_id)
        .unwrap(),
    );
    let world = fixture.context.original_world_binding.as_ref().unwrap();
    let request = WorldDispatchRequestV1 {
        request_id: Some("req_ephemeral_receiver_once".into()),
        idempotency_key: Some("idem_ephemeral_receiver_once".into()),
        orchestration_session_id: Some(selected.original_session_id().into()),
        caller_participant_id: Some(selected.current_participant_id().into()),
        action: WorldDispatchActionV1::RunWorldTask,
        mode: WorldDispatchModeV1::Ephemeral,
        target_backend_id: Some(selected.requested_selection().backend_id.clone()),
        task_run_id: None,
        target_participant_id: None,
        world_id: Some(world.world_id.clone()),
        world_generation: Some(world.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::Task(TaskPayloadV1 {
            prompt: "ephemeral receiver source only; no native input".into(),
        }),
    }
    .validate()
    .unwrap();
    let retained = fixture
        .home
        .join("authority-v1/retained-worker-admission-v1");
    assert!(!retained.exists());
    let (_, _, _, admission) = publish_ordinary_task_source_fixture_with_receiver_for_test(
        &AgentRuntimeStateStore::new().unwrap(),
        Arc::clone(&selected),
        request,
        |admission, span, registry, proposal| {
            // Genuine B1/template/opaque Execute publishers; synthetic Start
            // afterward is not Root, native, kernel or provider proof.
            admission.claim_initial_bootstrap_receiver(span)?;
            let consumed = component_tree_snapshot(&fixture.home);
            assert!(admission.claim_initial_bootstrap_receiver(span).is_err());
            assert!(admission
                .claim_initial_bootstrap_receiver("spn_restarted_ephemeral_receiver")
                .is_err());
            assert!(
                registry
                    .prepare_authenticated_ordinary_task_acceptance_proposal(
                        &selected,
                        selected.original_session_id(),
                        proposal.request_id(),
                        None,
                        |allocation, _| allocation
                            .existing_proposal
                            .ok_or_else(|| anyhow::anyhow!("lost original proposal"))
                    )
                    .is_err(),
                "unaccepted exact Task retry must not authorize another native send"
            );
            assert_eq!(consumed, component_tree_snapshot(&fixture.home));
            assert!(!retained.exists());
            Ok(())
        },
    )
    .unwrap();
    // Acceptance is later evidence, not a reset of the original receiver.
    assert!(admission
        .claim_initial_bootstrap_receiver("spn_accepted_task_replay")
        .is_err());
    assert!(!retained.exists());
}
