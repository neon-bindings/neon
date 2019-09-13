import * as TOML from 'toml';
import { assert } from 'chai';
import { setup, spawnable } from '../support/acceptance';
import { readFile } from '../support/fs';

type SpawnNeonNewOptions = {
  version?: string,
  desc?: string,
  node?: string,
  git?: string,
  author?: string,
  email?: string,
  license?: string
};

function spawnNeonNew(cx: Mocha.ITestCallbackContext, name: string, opts: SpawnNeonNewOptions = {}, cb: () => void) {
  spawnable(cx).spawn(['new', name])
    .wait('This utility will walk you through creating the')
    .wait('version').sendline(opts.version || '')
    .wait('desc').sendline(opts.desc || '')
    .wait('node').sendline(opts.node || '')
    .wait('git').sendline(opts.git || '')
    .wait('author').sendline(opts.author || '')
    .wait('email').sendline(opts.email || '')
    .wait('license').sendline(opts.license || '')
    .sendEof()
    .run(err => {
      if (err) throw err;

      cb();
    });
}

describe('neon new', function() {
  setup();

  it('should create a new project', function(done) {
    spawnNeonNew(this, 'my-app', {desc: 'My new app!'}, () => {
      let pkg = JSON.parse(readFile(this.cwd, 'my-app/package.json'));
      assert.propertyVal(pkg, 'name', 'my-app');
      assert.propertyVal(pkg, 'version', '0.1.0');
      assert.propertyVal(pkg, 'description', 'My new app!');
      assert.propertyVal(pkg, 'license', 'MIT');
      assert.nestedProperty(pkg, 'dependencies.neon-cli');

      let cargo = TOML.parse(readFile(this.cwd, 'my-app/native/Cargo.toml'));
      assert.nestedPropertyVal(cargo, 'package.name', 'my-app');
      assert.nestedPropertyVal(cargo, 'package.version', '0.1.0');
      assert.nestedPropertyVal(cargo, 'package.license', 'MIT');
      assert.nestedPropertyVal(cargo, 'lib.name', 'my_app');
      assert.nestedProperty(cargo, 'dependencies.neon');

      let indexjs = readFile(this.cwd, 'my-app/lib/index.js');
      assert.include(indexjs, `require('../native')`);

      let librs = readFile(this.cwd, 'my-app/native/src/lib.rs');
      assert.include(librs, `extern crate neon;`);

      done();
    });
  });

  it('should create a new project as a scoped package', function(done) {
    spawnNeonNew(this, '@me/my-package', {}, () => {
      let pkg = JSON.parse(readFile(this.cwd, 'my-package/package.json'));
      assert.propertyVal(pkg, 'name', '@me/my-package');

      let readme = readFile(this.cwd, 'my-package/README.md');
      assert.match(readme, /@me\/my-package/);

      let cargo = TOML.parse(readFile(this.cwd, 'my-package/native/Cargo.toml'));
      assert.nestedPropertyVal(cargo, 'package.name', 'my-package');
      assert.nestedPropertyVal(cargo, 'lib.name', 'my_package');

      done();
    });
  });

  it('should escape quotes in the generated package.json and Cargo.toml', function(done) {
    let opts = {
      desc: 'Foo "bar"',
      author: 'Foo "Bar" Baz',
      git: 'http://www.example.com/foo.git?bar="baz"',
      email: 'haywoodjabuzoff@example.com'
    };

    spawnNeonNew(this, 'my-app', opts, () => {
      let pkg = JSON.parse(readFile(this.cwd, 'my-app/package.json'));
      assert.propertyVal(pkg, 'description', 'Foo "bar"');
      assert.nestedPropertyVal(pkg, 'repository.url', 'http://www.example.com/foo.git?bar=%22baz%22');
      assert.propertyVal(pkg, 'author', 'Foo "Bar" Baz <haywoodjabuzoff@example.com>');

      let cargo = TOML.parse(readFile(this.cwd, 'my-app/native/Cargo.toml'));
      assert.includeDeepMembers(cargo.package.authors, ['Foo "Bar" Baz <haywoodjabuzoff@example.com>'])

      done();
    });
  });
});
