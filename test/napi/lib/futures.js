const assert = require("assert");

const addon = require("..");

async function assertRejects(f, ...args) {
  try {
    await f();
  } catch (err) {
    assert.throws(() => {
      throw err;
    }, ...args);

    return;
  }

  assert.throws(() => {}, ...args);
}

describe("Futures", () => {
  describe("Channel", () => {
    it("should be able to await channel result", async () => {
      const sum = await addon.lazy_async_add(
        () => 1,
        () => 2
      );

      assert.strictEqual(sum, 3);
    });

    it("exceptions should be handled", async () => {
      await assertRejects(async () => {
        await addon.lazy_async_add(
          () => 1,
          () => {
            throw new Error("Failed to get Y");
          }
        );
      }, /exception/i);
    });
  });

  describe("JsFuture", () => {
    it("should be able to convert a promise to a future", async () => {
      const nums = new Float64Array([1, 2, 3, 4]);
      const sum = await addon.lazy_async_sum(async () => nums);

      assert.strictEqual(sum, 10);
    });

    it("should catch promise rejection", async () => {
      await assertRejects(async () => {
        await addon.lazy_async_sum(async () => {
          throw new Error("Oh, no!");
        });
      }, /exception/i);
    });
  });

  describe("Exported Async Functions", () => {
    it("should be able to call `async fn`", async () => {
      assert.strictEqual(await addon.async_fn_add(1, 2), 3);
    });

    it("should be able to call fn with async block", async () => {
      assert.strictEqual(await addon.async_add(1, 2), 3);
    });

    it("should be able to call fallible `async fn`", async () => {
      assert.strictEqual(await addon.async_fn_div(10, 2), 5);

      await assertRejects(() => addon.async_fn_div(10, 0), /Divide by zero/);
    });

    it("should be able to call fallible `async fn`", async () => {
      assert.strictEqual(await addon.async_fn_div(10, 2), 5);

      await assertRejects(() => addon.async_fn_div(10, 0), /Divide by zero/);
    });

    it("should be able to call fallible fn with async block", async () => {
      assert.strictEqual(await addon.async_div(10, 2), 5);

      await assertRejects(() => addon.async_div(10, 0), /Divide by zero/);
    });

    it("should be able to code on the event loop before and after async", async () => {
      let startCalled = false;
      let endCalled = false;
      const eventHandler = (event) => {
        switch (event) {
          case "start":
            startCalled = true;
            break;
          case "end":
            endCalled = true;
            break;
        }
      };

      process.on("async_with_events", eventHandler);

      try {
        let res = await addon.async_with_events([
          [1, 2],
          [3, 4],
          [5, 6],
        ]);

        assert.deepStrictEqual([...res], [2, 12, 30]);
        assert.ok(startCalled, "Did not emit start event");
        assert.ok(endCalled, "Did not emit end event");
      } finally {
        process.off("async_with_events", eventHandler);
      }
    });
  });
});
