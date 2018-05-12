var addon = require('../native');
var assert = require('chai').assert;

describe('Concurrency', function() {
  it('completes a successful task', function (done) {
    addon.perform_async_task((err, result) => {
      assert(result === 17, "task sent back incorrect value");
      done();
    });
  });

  it('executes microtasks after callback', function () {
    return new Promise((resolve, reject) =>
      addon.perform_async_task((err, result) => {
        if (err) { return reject(err); }
        resolve(result);
      })
    ).then(value =>
      assert(value === 17, "task sent back incorrect value")
    );
  });

  it('completes a successful task on the Node thread pool', function (done) {
    addon.perform_async_task_uv((err, result) => {
        if (err) { return done(err); }
        assert(result === 17, "task send back incorrect value");
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
    let nextCalls = 0;

    const send = addon.create_success_worker((err, complete, event) => {
      if (err) { return done(err); }
      if (event) { 
        const expectedValue = nextCalls === 0 ? "Hello" : "World";
        assert(event === expectedValue, "worker emitted incorrect values");
        nextCalls++;
        return;
      }
      if (complete) {
        assert(complete === "Goodbye", "worker sent incorrect completion value");
        done();
      }
    });

    send("Goodbye");
  })
});
