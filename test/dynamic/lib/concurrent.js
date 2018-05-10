var addon = require('../native');
var assert = require('chai').assert;

describe('Concurrency', function() {
  it('completes a successful task', function (done) {
    const complete = value => {
        assert(value === 17, "task sent back incorrect value");
        done();
    }
    addon.perform_async_task(done, complete);
  });

  it('executes microtasks after callback', function () {
    return new Promise((resolve, reject) =>
      addon.perform_async_task(reject, resolve)
    ).then(value =>
      assert(value === 17, "task sent back incorrect value")
    );
  });

  it('completes a successful task on the Node thread pool', function (done) {
    addon.perform_async_task_uv((err, value) => {
        if (err) { return done(err); }
        assert(value === 17, "task send back incorrect value");
        done();
    });
  });

  it('executes microtasks after callback on the Node thread pool', function () {
    return new Promise((resolve, reject) => {
      addon.perform_async_task_uv((err, value) => {
        if (err) { return reject(err); }
        resolve(value);
      })
    }).then(value =>
      assert(value === 17, "task send back incorrect value")
    );
  });

  it('runs a worker to completion', function (done) {
    const complete = value => {
      assert(value === "Goodbye", "worker sent incorrect completion value");
      done();
    }
    let nextCalls = 0;
    const next = value => {
      const expectedValue = nextCalls === 0 ? "Hello" : "World";
      assert(value === expectedValue, "worker emitted incorrect values");
      nextCalls++;
    }
    const send = addon.create_success_worker(done, complete, next);
    send("Goodbye");
  })
});
