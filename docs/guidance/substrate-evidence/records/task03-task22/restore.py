#!/usr/bin/env python3
"""Reconstruct the Task22 source snapshot in an already-fetched, clean base checkout."""
import hashlib, json, os, stat, subprocess, sys
from pathlib import Path
if len(sys.argv) != 4:
    raise SystemExit('usage: python3 restore.py PACKAGE PRIVATE_MAP CLEAN_CHECKOUT')
pkg, private, checkout = map(Path, sys.argv[1:])
manifest = json.loads((pkg / 'manifest.json').read_text())
map_data = json.loads(private.read_text())
token, original = map_data['placeholder'].encode(), map_data['original_prefix'].encode()
def sha(data): return hashlib.sha256(data).hexdigest()
def git(*args, data=None): return subprocess.run(['git', '-C', str(checkout), *args], input=data, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True).stdout
if git('rev-parse', 'HEAD').decode().strip() != manifest['base_commit']:
    raise SystemExit('wrong base commit')
if git('status', '--porcelain=v1', '--untracked-files=all'):
    raise SystemExit('checkout is not clean')
patch = (pkg / manifest['tracked_patch']['candidate_file']).read_bytes()
if sha(patch) != manifest['tracked_patch']['candidate_sha256']:
    raise SystemExit('screened patch hash mismatch')
patch = patch.replace(token, original)
if sha(patch) != manifest['tracked_patch']['original_sha256']:
    raise SystemExit('private mapping does not reproduce original patch')
for row in manifest['untracked_source_files']:
    target = checkout / row['path']
    if target.exists() or target.is_symlink(): raise SystemExit('untracked destination already exists: ' + row['path'])
git('apply', '--check', '-', data=patch)
git('apply', '-', data=patch)
for row in manifest['untracked_source_files']:
    data = (pkg / row['candidate_file']).read_bytes()
    if sha(data) != row['candidate_sha256']: raise SystemExit('screened source hash mismatch: ' + row['path'])
    data = data.replace(token, original)
    if sha(data) != row['original_sha256']: raise SystemExit('original source hash mismatch: ' + row['path'])
    target = checkout / row['path']; target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data); target.chmod(int(row['original_mode'], 8))
for row in manifest['tracked_working_files']:
    target = checkout / row['path']; mode = row['mode']
    if sha(target.read_bytes()) != row['sha256'] or stat.S_IMODE(target.stat().st_mode) != int(mode, 8):
        raise SystemExit('tracked source mismatch: ' + row['path'])
for row in manifest['untracked_source_files']:
    target = checkout / row['path']
    if sha(target.read_bytes()) != row['original_sha256'] or stat.S_IMODE(target.stat().st_mode) != int(row['original_mode'], 8):
        raise SystemExit('untracked source mismatch: ' + row['path'])
if git('diff', '--cached', '--binary', '--full-index', '--no-ext-diff'):
    raise SystemExit('staged diff is not empty')
if sha(git('diff', '--binary', '--full-index', '--no-ext-diff')) != manifest['tracked_patch']['original_sha256']:
    raise SystemExit('final patch mismatch')
print('restored and checked 37 tracked and 11 untracked source files; native outputs are excluded')
