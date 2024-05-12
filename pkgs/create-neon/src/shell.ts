import { ChildProcess, spawn } from 'node:child_process';
import { PassThrough, Readable, Writable } from 'node:stream';
import { StringDecoder } from 'node:string_decoder';
import readline from 'node:readline/promises';

export function readChunks(input: Readable): Readable {
  let output = new PassThrough({ objectMode: true });
  let decoder = new StringDecoder('utf8');
  input.on('data', (data) => {
    output.write(decoder.write(data));
  });
  input.on('close', () => {
    output.write(decoder.end());
    output.end();
  });
  return output;
}

class NpmInit {
  private _tmp: string;
  private _regexp: RegExp;
  private _child: ChildProcess;

  constructor(interactive: boolean, args: string[], cwd: string, tmp: string) {
    this._tmp = tmp;
    this._regexp = new RegExp(tmp + ".");
    this._child = spawn('npm', ['init', ...args], {
      stdio: ['inherit', 'pipe', 'inherit'],
      shell: true,
      cwd
    });
    this.filterStdout({ interactive }).then(() => {});
  }

  exit(): Promise<number | null> {
    let resolve: (code: number | null) => void;
    const result: Promise<number | null> = new Promise((res) => {
      resolve = res;
    });
    this._child.on('exit', (code) => {
      resolve(code);
    });
    return result;
  }

  async filterStdout(opts: { interactive: boolean }) {
    // We'll suppress the `npm init` interactive prelude text,
    // in favor of printing our own create-neon version of the text.
    let inPrelude = opts.interactive;

    for await (const chunk of readChunks(this._child.stdout!)) {
      const lines = (chunk as string).split(/\r?\n/);
      if (opts.interactive && inPrelude) {
        // The first interactive prompt marks the end of the prelude.
        const i = lines.findIndex(line => line.match(/^[a-z ]+:/));

        // No prompt? We're still in the prelude so ignore and continue.
        if (i === -1) {
          continue;
        }

        // Ignore the prelude lines up to the first interactive prompt.
        lines.splice(0, i);
        inPrelude = false;
      }

      // Print out all the lines.
      lines.forEach((line, i) => {
        // Remove the temp dir from any paths printed out by `npm init`.
        process.stdout.write(line.replace(this._regexp, ""));
        if (i < lines.length - 1) {
          process.stdout.write('\n');
        }
      });
    }
  }
}

export function npmInit(
  interactive: boolean,
  args: string[],
  cwd: string,
  tmp: string
): Promise<number | null> {
  return (new NpmInit(interactive, args, cwd, tmp)).exit();
}

export type Parser<T> = (v: string) => T;

export function oneOf<T extends {}>(opts: T): Parser<T[keyof T]> {
  return (v: string) => {
    for (const key in opts) {
      if (v === key) {
        return opts[key];
      }
    }
    throw new Error('parse error');
  };
}

export interface Question<T> {
  prompt: string,
  parse: Parser<T>,
  default: T,
  error?: string
};

export class Dialog {
  private _rl: readline.Interface | undefined;

  constructor() {
    this._rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  }

  private rl(): readline.Interface {
    if (!this._rl) {
      throw new Error("dialog already ended");
    }
    return this._rl;
  }

  end() {
    this.rl().close();
    this._rl = undefined;
  }

  async ask<T>(opts: Question<T>): Promise<T> {
    while (true) {
      try {
        const answer = (await this.rl().question(`neon ${opts.prompt}: (${String(opts.default)}) `)).trim();
        return answer === "" ? opts.default : opts.parse(answer);
      } catch (_ignored) {
        if (opts.error) {
          console.log(`Sorry, ${opts.error}`);
        }
      }
    }
  }
}
