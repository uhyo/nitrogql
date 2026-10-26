#!/bin/sh
# Compares type checking costs of expanded and referred types with TypeScript 7.
# Every result type of the colocation case is fully resolved by relating it to a deep mapped type.
# Usage: ./bench.sh [number of operations, default 200]
set -eu
cd "$(dirname "$0")"
NITROGQL_FRAGMENT_REF_PROTOTYPE_OUT="$PWD/generated" \
NITROGQL_FRAGMENT_REF_COLOCATION_OPERATIONS="${1:-200}" \
  cargo test --quiet -p nitrogql-printer write_fragment_ref_cases -- --ignored > /dev/null
dir=generated/colocation
for side in expanded main; do
  {
    echo "import type * as M from \"./$side\";"
    echo "type Walk<T> = T extends object ? { [K in keyof T]: Walk<T[K]> } : T;"
    grep -o 'export type Q[0-9]*Result' "$dir/main.ts" | sed 's/export type \(.*\)/declare const a_\1: M.\1;\nexport const b_\1: Walk<M.\1> = a_\1;/'
  } > "$dir/force-$side.ts"
  echo "{\"extends\": \"../../../tsconfig.json\", \"include\": [\"force-$side.ts\"]}" > "$dir/tsconfig.$side.json"
  printf "%-9s" "$side"
  npx tsc -p "$dir/tsconfig.$side.json" --extendedDiagnostics | grep -E "^(Instantiations|Check time)" | tr -s ' ' | tr '\n' ' '
  echo
done
