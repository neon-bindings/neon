import { ChildProcess, spawn } from "node:child_process";
import { PassThrough, Readable, Writable } from "node:stream";
import { StringDecoder } from "node:string_decoder";
import readline from "node:readline/promises";

export function readChunks(input: Readable): Readable {
  let output = new PassThrough({ objectMode: true });
  let decoder = new StringDecoder("utf8");
  input.on("data", (data) => {
    output.write(decoder.write(data));
  });
  input.on("close", () => {
    output.write(decoder.end());
    output.end();
  });
  return output;
}

// A child process representing a modified `npm init` invocation:
// - If interactive, the initial prelude of stdout text is suppressed
//   so we can present a modified prelude for create-neon.
// - The process is being run in a temp subdirectory, so any output that
//   includes the temp directory in a path is transformed to remove it.
class NpmInit {
  private _regexp: RegExp;
  private _child: ChildProcess;

  constructor(interactive: boolean, args: string[], cwd: string, tmp: string) {
    this._regexp = new RegExp(tmp + ".");
    this._child = spawn("npm", ["init", ...args], {
      stdio: ["inherit", "pipe", "inherit"],
      shell: true,
      cwd,
    });
    this.filterStdout({ interactive }).then(() => {});
  }

  exit(): Promise<number | null> {
    let resolve: (code: number | null) => void;
    const result: Promise<number | null> = new Promise((res) => {
      resolve = res;
    });
    this._child.on("exit", (code) => {
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
        // If there's a prompt, it'll be at the end of the data chunk
        // since npm init will have flushed stdout to block on stdin.
        if (!lines[lines.length - 1].match(/^[a-z ]+:/)) {
          // We're still in the prelude so suppress all the lines and
          // wait for the next chunk of stdout data.
          continue;
        }

        // Suppress the prelude lines up to the prompt.
        lines.splice(0, lines.length - 1);
        inPrelude = false;
      }

      // Print out all the lines.
      lines.forEach((line, i) => {
        // Remove the temp dir from any paths printed out by `npm init`.
        process.stdout.write(line.replace(this._regexp, ""));
        if (i < lines.length - 1) {
          process.stdout.write("\n");
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
  return new NpmInit(interactive, args, cwd, tmp).exit();
}

export type Parser<T> = (v: string) => T;

export function oneOf<T extends {}>(opts: T): Parser<T[keyof T]> {
  return (v: string) => {
    for (const key in opts) {
      if (v === key) {
        return opts[key];
      }
    }
    throw new Error("parse error");
  };
}

export interface Question<T> {
  prompt: string;
  parse: Parser<T>;
  default: T;
  error?: string;
}

export class Dialog {
  private _rl: readline.Interface | undefined;

  constructor() {
    this._rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });
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
        const answer = (
          await this.rl().question(
            `neon ${opts.prompt}: (${String(opts.default)}) `
          )
        ).trim();
        return answer === "" ? opts.default : opts.parse(answer);
      } catch (_ignored) {
        if (opts.error) {
          console.log(`Sorry, ${opts.error}`);
        }
      }
    }
  }
}
