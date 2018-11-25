var addon = require('../native');
var assert = require('chai').assert;
var events = require('events');
var util = require('util');

util.inherits(addon.Emitter, events.EventEmitter);

describe('EventHandler', function() {
  it('event emitter', function (done) {
    var e = new addon.Emitter();

    var taskIds = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    e.on('progress', function (tid) {
      var index = taskIds.indexOf(tid);
      assert.notEqual(index, -1);
      taskIds.splice(index, 1);
    })
    e.on('end', function (result) {
      assert.equal(taskIds.length, 0);
      assert.equal(result, 100);
      // release the underlying EventHandler
      e.shutdown();
      done();
    })
    e.start();
  });

  it('test event emitter', function (done) {
    var e = new addon.TestEmitter(function (cmd) {
      if(cmd == 'number') {
        return 12;
      }
      else if(cmd == 'done') {
        // release the underlying EventHandler
        e.shutdown();
        setTimeout(done);
      }
      else {
        assert.fail(cmd);
      }
    });
    e.start();
  });
});
