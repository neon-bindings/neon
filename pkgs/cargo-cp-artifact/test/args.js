"use strict";

const assert = require("assert");

const { parse } = require("../src/args");

describe("Argument Parsing", () => {
  it("throws on invalid artifact type", () => {
    assert.throws(() => parse(["-an", "a", "b", "--"]), /artifact type/);
  });

  it("npm must have an environment variable", () => {
    assert.throws(() => parse(["-nc", "a", "b", "--"], {}), /environment/);
  });

  it("must provide a command", () => {
    assert.throws(() => parse(["-ac", "a", "b"]), /Missing command/);
    assert.throws(() => parse(["-ac", "a", "b", "--"]), /Missing command/);
  });

  it("cannot provide invalid option", () => {
    assert.throws(() => parse(["-q"], {}), /Unexpected option/);
  });

  it("should be able to use --artifact", () => {
    const args = "bin my-crate my-bin -- a b c".split(" ");
    const expected = {
      artifacts: {
        "bin:my-crate": ["my-bin"],
      },
      cmd: "a",
      args: ["b", "c"],
    };

    assert.deepStrictEqual(parse(["--artifact", ...args]), expected);
    assert.deepStrictEqual(parse(["-a", ...args]), expected);
  });

  it("should be able to use --npm", () => {
    const args = "bin my-bin -- a b c".split(" ");
    const env = {
      npm_package_name: "my-crate",
    };

    const expected = {
      artifacts: {
        "bin:my-crate": ["my-bin"],
      },
      cmd: "a",
      args: ["b", "c"],
    };

    assert.deepStrictEqual(parse(["--npm", ...args], env), expected);
    assert.deepStrictEqual(parse(["-n", ...args], env), expected);
  });

  it("should be able to use short-hand for crate type with -a", () => {
    const args = "-ab my-crate my-bin -- a b c".split(" ");
    const expected = {
      artifacts: {
        "bin:my-crate": ["my-bin"],
      },
      cmd: "a",
      args: ["b", "c"],
    };

    assert.deepStrictEqual(parse(args), expected);
  });

  it("should be able to use short-hand for crate type with -n", () => {
    const args = "-nb my-bin -- a b c".split(" ");
    const env = {
      npm_package_name: "my-crate",
    };

    const expected = {
      artifacts: {
        "bin:my-crate": ["my-bin"],
      },
      cmd: "a",
      args: ["b", "c"],
    };

    assert.deepStrictEqual(parse(args, env), expected);
  });

  it("should remove namespace from package name", () => {
    const args = "-nc index.node -- a b c".split(" ");
    const env = {
      npm_package_name: "@my-namespace/my-crate",
    };

    const expected = {
      artifacts: {
        "cdylib:my-crate": ["index.node"],
      },
      cmd: "a",
      args: ["b", "c"],
    };

    assert.deepStrictEqual(parse(args, env), expected);
  });

  it("should be able to provide multiple artifacts", () => {
    const args = `
      -nb my-bin
      --artifact d a b
      -ac my-crate index.node
      --npm bin other-copy
      -- a b c
    `
      .trim()
      .split("\n")
      .map((line) => line.trim())
      .join(" ")
      .split(" ");

    const env = {
      npm_package_name: "my-crate",
    };

    assert.deepStrictEqual(parse(args, env), {
      artifacts: {
        "bin:my-crate": ["my-bin", "other-copy"],
        "dylib:a": ["b"],
        "cdylib:my-crate": ["index.node"],
      },
      cmd: "a",
      args: ["b", "c"],
    });
  });
});
