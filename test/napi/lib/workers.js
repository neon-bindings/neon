const assert = require("assert");
const {
  Worker,
  isMainThread,
  parentPort,
  threadId,
  workerData,
} = require("worker_threads");

const addon = require("..");

// Receive a message, try that method and return the error message
if (!isMainThread) {
  // RACE: Attempt to reproduce shutdown race condition bug. This depends on timings
  // that may differ across systems. It should not produce spurious failures, but may
  // succeed even if the presence of a bug.
  addon.reject_after(new Error("Oh, no!"), 200).catch(() => {});

  // Reproduce another shutdown bug; this one isn't timing-dependent.
  let boxed_channels = addon.box_channels();

  addon.get_or_init_thread_id(threadId);

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
        case "get_thread_id":
          {
            let id = addon.get_or_init_thread_id(NaN);
            parentPort.postMessage(id);
          }
          break;
        default:
          throw new Error(`Unexpected message: ${message}`);
      }

      throw new Error("Did not throw an exception");
    } catch (err) {
      parentPort.postMessage(err);
    }
  });

  if (workerData === "notify_when_startup_complete") {
    parentPort.postMessage("startup_complete");
  }

  return;
}

// From here on, we're in the main thread.

// Set the `THREAD_ID` Global value in the main thread cell.
addon.get_or_init_thread_id(threadId);

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

describe("Instance-local storage", () => {
  it("should be able to read an instance local from the main thread", () => {
    let lookedUpId = addon.get_or_init_thread_id(NaN);
    assert(!Number.isNaN(lookedUpId));
    assert.strictEqual(lookedUpId, threadId);
  });

  it("should be able to store rooted objects in instance locals", () => {
    addon.stash_global_object();
    assert.strictEqual(global, addon.unstash_global_object());
  });

  it("should gracefully panic upon reentrant get_or_try_init", () => {
    // 1. Global should start out uninitialized
    assert.strictEqual(null, addon.get_reentrant_value());

    // 2. Re-entrancy should panic
    let innerClosureExecuted = false;
    try {
      let result = addon.reentrant_try_init(() => {
        addon.reentrant_try_init(() => {
          innerClosureExecuted = true;
        });
      });
      assert.fail("should have panicked on re-entrancy");
    } catch (expected) {
      assert.strictEqual(
        innerClosureExecuted,
        false,
        "inner closure should not have executed"
      );
    }

    try {
      // 3. Local should still be uninitialized
      assert.strictEqual(null, addon.get_reentrant_value());

      // 4. Successful fallible initialization
      let result = addon.reentrant_try_init(() => {});
      assert.strictEqual(42, result);
      assert.strictEqual(42, addon.get_reentrant_value());
    } catch (unexpected) {
      assert.fail("couldn't set reentrant local after initial failure");
    }
  });

  it("should allocate separate locals for each addon instance", (cb) => {
    let mainThreadId = addon.get_or_init_thread_id(NaN);
    assert(!Number.isNaN(mainThreadId));

    const worker = new Worker(__filename);

    worker.once("message", (message) => {
      assert.strictEqual(typeof message, "number");
      assert.notStrictEqual(message, mainThreadId);
      let mainThreadIdAgain = addon.get_or_init_thread_id(NaN);
      assert(!Number.isNaN(mainThreadIdAgain));
      assert.strictEqual(mainThreadIdAgain, mainThreadId);
      cb();
    });

    worker.postMessage("get_thread_id");
  });

  it("should be able to exit a worker without a crash", (cb) => {
    const worker = new Worker(__filename, {
      workerData: "notify_when_startup_complete",
    });

    worker.once("message", async () => {
      await worker.terminate();
      setTimeout(cb, 200);
    });
  });
});
