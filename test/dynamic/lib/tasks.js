var addon = require('../native');
var assert = require('chai').assert;

describe('Task', function() {
  it('completes a successful task', function (done) {
    addon.perform_async_task((err, n) => {
      if (err) {
        done(err);
      } else if (n === 17) {
        done();
      } else {
        done(new Error("not 17 but: " + n));
      }
    });
  });

  it('completes a failing task', function (done) {
    addon.perform_failing_task((err, n) => {
      if (err) {
        if (err.message === 'I am a failing task') {
          done();
        } else {
          done(new Error("expected error message 'I am a failing task', got: " + err.message));
        }
      } else {
        done(new Error("expected task to fail, got: " + n));
      }
    });
  });

  it('executes microtasks after callback', function () {
    return new Promise((resolve, reject) => {
      addon.perform_async_task((err, res) => {
        if (err) {
          reject(err);
        } else {
          resolve(res);
        }
      });
    });
  });
});
