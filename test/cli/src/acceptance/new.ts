import * as TOML from 'toml';
import { assert } from 'chai';
import { setup, spawnable } from '../support/acceptance';
import { readFile } from '../support/fs';
import * as path from 'path';

type SpawnNeonNewOptions = {
  version?: string,
  desc?: string,
  node?: string,
  git?: string,
  author?: string,
  email?: string,
  license?: string,
  neon?: string,
  features?: string
};

function spawnNeonNew(cx: Mocha.ITestCallbackContext, name: string, opts: SpawnNeonNewOptions = {}, cb: () => void) {
  let args = ['new'];

  if (opts.neon) {
    args.push("--neon");
    args.push(opts.neon);
  }

  if (opts.features) {
    args.push('--features');
    args.push(opts.features);
  }

  args.push(name);

  spawnable(cx).spawn(args)
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

function manifests(cwd: string, lib: string) : { pkg: any, cargo: any } {
  return {
    pkg: JSON.parse(readFile(cwd, lib, 'package.json')),
    cargo: TOML.parse(readFile(cwd, lib, 'native', 'Cargo.toml'))
  };
}

function assertNormalNeonCli(pkg: any) {
  assert.typeOf(pkg.dependencies['neon-cli'], 'string');
  assert.match(pkg.dependencies['neon-cli'], /^\^\d+\.\d+\.\d+$/);
}

function assertLocalNeonCli(pkg: any) {
  assert.nestedProperty(pkg, 'dependencies.neon-cli');
  assert.match(pkg.dependencies['neon-cli'], /^file:.*cli$/);
}

function assertRelativeNeonCli(pkg: any) {
  assert.match(pkg.dependencies['neon-cli'], /^file:/);
}

function assertAbsoluteNeonCli(pkg: any) {
  assert.match(pkg.dependencies['neon-cli'], /^file:/);
  let local = pkg.dependencies['neon-cli'].substring(5).trim();
  assert.isTrue(path.isAbsolute(local));
}

function assertLocalNeon(cargo: any) {
  assert.typeOf(cargo.dependencies.neon.path, 'string');
  assert.typeOf(cargo['build-dependencies']['neon-build'].path, 'string');
  assert.nestedPropertyVal(cargo, 'build-dependencies.neon-build.path', path.join(cargo.dependencies.neon.path, "crates", "neon-build"));
}

function assertNeonFeatureFlags(cargo: any) {
  assert.deepEqual(cargo['build-dependencies']['neon-build'].features, ["n-api"]);
  assert.deepEqual(cargo.dependencies.neon.features, ["n-api"]);
}

describe('neon new', function() {
  setup();

  it('should create a new project', function(done) {
    spawnNeonNew(this, 'my-app', {desc: 'My new app!'}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assert.propertyVal(pkg, 'name', 'my-app');
      assert.propertyVal(pkg, 'version', '0.1.0');
      assert.propertyVal(pkg, 'description', 'My new app!');
      assert.propertyVal(pkg, 'license', 'MIT');
      assert.nestedProperty(pkg, 'dependencies.neon-cli');

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
      let { pkg, cargo } = manifests(this.cwd, 'my-package');
      assert.propertyVal(pkg, 'name', '@me/my-package');

      let readme = readFile(this.cwd, 'my-package/README.md');
      assert.match(readme, /@me\/my-package/);

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
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assert.propertyVal(pkg, 'description', 'Foo "bar"');
      assert.nestedPropertyVal(pkg, 'repository.url', 'http://www.example.com/foo.git?bar=%22baz%22');
      assert.propertyVal(pkg, 'author', 'Foo "Bar" Baz <haywoodjabuzoff@example.com>');
      assert.includeDeepMembers(cargo.package.authors, ['Foo "Bar" Baz <haywoodjabuzoff@example.com>'])
      done();
    });
  });

  it('supports relative paths to Neon source directories', function(done) {
    spawnNeonNew(this, 'my-app', {neon: '.'}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assertLocalNeonCli(pkg);
      assertRelativeNeonCli(pkg);
      assertLocalNeon(cargo);
      done();
    });
  });

  it('supports absolute paths to Neon source directories', function(done) {
    spawnNeonNew(this, 'my-app', {neon: path.resolve('.')}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assertLocalNeonCli(pkg);
      assertAbsoluteNeonCli(pkg);
      assertLocalNeon(cargo);
      done();
    });
  });

  it('supports paths to Neon source directories and feature flags', function(done) {
    spawnNeonNew(this, 'my-app', {neon: '.', features: 'n-api'}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assertLocalNeonCli(pkg);
      assertLocalNeon(cargo);
      assertNeonFeatureFlags(cargo);
      done();
    });
  });

  it('supports Neon feature flags', function(done) {
    spawnNeonNew(this, 'my-app', {features: 'n-api'}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assertNormalNeonCli(pkg);
      assertNeonFeatureFlags(cargo);
      done();
    });
  });

  it('supports semver ranges of Neon', function(done) {
    spawnNeonNew(this, 'my-app', {neon: '^0.2'}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assert.nestedPropertyVal(pkg, 'dependencies.neon-cli', '^0.2');
      assert.nestedPropertyVal(cargo, 'dependencies.neon', '^0.2');
      assert.nestedPropertyVal(cargo, 'build-dependencies.neon-build', '^0.2');
      done();
    });
  });

  it('supports semver ranges of Neon with feature flags', function(done) {
    spawnNeonNew(this, 'my-app', {neon: '^0.2', features: 'n-api'}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assert.nestedPropertyVal(pkg, 'dependencies.neon-cli', '^0.2');
      assert.nestedPropertyVal(cargo, 'dependencies.neon.version', '^0.2');
      assert.deepEqual(cargo.dependencies.neon.features, ['n-api']);
      assert.nestedPropertyVal(cargo, 'build-dependencies.neon-build.version', '^0.2');
      assert.deepEqual(cargo['build-dependencies']['neon-build'].features, ['n-api']);
      done();
    });
  });

  it('supports specific semver versions of Neon', function(done) {
    spawnNeonNew(this, 'my-app', {neon: '0.2.2'}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assert.nestedPropertyVal(pkg, 'dependencies.neon-cli', '^0.2.2');
      assert.nestedPropertyVal(cargo, 'dependencies.neon', '0.2.2');
      assert.nestedPropertyVal(cargo, 'build-dependencies.neon-build', '0.2.2');
      done();
    });
  });

  it('supports specific semver versions of Neon with feature flags', function(done) {
    spawnNeonNew(this, 'my-app', {neon: '0.2.2', features: 'n-api'}, () => {
      let { pkg, cargo } = manifests(this.cwd, 'my-app');
      assert.nestedPropertyVal(pkg, 'dependencies.neon-cli', '^0.2.2');
      assert.nestedPropertyVal(cargo, 'dependencies.neon.version', '0.2.2');
      assert.deepEqual(cargo.dependencies.neon.features, ['n-api']);
      assert.nestedPropertyVal(cargo, 'build-dependencies.neon-build.version', '0.2.2');
      assert.deepEqual(cargo['build-dependencies']['neon-build'].features, ['n-api']);
      done();
    });
  });

});
