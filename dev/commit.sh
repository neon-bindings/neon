#!/bin/bash

DEV=$(dirname $0)
NEON=$($DEV/home.sh)
VERSION=$($DEV/version.sh)

cd $NEON

# ISSUE(#319): this script is not yet tested on the production repo

git commit -m "v$VERSION" || exit $?

git push

curl \
    -u dherman \
    --request POST \
    --data '{"tag_name":"'"$VERSION"'","target_commitish":"master","name":"'"v$VERSION"'","body":"","draft":false,"prerelease":true}' \
    https://api.github.com/repos/neon-bindings/neon/releases
