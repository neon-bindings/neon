#!/bin/bash

# version.sh: Print the Neon version, as extracted from the Cargo manifest.

$(dirname $0)/boot.sh || exit $?

NEON=$($(dirname $0)/home.sh)

toml --nocolor "$NEON/Cargo.toml" package.version | xargs echo
