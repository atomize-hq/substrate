from pathlib import Path
import re, sys
p=Path(sys.argv[1])
s=p.read_text()
new = r'''cat > "$RUN/snapshot.py" <<'PY'
#!/usr/bin/python3
import base64, hashlib, json, os, subprocess, sys
path, output = sys.argv[1:3]
def stat_record():
    s = os.stat(path, follow_symlinks=False)
    return {"st_dev": s.st_dev, "st_ino": s.st_ino, "st_mode_octal": format(s.st_mode, "#o"),
            "st_uid": s.st_uid, "st_gid": s.st_gid, "st_size": s.st_size,
            "st_blocks": getattr(s, "st_blocks", None), "st_blksize": getattr(s, "st_blksize", None),
            "st_flags": getattr(s, "st_flags", None), "st_atime_ns": getattr(s, "st_atime_ns", None),
            "st_mtime_ns": getattr(s, "st_mtime_ns", None), "st_ctime_ns": getattr(s, "st_ctime_ns", None),
            "st_birthtime": getattr(s, "st_birthtime", None)}
def command(argv):
    return subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
before = stat_record()
with open(path, "rb") as f: data = f.read()
listed=command(["/usr/bin/xattr", path])
if listed.returncode != 0:
    raise SystemExit("xattr listing failed rc=%d stderr=%r" % (listed.returncode, listed.stderr))
names=sorted(n for n in listed.stdout.decode("utf-8", "surrogateescape").splitlines() if n)
attrs=[]
for name in names:
    raw=command(["/usr/bin/xattr", "-p", name, path])
    rendered=command(["/usr/bin/xattr", "-p", "-x", name, path])
    if raw.returncode != 0 or rendered.returncode != 0:
        raise SystemExit("xattr read failed for %r raw_rc=%d rendered_rc=%d" % (name, raw.returncode, rendered.returncode))
    value=raw.stdout
    attrs.append({"name":name, "length":len(value), "hex":value.hex(),
                  "base64":base64.b64encode(value).decode("ascii"), "sha256":hashlib.sha256(value).hexdigest(),
                  "xattr_p_x_stdout":rendered.stdout.decode("ascii", "backslashreplace"),
                  "xattr_p_x_stderr":rendered.stderr.decode("ascii", "backslashreplace")})
after=stat_record()
with open(output,"w",encoding="utf-8") as f:
    json.dump({"path":path,"ordinary_metadata_before_content_read":before,
               "content":{"length":len(data),"sha256":hashlib.sha256(data).hexdigest(),"base64":base64.b64encode(data).decode("ascii")},
               "complete_xattrs":attrs,"ordinary_metadata_after_content_and_xattr_read":after},f,indent=2,sort_keys=True)
    f.write("\n")
PY
chmod 700 "$RUN/snapshot.py"'''
pattern=r'''cat > "\$RUN/snapshot\.py" <<'PY'\n.*?^PY\nchmod 700 "\$RUN/snapshot\.py"'''
s, n = re.subn(pattern, new, s, count=1, flags=re.S|re.M)
if n != 1: raise SystemExit(f"snapshot block replacement count {n}")
s=s.replace("run_sh volume-diskutil \"/usr/sbin/diskutil info -plist '$RUN'\"", "run_sh volume-diskutil \"/usr/sbin/diskutil info -plist \\\"$(/bin/df -P '$RUN' | /usr/bin/tail -1 | /usr/bin/awk '{print $1}')\\\"\"")
s=s.replace("explicit_product_trace_uses_bound_prefix_and_rejects_conflicts -- --exact --nocapture", "explicit_product_trace_uses_bound_prefix_and_rejects_conflicts -- --nocapture")
s=s.replace("/usr/bin/test ! -e", "/bin/test ! -e")
p.write_text(s)
