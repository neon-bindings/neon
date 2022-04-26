import { ChildProcess } from "child_process";
import { PassThrough, Readable, Writable } from "stream";
import { StringDecoder } from "string_decoder";
import readStream from "stream-to-string";

function readChunks(input: Readable): Readable {
  let output = new PassThrough({ objectMode: true });
  let decoder = new StringDecoder("utf8");
  input.on("data", (data) => {
    output.write(decoder.write(data));
  });
  input.on("close", () => {
    output.write(decoder.end());
    output.destroy();
  });
  return output;
}

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

async function* run(
  script: Record<string, string>,
  stdin: Writable,
  stdout: Readable
) {
  let lines = new LinesBuffer();

  let keys = Object.keys(script);
  let i = 0;
  for await (let chunk of readChunks(stdout)) {
    lines.add(splitLines(chunk));
    let found = lines.find((line) => line.startsWith(keys[i]));
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
  script: Record<string, string>
): Promise<void> {
  let output: string[][] = [];
  for await (let lines of run(script, child.stdin!, child.stdout!)) {
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
