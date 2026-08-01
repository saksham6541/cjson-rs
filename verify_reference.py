import os
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parent
exe = (
    root / "target" / ("cjson_reference.exe" if os.name == "nt" else "cjson_reference")
)

samples = ['{"a": [1, 2, 3]}', '{"x":true}', '{"n":null}', '{"s":"hi"}']
for sample in samples:
    proc = subprocess.run([str(exe), sample], capture_output=True, text=True)
    print(sample, "rc=", proc.returncode, "stdout=", proc.stdout.strip())
