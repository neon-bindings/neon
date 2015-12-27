# Neon-cli

Automatically create and build [Neon](https://github.com/dherman/neon) modules.

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

So far Neon is only working on OS X. You'll need [XCode](https://developer.apple.com/xcode/download/), Node v4 or later, and Rust 1.5 or later.

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

The [community Slack](http://neon-community.slack.com) is open to all; use [the Slackin app](http://neon-community.herokuapp.com) to receive an invitation.

There's also an IRC channel at `#neon` on [Mozilla IRC](https://wiki.mozilla.org/IRC) (`irc.mozilla.org`).

# Known Limitations

* I've only gotten this working on OS X.
* Currently, downstream clients of a native Rust module have to have Rust installed on their system in order to build it.
* There's no way to fallback on [precompiled](https://github.com/mapbox/node-pre-gyp) or [portable](http://insertafter.com/en/blog/native-node-module.html) implementations.

I would love to work with people on fixing these limitations!


# License

MIT
