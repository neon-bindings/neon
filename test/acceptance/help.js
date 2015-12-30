import { setup } from '../support/acceptance';

describe('neon help', function() {
  setup();

  it('should print neon usage', function(done) {
    this.spawn(['help'])
        .wait('Usage:')
        .wait('neon new')
        .wait('neon version')
        .wait('neon help')
        .run(err => {
          if (err) throw err;
          done();
        });
  });
});
