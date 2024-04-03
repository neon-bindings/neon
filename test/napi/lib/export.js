const assert = require("assert");

const addon = require("..");

describe("neon::export macro", () => {
  describe("globals", globals);
  describe("functions", functions);
});

function globals() {
  it("values", () => {
    assert.strictEqual(addon.NUMBER, 42);
    assert.strictEqual(addon.STRING, "Hello, World!");
    assert.strictEqual(addon.renamedString, "Hello, World!");
  });

  it("json", () => {
    assert.deepStrictEqual(addon.MESSAGES, ["hello", "neon"]);
    assert.deepStrictEqual(addon.renamedMessages, ["hello", "neon"]);
  });
}

function functions() {
  it("void function", () => {
    assert.strictEqual(addon.no_args_or_return(), undefined);
  });

  it("add - sync", () => {
    assert.strictEqual(addon.simple_add(1, 2), 3);
    assert.strictEqual(addon.renamedAdd(1, 2), 3);
  });

  it("add - task", async () => {
    const p1 = addon.add_task(1, 2);
    const p2 = addon.renamedAddTask(1, 2);

    assert.ok(p1 instanceof Promise);
    assert.ok(p2 instanceof Promise);

    assert.strictEqual(await p1, 3);
    assert.strictEqual(await p2, 3);
  });

  it("json sort", () => {
    const arr = ["b", "c", "a"];
    const expected = [...arr].sort();

    assert.deepStrictEqual(addon.json_sort(arr), expected);
    assert.deepStrictEqual(addon.renamedJsonSort(arr), expected);
  });

  it("json sort - task", async () => {
    const arr = ["b", "c", "a"];
    const expected = [...arr].sort();
    const p1 = addon.json_sort_task(arr);
    const p2 = addon.renamedJsonSortTask(arr);

    assert.ok(p1 instanceof Promise);
    assert.ok(p2 instanceof Promise);

    assert.deepStrictEqual(await p1, expected);
    assert.deepStrictEqual(await p2, expected);
  });

  it("can use context and handles", () => {
    const actual = addon.concat_with_cx_and_handle("Hello,", " World!");
    const expected = "Hello, World!";

    assert.strictEqual(actual, expected);
  });

  it("error conversion", () => {
    const msg = "Oh, no!";
    const expected = new Error(msg);

    assert.throws(() => addon.fail_with_throw(msg), expected);
  });

  it("tasks are concurrent", async () => {
    const time = 100;
    const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
    const start = process.hrtime.bigint();

    await Promise.all([addon.sleep_task(time), sleep(time)]);

    const end = process.hrtime.bigint();
    const duration = end - start;

    // If `addon.sleep_task` blocks the thread, the tasks will run sequentially
    // and take a minimum of 2x `time`. Since they are run concurrently, we
    // expect the time to be closer to 1x `time`.
    const maxExpected = 2000000n * BigInt(time);

    assert.ok(duration < maxExpected);
  });
}
