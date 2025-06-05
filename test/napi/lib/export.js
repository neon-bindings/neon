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
    assert.strictEqual(addon.noArgsOrReturn(), undefined);
  });

  it("add - sync", () => {
    assert.strictEqual(addon.simpleAdd(1, 2), 3);
    assert.strictEqual(addon.renamedAdd(1, 2), 3);
  });

  it("add - task", async () => {
    const p1 = addon.addTask(1, 2);
    const p2 = addon.renamedAddTask(1, 2);

    assert.ok(p1 instanceof Promise);
    assert.ok(p2 instanceof Promise);

    assert.strictEqual(await p1, 3);
    assert.strictEqual(await p2, 3);
  });

  it("json sort", () => {
    const arr = ["b", "c", "a"];
    const expected = [...arr].sort();

    assert.deepStrictEqual(addon.jsonSort(arr), expected);
    assert.deepStrictEqual(addon.renamedJsonSort(arr), expected);
  });

  it("json sort - task", async () => {
    const arr = ["b", "c", "a"];
    const expected = [...arr].sort();
    const p1 = addon.jsonSortTask(arr);
    const p2 = addon.renamedJsonSortTask(arr);

    assert.ok(p1 instanceof Promise);
    assert.ok(p2 instanceof Promise);

    assert.deepStrictEqual(await p1, expected);
    assert.deepStrictEqual(await p2, expected);
  });

  it("can use context and handles", () => {
    const actual = addon.concatWithCxAndHandle("Hello,", " World!");
    const expected = "Hello, World!";

    assert.strictEqual(actual, expected);
  });

  it("error conversion", () => {
    const msg = "Oh, no!";
    const expected = new Error(msg);

    assert.throws(() => addon.failWithThrow(msg), expected);
  });

  it("tasks are concurrent", async () => {
    const time = 500;
    const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
    const start = process.hrtime.bigint();

    await Promise.all([addon.sleepTask(time), sleep(time)]);

    const end = process.hrtime.bigint();
    const duration = end - start;

    // If `addon.sleepTask` blocks the thread, the tasks will run sequentially
    // and take a minimum of 2x `time`. Since they are run concurrently, we
    // expect the time to be closer to 1x `time`.
    const maxExpected = 2000000n * BigInt(time);

    assert.ok(duration < maxExpected);
  });

  it("can use generic Cx in exported functions", () => {
    assert.strictEqual(addon.numberWithCx(42), 42);
  });

  it("i32 parameters", () => {
    assert.strictEqual(addon.addI32(5, 3), 8);
    assert.strictEqual(addon.addI32(-10, 20), 10);
    assert.strictEqual(addon.addI32(-5, -3), -8);
    assert.strictEqual(addon.toI32(Infinity), 0);
    assert.strictEqual(addon.toI32(-Infinity), 0);
  });

  it("u32 parameters", () => {
    assert.strictEqual(addon.addU32(5, 3), 8);
    assert.strictEqual(addon.addU32(100, 200), 300);
    assert.strictEqual(addon.addU32(0, 42), 42);
    assert.strictEqual(addon.toU32(Infinity), 0);
    assert.strictEqual(addon.toU32(-Infinity), 0);
  });
}
