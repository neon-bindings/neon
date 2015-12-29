# Neon-cli

[![Build Status](https://travis-ci.org/rustbridge/neon-cli.svg?branch=master)](https://travis-ci.org/rustbridge/neon-cli)
[![npm](https://img.shields.io/npm/v/neon-cli.svg)](https://www.npmjs.com/package/neon-cli)

Automatically create and build [Neon](https://github.com/rustbridge/neon) modules.

![Screencast](screencast.gif)

# Getting started

Install `neon-cli` as a global package:

```
npm install -g neon-cli
```

To create a new Neon project, use `neon new`:

```
neon new my-project
```

This will ask you a few questions and then generate a project skeleton for you. Follow the instructions from there to build and run your project!

# Requirements

You'll need the following on all OSes:

* [Node](http://nodejs.org) v4 or later;
* [Rust](http://rust-lang.org) v1.5 or later;
* [multirust](https://github.com/brson/multirust) (only required for Neon projects that override the system default Rust).

For Mac OS X, you'll need:

* OS X 10.7 or later;
* [XCode](https://developer.apple.com/xcode/download/).

# Commands

## neon new

Creates a new Neon project skeleton.

```
neon new name
```

The `name` is the project name and the name of the subdirectory of the current working directory that will be created.

## neon build

Builds a Neon project. This command should be part of the `postinstall` script in your `package.json`, which is automatically set up by `neon new`.

```
neon build [--rust toolchain] [--debug]
```

* `--rust`: Use this to specify that [multirust](https://github.com/brson/multirust) should be used instead of the system default Rust installation. The `toolchain` parameter is passed to multirust as the Rust toolchain to use for all build commands.
* `--debug`: Use this to create a debug build.

# Get Involved

The Neon community is just getting started and there's tons of fun to be had. Come play! :)

The [Rust Bridge community Slack](http://rustbridge.slack.com) is open to all; use [the Slackin app](http://rustbridge-community-slackin.herokuapp.com) to receive an invitation.

There's also an IRC channel at `#neon` on [Mozilla IRC](https://wiki.mozilla.org/IRC) (`irc.mozilla.org`).

# License

MIT
