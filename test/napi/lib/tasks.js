var addon = require('../native');
const { expect } = require('chai');
var assert = require('chai').assert;

describe('Task', function() {
  it('completes a successful task', function (done) {
    addon.perform_async_task('World', (err, n) => {
      var expected = 'Hello, World!';
      if (err) {
        done(err);
      } else if (n === expected) {
        done();
      } else {
        done(new Error(`not ${expected} but: ${n}`));
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
      addon.perform_async_task('World', (err, res) => {
        if (err) {
          reject(err);
        } else {
          resolve(res);
        }
      });
    });
  });
});
