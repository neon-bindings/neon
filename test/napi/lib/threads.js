const addon = require('..');
const assert = require('chai').assert;

(function () {
  // These tests require GC exposed to shutdown properly; skip if it is not
  return typeof global.gc === 'function' ? describe : describe.skip;
})()('sync', function() {
  afterEach(() => {
    // Force garbage collection to shutdown `Channel`
    global.gc();
  });

  it('can create and deref a root', function () {
    const expected = {};
    const result = addon.useless_root(expected);

    assert.strictEqual(expected, result);
  });

  it('should be able to callback from another thread', function (cb) {
    addon.thread_callback(cb);
  });

  it('should be able to callback from multiple threads', function (cb) {
    const n = 4;
    const set = new Set([...new Array(n)].map((_, i) => i));

    addon.multi_threaded_callback(n, function (x) {
      if (!set.delete(x)) {
        cb(new Error(`Unexpected callback value: ${x}`));
      }

      if (set.size === 0) {
        cb();
      }
    });
  });

  it('should be able to use an async greeter', function (cb) {
    const greeter = addon.greeter_new('Hello, World!', function (greeting) {
      if (greeting === 'Hello, World!') {
        cb();
      } else {
        new Error('Greeting did not match');
      }
    });

    addon.greeter_greet(greeter);
  });

  it('should run callback on drop', function (cb) {
    // IIFE to allow GC
    (function () {
      addon.greeter_new('Hello, World!', function () {}, function () {
        // No assert needed; test will timeout
        cb();
      })
    })();

    global.gc();
  });

  it('should be able to unref channel', function () {
    // If the Channel is not unreferenced, the test runner will not cleanly exit
    addon.leak_channel();
  });

  it('should drop leaked Root from the global queue', function (cb) {
    addon.drop_global_queue(cb);

    // Asynchronously GC to give the task queue a chance to execute
    setTimeout(() => global.gc(), 10);
  });

  it('should be able to join on the result of a channel', function (cb) {
    // `msg` is closed over by multiple functions. A function that returns the
    // current value is passed to the Neon function `addon.channel_join`. Additionally,
    // the value is modified after `10ms` in a timeout.
    let msg = "Uninitialized";

    // The `addon.channel_join` function will wait 100ms before fetching the current
    // value of `msg` using the first closure. The second closure is called
    // after fetching and processing the message. We expect the message to already
    // have been changed.
    addon.channel_join(() => msg, (res) => {
      assert.strictEqual(res, "Received: Hello, World!");
      cb();
    });

    // Change the value of `msg` after 10ms. This should happen before `addon.channel_join`
    // fetches it.
    setTimeout(() => {
      msg = "Hello, World!";
    }, 10);
  });

  it('should be able to sum numbers on the libuv pool', async function () {
    const nums = new Float64Array([...new Array(10000)].map(() => Math.random()));
    const expected = nums.reduce((y, x) => y + x, 0);
    const actual = await addon.sum(nums);

    assert.strictEqual(expected, actual);
  });

  it('should be able to resolve a promise manually', async function () {
    const nums = new Float64Array([...new Array(10000)].map(() => Math.random()));
    const expected = nums.reduce((y, x) => y + x, 0);
    const actual = await addon.sum_manual_promise(nums);

    assert.strictEqual(expected, actual);
  });

  it('should be able to resolve a promise from a rust thread', async function () {
    const nums = new Float64Array([...new Array(10000)].map(() => Math.random()));
    const expected = nums.reduce((y, x) => y + x, 0);
    const actual = await addon.sum_rust_thread(nums);

    assert.strictEqual(expected, actual);
  });

  it('should reject promise if leaked', async function () {
    try {
      await addon.leak_promise();
    } catch (err) {
      assert.instanceOf(err, Error);
      assert.ok(/Deferred/.test(err));
    }
  });
});
