#!/bin/bash
set -eux

# Script to bump version of all packages.
# First argument is anything recognized by `npm version`

if [ "$#" -ne 1 ]; then
    echo "Illegal number of parameters"
    exit 1
fi

# bump version
npm version $1 --workspaces --include-workspace-root --no-git-tag-version
# update dependencies (can be done by build-node.sh)
"$(dirname "$0")/build-node.sh"
# run npm install to update lockfile
npm install
# done!

