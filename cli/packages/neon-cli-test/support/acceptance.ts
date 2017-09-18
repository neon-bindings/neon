import tmp = require('tmp');
import * as path from 'path';
import { spawn } from 'nexpect';

const NODE = process.execPath;
const NEON = path.resolve('bin/cli.js');

export interface Spawnable {
  cwd: string;
  spawn(args: string | string[]): NexpectChain;
  spawn(args: string | string[], options: NexpectOptions): NexpectChain;
  spawn(args: string | string[], params: string[], options: NexpectOptions): NexpectChain;
}

export interface NexpectOptions {
  cwd?: string,
  env?: object,
  ignoreCase?: boolean,
  stripColors?: boolean,
  stream?: 'stdout' | 'stderr' | 'all',
  verbose?: boolean
}

export interface NexpectChain {
  expect(expectation: string | RegExp): NexpectChain;
  wait(expectation: string | RegExp): NexpectChain;
  sendline(line: string): NexpectChain;
  sendEof(): NexpectChain;
  run(callback: (error: Error | null, output: string[], exit: number | string) => void): void;
}

function isSpawnable<T>(x: T): x is T & Spawnable {
  return !!x &&
         typeof x === 'object' &&
         typeof (x as any).cwd === 'string' &&
         typeof (x as any).spawn === 'function';
}

export function spawnable(obj: Mocha.ITestCallbackContext): Mocha.ITestCallbackContext & Spawnable {
  if (!isSpawnable(obj)) {
    throw new TypeError("mocha callback run without running setup()");
  }
  return obj;
}

export function setup(stream: string = 'all') {
  let tmpobj: tmp.SynchrounousResult;

  beforeEach(function() {
    tmpobj = tmp.dirSync({ unsafeCleanup: true });

    this.cwd = tmpobj.name;
    this.spawn = (args: string[]) => spawn(NODE, [NEON].concat(args), { cwd: this.cwd, stream, stripColors: true });
  });

  afterEach(function() {
    delete this.cwd;
    delete this.spawn;

    tmpobj.removeCallback();
  });
};
