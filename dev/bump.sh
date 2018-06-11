#!/bin/bash

DEV=$(dirname $0)

"$DEV/boot.sh" || exit $?

NEON=$(cd $DEV/.. && pwd)

CRATES=("$NEON" "$NEON/crates/neon-build" "$NEON/crates/neon-runtime")

echo_usage() {
    echo "Usage: bump.sh [ major | minor | patch | <version> ]"
}

echo_full_usage() {
    echo "bump.sh: Bump the Neon version in all relevant places in the repo"
    echo
    echo_usage
}

die() {
    echo_usage
    echo
    echo $1
    exit 1
}

if [ $# -eq 0 ]; then
    echo_full_usage
    exit 0
fi

if [ $# -ne 1 ]; then
    echo_usage
    exit 1
fi

DELTA=$1

case "$DELTA" in
    -h | --help)
        echo_full_usage
        exit 0
        ;;
    -*)
        die "Unrecognized flag $DELTA."
        ;;
    major | minor | patch)
        ;;
    *)
        if ! semver $DELTA > /dev/null 2>&1 ; then
            die "Invalid semver version $DELTA."
        fi
esac

cargo_add() {
    local crate=$1
    local dep=$2
    local build=$3
    local version=$4

    cargo add --manifest-path=$crate/Cargo.toml $dep $build --vers "=${version}" --path crates/$dep
}

old_version=$($DEV/version.sh)

"$DEV/info.sh" "Bumping crate versions..."

for crate in "${CRATES[@]}" ; do
    (cd "$crate" && cargo bump $DELTA)
done

new_version=$($DEV/version.sh)

"$DEV/info.sh" "Bumping crate dependencies..."

cargo_add $NEON neon-runtime "" ${new_version}
cargo_add $NEON neon-build --build ${new_version}

"$DEV/info.sh" "Bumping template dependencies..."

sed -i~ \
    -e "s/^[[:space:]]*neon-build[[:space:]]*=[[:space:]]\\\"${old_version}\\\"*[[:space:]]*/neon-build = \"${new_version}\"/g" \
    -e "s/^[[:space:]]*neon[[:space:]]*=[[:space:]]\\\"${old_version}\\\"*[[:space:]]*/neon = \"${new_version}\"/g" \
    cli/templates/Cargo.toml.hbs

"$DEV/info.sh" "Bumping CLI version..."

(cd "$NEON/cli" && npm version --no-git-tag-version $DELTA) || exit $?

modified=$(git status -s | grep '^ M ' | awk '{ print $2; }' | fgrep -v package-lock.json)

git diff $modified
