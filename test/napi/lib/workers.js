const assert = require("assert");
const { Worker, isMainThread, parentPort } = require("worker_threads");

const addon = require("..");

// Receive a message, try that method and return the error message
if (!isMainThread) {
  parentPort.once("message", (message) => {
    try {
      switch (message) {
        case "get_and_replace":
          addon.get_and_replace({});
          break;
        case "get_or_init":
          addon.get_or_init(() => ({}));
          break;
        case "get_or_init_clone":
          addon.get_or_init_clone(() => ({}));
          break;
        default:
          throw new Error(`Unexpected message: ${message}`);
      }

      throw new Error("Did not throw an exception");
    } catch (err) {
      parentPort.postMessage(err);
    }
  });

  return;
}

describe("Worker / Root Tagging Tests", () => {
  describe("Single Threaded", () => {
    it("should be able to stash a global with `get_and_replace`", () => {
      const first = {};
      const second = {};

      assert.strictEqual(addon.get_and_replace(first), undefined);
      assert.strictEqual(addon.get_and_replace(second), first);
      assert.strictEqual(addon.get_and_replace({}), second);
    });

    it("should be able to lazily initialize with `get_or_init`", () => {
      const o = {};

      assert.strictEqual(
        addon.get_or_init(() => o),
        o
      );
      assert.strictEqual(
        addon.get_or_init(() => ({})),
        o
      );
      assert.strictEqual(addon.get_or_init(), o);
    });

    it("should be able to lazily initialize with `get_or_init_clone`", () => {
      const o = {};

      assert.strictEqual(
        addon.get_or_init_clone(() => o),
        o
      );
      assert.strictEqual(
        addon.get_or_init_clone(() => ({})),
        o
      );
      assert.strictEqual(addon.get_or_init_clone(), o);
    });
  });

  // Note: These tests require that the previous set of tests have run or else they will fail
  describe("Multi-Threaded", () => {
    it("should fail to use `get_and_replace`", (cb) => {
      const worker = new Worker(__filename);

      worker.once("message", (message) => {
        assert.ok(/wrong module/.test(message));
        cb();
      });

      worker.postMessage("get_and_replace");
    });

    it("should fail to use `get_or_init`", (cb) => {
      const worker = new Worker(__filename);

      worker.once("message", (message) => {
        assert.ok(/wrong module/.test(message));
        cb();
      });

      worker.postMessage("get_or_init");
    });

    it("should fail to use `get_or_init`", (cb) => {
      const worker = new Worker(__filename);

      worker.once("message", (message) => {
        assert.ok(/wrong module/.test(message));
        cb();
      });

      worker.postMessage("get_or_init_clone");
    });
  });
});
