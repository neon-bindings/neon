# cargo-cp-artifact

`cargo-cp-artifact` is a small command line utility for parsing cargo metadata output and copying a compiler artifact to a desired location.

## Installation

```sh
npm install -g cargo-cp-artifact
```

## Usage

```
cargo-cp-artifact --artifact artifact-kind crate-name output-file -- wrapped-command
```

`cargo-cp-artifact` accepts a list of crate name and artifact kind to output file mappings and a command to wrap.`cargo-cp-artifact` will read `stdout` of the wrapped command and parse it as [cargo metadata](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages). Compiler artifacts that match arguments provided will be copied to the target destination.

When wrapping a `cargo` command, it is necessary to include a `json` format to `--message-format`.

### Arguments

Multiple arguments are allowed to copy multiple build artifacts.

#### `--artifact`

_Alias: `-a`_

Followed by *three* arguments: `artifact-kind crate-name output-file`

#### `--npm`

_Alias: `-n`_

Followed by *two* arguments: `artifact-kind output-file`

The crate name will be read from the `npm_package_name` environment variable. If the package name includes a namespace (`@namespace/package`), the namespace will be removed when matching the crate name (`package`).

### Artifact Kind

Valid artifact kinds are `bin`, `cdylib`, and `dylib`. They may be abbreviated as `b`, `c`, and `d` respectively.

For example, `-ac` is the equivalent of `--artifact cdylib`.

## Examples

### Wrapping cargo

```sh
cargo-cp-artifact -a cdylib my-crate lib/index.node -- cargo build --message-format=json-render-diagnostics
```

### Parsing a file

```sh
cargo-cp-artifact -a cdylib my-crate lib/index.node -- cat build-output.txt
```

### `npm` script

`package.json`
```json
{
    "name": "my-crate",
    "scripts": {
        "build": "cargo-cp-artifact -nc lib/index.node -- cargo build --message-format=json-render-diagnostics"
    }
}
```

```sh
npm run build

# Additional arguments may be passed
npm run build -- --feature=serde
```

## Why does this exist?

At the time of writing, `cargo` does not include a configuration for outputting a library or binary to a specified location. An `--out-dir` option [exists on nightly](https://github.com/rust-lang/cargo/issues/6790), but does not allow specifying the name of the file.

It's surprisingly difficult to reliably find the location of a cargo compiler artifact. It is impacted by many parameters, including:

* Build profile
* Target, default or specified
* Crate name and name transforms

However, `cargo` can emit metadata on `stdout` while continuing to provide human readable diagnostics on `stderr`. The metadata may be parsed to more easily and reliably find the location of compiler artifacts.

`cargo-cp-artifact` chooses to wrap a command as a child process instead of reading `stdin` for two reasons:

1. Removes the need for `-o pipefile` when integrating with build tooling which may need to be platform agnostic.
2. Allows additional arguments to be provided when used in an [`npm` script](https://docs.npmjs.com/cli/v6/using-npm/scripts).
