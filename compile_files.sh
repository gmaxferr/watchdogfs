#!/usr/bin/env zsh
setopt extended_glob
set -euo pipefail

# output file (reset each run)
OUT="output.txt"
> "$OUT"

# project root name (used in headers)
PROJECT=$(basename "$PWD")

# recurse through every file under src/
for file in src/**/*; do
  [[ -f $file ]] || continue
  printf '<===== %s/%s =====>\n' "$PROJECT" "$file" >> "$OUT"
  cat "$file"                                           >> "$OUT"
  printf "\n"                                          >> "$OUT"
done
