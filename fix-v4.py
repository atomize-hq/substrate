from pathlib import Path
import sys
p=Path(sys.argv[1]); s=p.read_text()
old='    f.write("'+'\n'+ '")\nPY'
new='    f.write("\\\\n")\nPY'
if old not in s:
    raise SystemExit('broken newline pattern not found')
s=s.replace(old,new,1)
p.write_text(s)
