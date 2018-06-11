#!/bin/bash

# boot.sh: Ensure the development environment is setup correctly for running dev scripts.

# make sure the `semver` CLI tool is installed (used for argument validation)
if ! which semver > /dev/null 2>&1 ; then
    npm install -g semver
fi

# make sure the `toml` CLI tool is installed (used for extracting the bumped version)
if ! which toml > /dev/null 2>&1 ; then
    cargo install tomlcli
fi

# make sure the `cargo add` subcommand is installed
if ! which cargo-add > /dev/null 2>&1 ; then
    # Temporarily use my fork until cargo-edit 0.3 is released, which adds
    # support for simultaneously specifying a dependency's version and path.
    cargo install cargo-edit --git https://github.com/dherman/cargo-edit --version=0.3.0-beta.1 || exit 1
fi

# make sure the `cargo bump` subcommand is installed
if ! which cargo-bump > /dev/null 2>&1 ; then
    cargo install cargo-bump || exit 1
fi
