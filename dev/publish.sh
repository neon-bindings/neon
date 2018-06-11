#!/bin/bash

DEV=$(dirname $0)
NEON=$(cd $DEV/.. && pwd)

# ISSUE(#42): this script is not yet tested in production

(cd "$NEON/cli" && npm publish) || exit $?
(cd "$NEON/crates/neon-build" && cargo publish) || exit $?
(cd "$NEON/crates/neon-runtime" && cargo publish) || exit $?
(cd "$NEON" && cargo publish) || exit $?
