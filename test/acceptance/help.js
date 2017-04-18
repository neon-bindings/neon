import { setup } from '../support/acceptance';

function describeHelp(cmd, should, test, args) {
  describe(cmd, function() {
    setup('stderr');

    it(should, function(done) {
      test(this.spawn(args), done);
    });
  });
}

function testHelp(proc, done) {
  return proc
    .wait("Neon")
    .wait("native Node.js modules with Rust")
    .wait("Synopsis")
    .wait("$ neon [options] <command>")
    .wait("Command List")
    .wait("new")
    .wait("build")
    .wait("clean")
    .wait("version")
    .wait("help")
    .run(err => {
      if (err) throw err;
      done();
    });
}

describeHelp("neon help",   "should print neon usage", testHelp, ['help']);
describeHelp("neon --help", "should print neon usage", testHelp, ['--help']);
describeHelp("neon -h",     "should print neon usage", testHelp, ['-h']);

function testHelpClean(proc, done) {
  return proc
    .wait("neon clean")
    .wait("Remove build artifacts from a Neon project.")
    .wait("Synopsis")
    .wait("$ neon clean [options]")
    .wait("$ neon clean [options] module ...")
    .wait("Options")
    .wait("-p, --path")
    .run(err => {
      if (err) throw err;
      done();
    });
}

describeHelp("neon help clean",   "should print `neon clean` usage", testHelpClean, ['help', 'clean']);
describeHelp("neon clean --help", "should print `neon clean` usage", testHelpClean, ['clean', '--help']);
describeHelp("neon clean -h",     "should print `neon clean` usage", testHelpClean, ['clean', '-h']);
describeHelp("neon --help clean", "should print `neon clean` usage", testHelpClean, ['--help', 'clean']);
describeHelp("neon -h clean",     "should print `neon clean` usage", testHelpClean, ['-h', 'clean']);

function testHelpVersion(proc, done) {
  return proc
    .wait("neon version")
    .wait("Display the Neon version.")
    .wait("Synopsis")
    .wait("$ neon version")
    .run(err => {
      if (err) throw err;
      done();
    });
}

describeHelp("neon help version",   "should print `neon version` usage", testHelpVersion, ['help', 'version']);
describeHelp("neon version --help", "should print `neon version` usage", testHelpVersion, ['version', '--help']);
describeHelp("neon version -h",     "should print `neon version` usage", testHelpVersion, ['version', '-h']);
describeHelp("neon --help version", "should print `neon version` usage", testHelpVersion, ['--help', 'version']);
describeHelp("neon -h version",     "should print `neon version` usage", testHelpVersion, ['-h', 'version']);

function testHelpNew(proc, done) {
  return proc
    .wait("neon new")
    .wait("Create a new Neon project")
    .wait("Synopsis")
    .wait("$ neon new [@<scope>/]<name>")
    .run(err => {
      if (err) throw err;
      done();
    });
}

describeHelp("neon help new",   "should print `neon new` usage", testHelpNew, ['help', 'new']);
describeHelp("neon new --help", "should print `neon new` usage", testHelpNew, ['new', '--help']);
describeHelp("neon new -h",     "should print `neon new` usage", testHelpNew, ['new', '-h']);
describeHelp("neon --help new", "should print `neon new` usage", testHelpNew, ['--help', 'new']);
describeHelp("neon -h new",     "should print `neon new` usage", testHelpNew, ['-h', 'new']);

function testHelpBuild(proc, done) {
  return proc
    .wait("neon build")
    .wait("(Re)build a Neon project")
    .wait("Synopsis")
    .wait("$ neon build [options]")
    .wait("$ neon build [options] module ...")
    .wait("Options")
    .wait("-r, --rust")
    .wait("-d, --debug")
    .wait("-p, --path")
    .run(err => {
      if (err) throw err;
      done();
    });
}

describeHelp("neon help build",   "should print `neon build` usage", testHelpBuild, ['help', 'build']);
describeHelp("neon build --help", "should print `neon build` usage", testHelpBuild, ['build', '--help']);
describeHelp("neon build -h",     "should print `neon build` usage", testHelpBuild, ['build', '-h']);
describeHelp("neon --help build", "should print `neon build` usage", testHelpBuild, ['--help', 'build']);
describeHelp("neon -h build",     "should print `neon build` usage", testHelpBuild, ['-h', 'build']);

function testHelpHelp(proc, done) {
  return proc
    .wait("neon help")
    .wait("Get help about a Neon command")
    .wait("Synopsis")
    .wait("$ neon help [command]")
    .run(err => {
      if (err) throw err;
      done();
    });
}

describeHelp("neon help help",   "should print `neon help` usage", testHelpHelp, ['help', 'help']);
describeHelp("neon help --help", "should print `neon help` usage", testHelpHelp, ['help', '--help']);
describeHelp("neon help -h",     "should print `neon help` usage", testHelpHelp, ['help', '-h']);
describeHelp("neon --help help", "should print `neon help` usage", testHelpHelp, ['--help', 'help']);
describeHelp("neon -h help",     "should print `neon help` usage", testHelpHelp, ['-h', 'help']);
