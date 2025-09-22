const addon = require("..");
const { expect } = require("chai");
const assert = require("chai").assert;

describe("classes", function () {
  it("can create a Message class", function () {
    const Message = addon.Message;

    const message = new Message("Hello, Neon, this is your test speaking.");
    assert.instanceOf(message, Message);
    assert.strictEqual(message.read(), "Hello, Neon, this is your test speaking.");
    const message2 = message.concat(new Message("  <<FNORD>>"));
    assert.strictEqual(message2.read(), "Hello, Neon, this is your test speaking.  <<FNORD>>");
  });

  it("can subclass a Neon class", function () {
    const Message = addon.Message;

    class LoudMessage extends Message {
      shout() {
        return this.read().toUpperCase();
      }
    }

    const message = new LoudMessage("Hello, Neon, this is your test speaking.");
    assert.instanceOf(message, Message);
    assert.instanceOf(message, LoudMessage);
    assert.strictEqual(message.read(), "Hello, Neon, this is your test speaking.");
    assert.strictEqual(message.shout(), "HELLO, NEON, THIS IS YOUR TEST SPEAKING.");
    const message2 = message.concat(new Message("  <<FNORD>>"));
    assert.strictEqual(message2.read(), "Hello, Neon, this is your test speaking.  <<FNORD>>");
    assert.throws(() => message2.shout());
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

  it("can create a StringBuffer class with Default constructor", function () {
    const StringBuffer = addon.StringBuffer;

    const buffer = new StringBuffer();
    buffer.push("Hello");
    buffer.push(" ");
    buffer.push("World");
    assert.strictEqual(buffer.toString(), "Hello World");
  });

  it("can use a renamed method in StringBuffer", function () {
    const StringBuffer = addon.StringBuffer;

    const buffer = new StringBuffer();
    buffer.push("  Hello  ");
    assert.strictEqual(buffer.trimStart(), "Hello  ");
    assert.strictEqual(buffer.trimEnd(), "  Hello");

    assert.isTrue(buffer.includes("Hello"));
    assert.isFalse(buffer.includes("World"));
  });
});
