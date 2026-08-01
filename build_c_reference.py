#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
from pathlib import Path


def main() -> int:
    repo_root = Path(__file__).resolve().parent
    src_dir = repo_root / "original" / "cJSON"
    source_file = repo_root / "c_reference_main.c"
    target_dir = repo_root / "target"
    target_dir.mkdir(exist_ok=True)

    output_name = "cjson_reference.exe" if os.name == "nt" else "cjson_reference"
    output_path = target_dir / output_name

    compiler = shutil.which("cc") or shutil.which("gcc") or shutil.which("clang")
    if not compiler:
        print("error: no C compiler found (tried cc, gcc, clang)", file=sys.stderr)
        return 1

    command = [
        compiler,
        "-O2",
        f"-I{src_dir}",
        str(source_file),
        str(src_dir / "cJSON.c"),
        "-o",
        str(output_path),
    ]
    print(" ".join(command))
    subprocess.run(command, cwd=repo_root, check=True)
    print(f"built {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
