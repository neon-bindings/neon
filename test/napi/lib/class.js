const addon = require("..");
const { expect } = require("chai");
const assert = require("chai").assert;

describe("classes", function () {
  it("can create a native class", function () {
    const Message = addon.Message;

    const message = new Message("Hello, Neon, this is your test speaking.");
    assert.instanceOf(message, Message);
    assert.strictEqual(message.read(), "Hello, Neon, this is your test speaking.");
    const message2 = message.concat(new Message("  <<FNORD>>"));
    assert.strictEqual(message2.read(), "Hello, Neon, this is your test speaking.  <<FNORD>>");
  });

  it("can create a Point class", function () {
    const Point = addon.Point;

    const point = new Point(1, 2);
    assert.instanceOf(point, Point);
    assert.strictEqual(point.x(), 1);
    assert.strictEqual(point.y(), 2);

    const point2 = new Point(3, 4);
    assert.instanceOf(point2, Point);
    assert.strictEqual(point2.x(), 3);
    assert.strictEqual(point2.y(), 4);
    assert.strictEqual(point.distance(point2), Math.sqrt(8));
  });
});
