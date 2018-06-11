#!/bin/bash

DEV=$(dirname $0)
NEON=$($DEV/home.sh)

"$DEV/boot.sh" || exit $?

"$DEV/info.sh" "Running Neon tests..."

(cd "$NEON/test/dynamic/native" && cargo update) || exit $?
(cd "$NEON/test/static" && cargo clean && cargo update) || exit $?
(cd "$NEON" && cargo test --release) || exit $?

"$DEV/info.sh" "Creating smoke test..."

(cd "$NEON/cli" && npm run transpile && npm link) || exit $?
rm -rf /tmp/smoke-test
(cd /tmp && neon new smoke-test) || exit $?
cat <<END_SCRIPT | node > /tmp/smoke-test/package.json.new
    var json = require('/tmp/smoke-test/package.json');
    delete json.dependencies;
    json.scripts.install = '$NEON/cli/bin/cli.js build';
    console.log(JSON.stringify(json, null, 2));
END_SCRIPT
mv /tmp/smoke-test/package.json.new /tmp/smoke-test/package.json

cargo_add() {
    local crate=$1
    local dep=$2
    local build=$3
    local path=$4

    cargo add --manifest-path=$crate/Cargo.toml $dep $build --path "$path"
}

cargo_add /tmp/smoke-test/native neon "" "$NEON"
cargo_add /tmp/smoke-test/native neon-build --build "$NEON/crates/neon-build"

"$DEV/info.sh" "Building smoke test..."

(cd /tmp/smoke-test && npm i)

"$DEV/info.sh" "Running smoke test..."

pushd /tmp/smoke-test
output=$(node -e 'require(".")')
popd

if [ "$output" != "hello node" ]; then
    "$DEV/error.sh" "Smoke test failed."
    exit 1
fi

"$DEV/info.sh" "Smoke test passed."

rm -rf /tmp/smoke-test
