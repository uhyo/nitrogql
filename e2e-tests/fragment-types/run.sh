#!/bin/sh
# Generates the cases and type-checks them.
set -eu
cd "$(dirname "$0")"
NITROGQL_FRAGMENT_TYPES_OUT="$PWD/generated" \
  cargo test --quiet -p nitrogql-printer write_fragment_ref_cases -- --ignored
npx tsc -p .
echo "All cases passed."
