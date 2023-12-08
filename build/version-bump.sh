#!/bin/bash
set -ux

# Script to bump version of all packages.
# get "minor" or "patch" from first argument

if [ "$#" -ne 1 ]; then
    echo "Illegal number of parameters"
    exit 1
fi

case $1 in
    "minor" | "patch" )
        ;;
    * )
        echo "Illegal argument: $1"
        exit 1
        ;;
esac

# bump version
npm version $1 --workspaces --include-workspace-root --no-git-tag-version
# update dependencies (can be done by build-node.sh)
"$(dirname "$0")/build-node.sh"
# run npm install to update lockfile
npm install
# done!

