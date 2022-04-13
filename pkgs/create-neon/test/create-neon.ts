import { assert } from "chai";
import { spawn } from "child_process";
import execa from "execa";
import * as path from "path";
import { promises as fs } from "fs";
import * as TOML from "toml";
import expect from "../dev/expect";
import rimraf from "../dev/rimraf";

const NODE: string = process.execPath;
const CREATE_NEON = path.join(
  __dirname,
  "..",
  "dist",
  "src",
  "bin",
  "create-neon.js"
);

describe("Command-line argument validation", () => {
  it("requires an argument", async () => {
    try {
      await execa(NODE, [CREATE_NEON]);
      assert.fail("should fail when no argument is supplied");
    } catch (expected) {
      assert.isTrue(true);
    }
  });

  it("fails if the directory already exists", async () => {
    try {
      await execa(NODE, [CREATE_NEON, "src"]);
      assert.fail("should fail when directory exists");
    } catch (expected) {
      assert.isTrue(true);
    }
  });
});

const PROJECT = "create-neon-test-project";

describe("Project creation", () => {
  afterEach(async () => {
    await rimraf(PROJECT, { maxBusyTries: 3, emfileWait: 100 });
  });

  it("succeeds with all default answers", async () => {
    try {
      await expect(spawn(NODE, [CREATE_NEON, PROJECT]), {
        "package name:": "",
        "version:": "",
        "description:": "",
        "git repository:": "",
        "keywords:": "",
        "author:": "",
        "license:": "",
        "Is this OK?": "",
      });
    } catch (error: any) {
      assert.fail("create-neon unexpectedly failed: " + error.message);
    }

    let json = JSON.parse(
      await fs.readFile(path.join(PROJECT, "package.json"), {
        encoding: "utf8",
      })
    );

    assert.strictEqual(json.name, PROJECT);
    assert.strictEqual(json.main, "index.node");
    assert.strictEqual(json.version, "0.1.0");
    assert.strictEqual(json.scripts.test, "cargo test");
    assert.strictEqual(json.license, "ISC");
    assert.strictEqual(json.description, "");
    assert.strictEqual(json.author, "");

    let toml = TOML.parse(
      await fs.readFile(path.join(PROJECT, "Cargo.toml"), { encoding: "utf8" })
    );

    assert.strictEqual(toml.package.name, PROJECT);
    assert.strictEqual(toml.package.version, "0.1.0");
    assert.strictEqual(toml.package.license, "ISC");
    assert.deepEqual(toml.lib["crate-type"], ["cdylib"]);
  });

  it("handles quotation marks in author and description", async () => {
    try {
      await expect(spawn(NODE, [CREATE_NEON, PROJECT]), {
        "package name:": "",
        "version:": "",
        "description:": 'the "hello world" of examples',
        "git repository:": "",
        "keywords:": "",
        "author:": '"Dave Herman" <dherman@example.com>',
        "license:": "",
        "Is this OK?": "",
      });
    } catch (error) {
      assert.fail("create-neon unexpectedly failed");
    }

    let json = JSON.parse(
      await fs.readFile(path.join(PROJECT, "package.json"), {
        encoding: "utf8",
      })
    );

    assert.strictEqual(json.description, 'the "hello world" of examples');
    assert.strictEqual(json.author, '"Dave Herman" <dherman@example.com>');

    let toml = TOML.parse(
      await fs.readFile(path.join(PROJECT, "Cargo.toml"), { encoding: "utf8" })
    );

    assert.strictEqual(
      toml.package.description,
      'the "hello world" of examples'
    );
    assert.deepEqual(toml.package.authors, [
      '"Dave Herman" <dherman@example.com>',
    ]);
  });
});
