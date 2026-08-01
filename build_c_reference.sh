#!/usr/bin/env sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
source_dir="$repo_root/original/cJSON"
source_file="$repo_root/c_reference_main.c"
target_dir="$repo_root/target"
mkdir -p "$target_dir"

output_name="cjson_reference"
if [ "$(uname -s)" = "Windows_NT" ]; then
  output_name="cjson_reference.exe"
fi
output_path="$target_dir/$output_name"

compiler=""
for candidate in cc gcc clang; do
  if command -v "$candidate" >/dev/null 2>&1; then
    compiler="$candidate"
    break
  fi
done

if [ -z "$compiler" ]; then
  echo "error: no C compiler found (tried cc, gcc, clang)" >&2
  exit 1
fi

set -x
"$compiler" -O2 -I"$source_dir" "$source_file" "$source_dir/cJSON.c" -o "$output_path"
set +x
echo "built $output_path"
