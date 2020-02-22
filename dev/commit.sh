#!/bin/bash

DEV=$(dirname $0)
NEON=$($DEV/home.sh)
VERSION=$($DEV/version.sh)

cd $NEON

git commit -a -m "v$VERSION" || exit $?

git push

curl \
    -u $1 \
    --request POST \
    --data '{"tag_name":"'"$VERSION"'","target_commitish":"master","name":"'"v$VERSION"'","body":"","draft":false,"prerelease":true}' \
    https://api.github.com/repos/neon-bindings/neon/releases
