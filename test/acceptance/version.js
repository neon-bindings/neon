import { setup } from '../support/acceptance';
import { readFile } from '../support/fs';

const pkg = JSON.parse(readFile(__dirname, '../../package.json'));

describe('neon version', function() {
  setup();

  it('should print neon usage', function(done) {
    this.spawn(['version'])
        .wait(pkg.version)
        .run(err => {
          if (err) throw err;
          done();
        });
  });
});
