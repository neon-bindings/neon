import TOML from 'toml';
import { assert } from 'chai';
import { setup } from '../support/acceptance';
import { readFile } from '../support/fs';

describe('neon new', function() {
  setup();

  it('should create a new project', function(done) {
    this.spawn(['new', 'my-app'], { stripColors: true })
        .wait('This utility will walk you through creating the')
        .wait('version').sendline('')
        .wait('desc').sendline('My new app!')
        .wait('node').sendline('')
        .wait('git').sendline('')
        .wait('author').sendline('')
        .wait('email').sendline('')
        .wait('license').sendline('')
        .sendEof()
        .run(err => {
          if (err) throw err;

          let pkg = JSON.parse(readFile(this.cwd, 'my-app/package.json'));
          assert.propertyVal(pkg, 'name', 'my-app');
          assert.propertyVal(pkg, 'version', '0.1.0');
          assert.propertyVal(pkg, 'description', 'My new app!');
          assert.propertyVal(pkg, 'license', 'MIT');
          assert.deepProperty(pkg, 'dependencies.neon-bridge');

          let cargo = TOML.parse(readFile(this.cwd, 'my-app/Cargo.toml'));
          assert.deepPropertyVal(cargo, 'package.name', 'my-app');
          assert.deepPropertyVal(cargo, 'package.version', '0.1.0');
          assert.deepPropertyVal(cargo, 'package.license', 'MIT');
          assert.deepProperty(cargo, 'dependencies.neon');

          let indexjs = readFile(this.cwd, 'my-app/lib/index.js');
          assert.include(indexjs, `require("neon-bridge").load()`);

          let librs = readFile(this.cwd, 'my-app/src/lib.rs');
          assert.include(librs, `extern crate neon;`);

          done();
        });
  });

  it('should create a new project as a scoped package', function(done) {
    this.spawn(['new', '@me/my-package'], { stripColors: true })
        .wait('This utility will walk you through creating the')
        .wait('version').sendline('')
        .wait('desc').sendline('My new scoped package')
        .wait('node').sendline('')
        .wait('git').sendline('')
        .wait('author').sendline('')
        .wait('email').sendline('')
        .wait('license').sendline('')
        .sendEof()
        .run(err => {
          if (err) throw err;

          let pkg = JSON.parse(readFile(this.cwd, 'my-package/package.json'));
          assert.propertyVal(pkg, 'name', '@me/my-package');

          let readme = readFile(this.cwd, 'my-package/README.md');
          assert.match(readme, /@me\/my-package/);

          let cargo = TOML.parse(readFile(this.cwd, 'my-package/Cargo.toml'));
          assert.deepPropertyVal(cargo, 'package.name', 'my-package');

          let binding_cc = readFile(this.cwd, 'my-package/src/binding.cc');
          assert.match(binding_cc, /my_package/);
          assert.notMatch(binding_cc, /@me/);

          let binding_gyp = readFile(this.cwd, 'my-package/binding.gyp');
          assert.match(binding_gyp, /my_package/);
          assert.notMatch(binding_gyp, /@me/);

          done();
        });
  });
});
