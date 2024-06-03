# Create Neon

The `create-neon` tool bootstraps [Neon](https://neon-bindings.com) projects, which allows developers to build binary Node modules written in [Rust](https://www.rust-lang.org).

## Usage

You can conveniently use this tool with the [`npm init`](https://docs.npmjs.com/cli/v7/commands/npm-init) syntax:

### Creating a Simple Project

To create a simple Neon project that consists purely of Rust code:

```sh
$ npm init neon[@latest] -- [<opts> ...] my-project
```

**Note:** The initial `--` is necessary for `npm init` to pass any command-line options to Neon.

**Note:** The `@latest` ensures that npm uses the latest version of this tool.

#### Global Options

```sh
-y|--yes  Skip interactive `npm init` questionnaire.
```

### Creating a Portable Library

Neon also makes it easy to create **portable, cross-platform libraries** by publishing pre-built binaries. This means you can implement your Node.js library in Rust and publish the binaries so that users of your library (and any downstream users of theirs!) on all major hardware and operating systems can take a dependency on your library---_without having to install Rust or run any builds_.

To create a portable npm library with pre-built binaries:

```sh
$ npm init neon[@latest] -- [<opts> ...] --lib [<lib-opts> ...] my-project
```

**Note:** The initial `--` is necessary for `npm init` to pass any command-line options to Neon.

This will generate a project that can be used by pure JavaScript or TypeScript consumers without them even being aware of the use of Rust under the hood. It achieves this by publishing pre-built binaries for common Node platform architectures that are loaded just-in-time by a JS wrapper module.

This command generates the necessary npm and CI/CD configuration boilerplate to require nearly zero manual installation on typical GitHub-hosted repos. The only manual step required is to configure GitHub Actions with the necessary npm access token to enable automated publishing.

This command chooses the most common setup by default, but allows customization with fine-grained configuration options. These configuration options can also be modified later with the [Neon CLI](https://www.npmjs.com/package/@neon-rs/cli).

#### Library Options

```sh
--ci none|github       CI/CD provider to generate config for.
                       (Default: github)
--bins none|npm[:org]  Cache provider to publish pre-built binaries.
                       (Default: npm, with org inferred from package)
--platform <platform>  Binary platform to add support to this library for.
                       This option can be specified multiple times.
                       (Default: macos, linux, windows)
```
