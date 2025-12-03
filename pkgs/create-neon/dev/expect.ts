import { ChildProcess } from "child_process";
import { Readable, Writable } from "stream";
import readStream from "stream-to-string";
import { readChunks } from "../src/shell.js";

function splitLines(s: string): string[] {
  return s.split(/([^\n]*\r?\n)/).filter((x) => x);
}

function isCompleteLine(s: string): boolean {
  return s.endsWith("\n");
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
    if (lines.length === 0) {
      return;
    }
    if (isCompleteLine(lines[lines.length - 1])) {
      lines.push("");
    }
    this.buffer[this.buffer.length - 1] += lines.shift();
    this.buffer = this.buffer.concat(lines);
  }

  // Finds and removes lines from the buffer up to and including
  // the first line that satisfies the predicate p.
  // Returns the extracted lines, or null if no such line exists.
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

interface Pattern {
  expect(session: Session): AsyncGenerator<string[]>;
}

type QA = { q: string; a: string };

class ExpectLine implements Pattern {
  optional: QA[];
  required: QA;

  constructor(optional: QA[], required: QA) {
    this.optional = optional;
    this.required = required;
  }

  async *expect(session: Session): AsyncGenerator<string[]> {
    // Use a wrapper stream so that early exit from the for-await loop doesn't
    // cancel the underlying stream.
    let stdout = Readable.toWeb(session.stdout).values({ preventCancel: true });

    for await (let chunk of stdout) {
      session.buffer.add(splitLines(chunk));

      let maxFound = -1;

      // Check for optional lines
      for (let i = 0; i < this.optional.length; i++) {
        let found = session.buffer.find((line) =>
          line.startsWith(this.optional[i].q)
        );
        if (found) {
          session.stdin.write(this.optional[i].a + "\n");
          maxFound = i;
          yield found;
        }
      }

      // Remove from queue any lines that were found
      this.optional.splice(0, maxFound + 1);

      // Check for required line
      let found = session.buffer.find((line) =>
        line.startsWith(this.required.q)
      );
      if (found) {
        session.stdin.write(this.required.a + "\n");
        yield found;
        return;
      }
    }
  }
}

// We don't currently have any scripts that end with an optional line, but if we did we'd need this class.
class ExpectEOF implements Pattern {
  optional: QA[];

  constructor(optional: QA[]) {
    this.optional = optional;
    throw new Error("Class not implemented.");
  }

  async *expect(session: Session): AsyncGenerator<string[]> {
    throw new Error("Method not implemented.");
  }
}

class Script {
  clauses: Pattern[];

  constructor(src: Record<string, string>) {
    this.clauses = [];

    let keys = Object.keys(src);
    let i = 0;
    let optional: QA[] = [];

    while (i < keys.length) {
      if (keys[i].startsWith("?")) {
        // Collect optional lines
        optional.push({ q: keys[i].substring(1).trim(), a: src[keys[i]] });
      } else {
        // Collect required line
        this.clauses.push(
          new ExpectLine(optional, { q: keys[i], a: src[keys[i]] })
        );
        optional = [];
      }
      i++;
    }

    if (optional.length > 0) {
      this.clauses.push(new ExpectEOF(optional));
    }
  }

  async *run(session: Session): AsyncGenerator<string[]> {
    for (let clause of this.clauses) {
      for await (let lines of clause.expect(session)) {
        yield lines;
      }
    }
  }
}

class Session {
  buffer: LinesBuffer;
  stdin: Writable;
  stdout: Readable;

  constructor(stdin: Writable, stdout: Readable) {
    this.buffer = new LinesBuffer();
    this.stdin = stdin;
    this.stdout = readChunks(stdout);
  }
}

function exit(child: ChildProcess): Promise<number | null> {
  let resolve: (code: number | null) => void;
  let result: Promise<number | null> = new Promise((res) => {
    resolve = res;
  });
  child.on("exit", (code) => {
    resolve(code);
  });
  return result;
}

export default async function expect(
  child: ChildProcess,
  src: Record<string, string>
): Promise<void> {
  let output: string[][] = [];
  let script = new Script(src);
  let session = new Session(child.stdin!, child.stdout!);
  for await (let lines of script.run(session)) {
    output.push(lines);
  }
  let stderr = await readStream(child.stderr!);
  let code = await exit(child);
  switch (code) {
    case null:
      throw new Error("child process interrupted");
    case 0:
      return;
    default:
      console.log("stderr: " + stderr.trim());
      console.log("stdout: " + JSON.stringify(output));
      throw new Error("child process exited with code " + code);
  }
}
