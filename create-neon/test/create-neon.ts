import { assert } from 'chai';
import { Readable, PassThrough, Writable } from 'stream';
//import * as readline from 'readline';
import { ChildProcess, spawn } from 'child_process';
import execa from 'execa';
import * as path from 'path';
import { readFile, rmdir } from 'fs/promises';
import { StringDecoder } from 'string_decoder';
import * as TOML from 'toml';

function readChunks(input: Readable): Readable {
  let output = new PassThrough({ objectMode: true });
  let decoder = new StringDecoder('utf8');
  input.on('data', data => {
    output.write(decoder.write(data));
  });
  input.on('close', () => {
    output.write(decoder.end());
    output.destroy();
  });
  return output;
}

function splitLines(s: string): string[] {
  return s.split(/([^\n]*\r?\n)/).filter(x => x);
}

function isCompleteLine(s: string): boolean {
  return s.endsWith('\n');
}

class LinesBuffer {

  // INVARIANT: (this.buffer.length > 0) &&
  //            !isCompleteLine(this.buffer[this.buffer.length - 1])
  // In other words, the last line in the buffer is always incomplete.
  private buffer: string[];

  constructor() {
    this.buffer = [""];
  }

  add(lines: string[]) {
    if (isCompleteLine(lines[lines.length - 1])) {
      lines.push("");
    }
    this.buffer[this.buffer.length - 1] += lines.shift();
    this.buffer = this.buffer.concat(lines);
  }

  find(p: (s: string) => boolean): string[] | null {
    let index = this.buffer.findIndex(p);
    if (index === -1) {
      return null;
    }
    let extracted = this.buffer.splice(0, index + 1);
    if (this.buffer.length === 0) {
      this.buffer.push("");
    }
    return extracted;
  }
}

async function* dialog(script: Record<string, string>, stdin: Writable, stdout: Readable) {
  let lines = new LinesBuffer();

  let keys = Object.keys(script);
  let i = 0;
  for await (let chunk of readChunks(stdout)) {
    lines.add(splitLines(chunk));
    let found = lines.find(line => line.startsWith(keys[i]));
    if (found) {
      stdin.write(script[keys[i]] + "\n");
      yield found;
      i++;
      if (i >= keys.length) {
        break;
      }
    }
  }
}

const NODE: string = process.execPath;
const CREATE_NEON = path.join(__dirname, '..', 'dist', 'src', 'bin', 'create-neon.js');

describe('Command-line argument validation', () => {
  it('requires an argument', async () => {
    try {
      await execa(NODE, [CREATE_NEON]);
      assert.fail("should fail when no argument is supplied");
    } catch (expected) {
      assert.isTrue(true);
    }
  });

  it('rejects extra arguments', async () => {
    try {
      await(execa(NODE, [CREATE_NEON, 'name', 'ohnoanextraargument']));
      assert.fail("should fail when too many arguments are supplied");
    } catch (expected) {
      assert.isTrue(true);
    }
  });

  it('fails if the directory already exists', async () => {
    try {
      await execa(NODE, [CREATE_NEON, 'dist']);
      assert.fail("should fail when directory exists");
    } catch (expected) {
      assert.isTrue(true);
    }
  });
});

const PROJECT = 'create-neon-test-project';

async function start(): Promise<ChildProcess> {
  await rmdir(PROJECT, { recursive: true });
  return spawn(NODE, [CREATE_NEON, PROJECT]);
}

/*
function timeout(ms: number): Promise<void> {
  let resolve: () => void;
  let result: Promise<void> = new Promise(res => { resolve = res; });
  setTimeout(() => { resolve() }, ms);
  return result;
}
*/

function exit(child: ChildProcess): Promise<number | null> {
  let resolve: (code: number | null) => void;
  let result: Promise<number | null> = new Promise(res => { resolve = res; });
  child.on('exit', code => {
    resolve(code);
  });
  return result;
}

const DEFAULTS_SCRIPT = {
  'package name:':   '',
  'version:':        '',
  'description:':    '',
  'git repository:': '',
  'keywords:':       '',
  'author:':         '',
  'license:':        '',
  'Is this OK?':     ''
};

describe('Project creation', () => {
  it('succeeds with all default answers', async () => {
    let child = await start();
    for await (let _ of dialog(DEFAULTS_SCRIPT, child.stdin!, child.stdout!)) { }

    let code = await exit(child);

    assert.strictEqual(code, 0);

    let json = JSON.parse(await readFile(path.join(PROJECT, 'package.json'), { encoding: 'utf8' }));

    assert.strictEqual(json.name, PROJECT);
    assert.strictEqual(json.main, 'index.node');
    assert.strictEqual(json.version, '0.1.0');
    assert.strictEqual(json.scripts.test, 'cargo test');
    assert.strictEqual(json.license, 'ISC');
    assert.strictEqual(json.description, '');
    assert.strictEqual(json.author, '');

    let toml = TOML.parse(await readFile(path.join(PROJECT, 'Cargo.toml'), { encoding: 'utf8' }));

    assert.strictEqual(toml.package.name, PROJECT);
    assert.strictEqual(toml.package.version, '0.1.0');
    assert.strictEqual(toml.package.license, 'ISC');
    assert.deepEqual(toml.lib['crate-type'], ['cdylib']);
  });

});
