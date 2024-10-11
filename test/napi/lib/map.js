var addon = require("..");
var assert = require("chai").assert;

describe("JsMap", function () {
  it("return a JsMap built in Rust", function () {
    assert.deepEqual(new Map(), addon.return_js_map());
  });

  it("return a JsMap with a number as keys and values", function () {
    assert.deepEqual(
      new Map([
        [1, 1000],
        [-1, -1000],
      ]),
      addon.return_js_map_with_number_as_keys_and_values()
    );
  });

  it("return a JsMap with heterogeneous keys/values", function () {
    assert.deepEqual(
      new Map([
        ["a", 1],
        [26, "z"],
      ]),
      addon.return_js_map_with_heterogeneous_keys_and_values()
    );
  });

  it("can read from a JsMap", function () {
    const map = new Map([
      [1, "a"],
      [2, "b"],
    ]);
    assert.strictEqual(addon.read_js_map(map, 2), "b");
  });

  it("can get size from a JsMap", function () {
    const map = new Map([
      [1, "a"],
      [2, "b"],
    ]);
    assert.strictEqual(addon.get_js_map_size(map), 2);
    assert.strictEqual(addon.get_js_map_size(new Map()), 0);
  });

  it("can modify a JsMap", function () {
    const map = new Map([[1, "a"]]);
    addon.modify_js_map(map, 2, "b");
    assert.deepEqual(
      map,
      new Map([
        [1, "a"],
        [2, "b"],
      ])
    );
  });

  it("returns undefined when accessing outside JsMap bounds", function () {
    assert.strictEqual(addon.read_js_map(new Map(), "x"), undefined);
  });

  it("can clear a JsMap", function () {
    const map = new Map([[1, "a"]]);
    addon.clear_js_map(map);
    assert.deepEqual(map, new Map());
  });

  it("can delete key from JsMap", function () {
    const map = new Map([
      [1, "a"],
      ["z", 26],
    ]);

    assert.strictEqual(addon.delete_js_map(map, "unknown"), false);
    assert.deepEqual(
      map,
      new Map([
        [1, "a"],
        ["z", 26],
      ])
    );

    assert.strictEqual(addon.delete_js_map(map, 1), true);
    assert.deepEqual(map, new Map([["z", 26]]));

    assert.strictEqual(addon.delete_js_map(map, "z"), true);
    assert.deepEqual(map, new Map());
  });

  it("can use `has` on JsMap", function () {
    const map = new Map([
      [1, "a"],
      ["z", 26],
    ]);

    assert.strictEqual(addon.has_js_map(map, 1), true);
    assert.strictEqual(addon.has_js_map(map, "z"), true);
    assert.strictEqual(addon.has_js_map(map, "unknown"), false);
  });

  it("can use `forEach` on JsMap", function () {
    const map = new Map([
      [1, "a"],
      ["z", 26],
    ]);
    const collected = [];

    assert.strictEqual(
      addon.for_each_js_map(map, (value, key, map) => {
        collected.push([key, value, map]);
      }),
      undefined
    );

    assert.deepEqual(collected, [
      [1, "a", map],
      ["z", 26, map],
    ]);
  });

  it("can use `groupBy` on JsMap", function () {
    const inventory = [
      { name: "asparagus", type: "vegetables", quantity: 9 },
      { name: "bananas", type: "fruit", quantity: 5 },
      { name: "goat", type: "meat", quantity: 23 },
      { name: "cherries", type: "fruit", quantity: 12 },
      { name: "fish", type: "meat", quantity: 22 },
    ];

    const restock = { restock: true };
    const sufficient = { restock: false };
    const result = addon.group_by_js_map(inventory, ({ quantity }) =>
      quantity < 6 ? restock : sufficient
    );
    assert.deepEqual(
      result,
      new Map([
        [restock, [{ name: "bananas", type: "fruit", quantity: 5 }]],
        [
          sufficient,
          [
            { name: "asparagus", type: "vegetables", quantity: 9 },
            { name: "goat", type: "meat", quantity: 23 },
            { name: "cherries", type: "fruit", quantity: 12 },
            { name: "fish", type: "meat", quantity: 22 },
          ],
        ],
      ])
    );
  });
});
