import tmp = require("tmp");
import * as path from "path";
import { spawn, SpawnChain } from "suspicion";

const NODE = process.execPath;
const NEON = path.join(
  __dirname,
  "..",
  "..",
  "..",
  "..",
  "cli",
  "bin",
  "cli.js"
);

export interface Spawnable {
  cwd: string;

  spawn(args: string[]): SpawnChain;
}

function isSpawnable<T>(x: T): x is T & Spawnable {
  return (
    !!x &&
    typeof x === "object" &&
    typeof (x as any).cwd === "string" &&
    typeof (x as any).spawn === "function"
  );
}

export function spawnable(obj: Mocha.Context): Mocha.Context & Spawnable {
  if (!isSpawnable(obj)) {
    throw new TypeError("mocha callback run without running setup()");
  }
  return obj;
}

export function setup(stream: string = "stdout") {
  let tmpobj: tmp.SynchrounousResult;

  beforeEach(function () {
    tmpobj = tmp.dirSync({ unsafeCleanup: true });

    this.cwd = tmpobj.name;
    this.spawn = (args: string[]) =>
      spawn(`"${NODE}"`, [`"${NEON}"`].concat(args), {
        shell: true,
        cwd: this.cwd,
        stream,
      });
  });

  afterEach(function () {
    delete this.cwd;
    delete this.spawn;

    tmpobj.removeCallback();
  });
}
