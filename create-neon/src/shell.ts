import { spawn } from 'child_process';

/**
 * Transparently shell out to an executable with a list of arguments.
 * All stdio is inherited directly from the current process.
 */
export default function shell(cmd: string, args: string[], cwd: string): Promise<undefined> {
  let child = spawn(cmd, args, { stdio: 'inherit', shell: true, cwd });

  let resolve: (result: undefined) => void;
  let reject: (error: Error) => void;

  let result: Promise<undefined> = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });

  child.on('exit', (code) => {
    if (code == null) {
      process.exit(1);
    }
    if (code !== 0) {
      process.exit(code);
    }
    resolve(undefined);
  });

  child.on('error', (error) => {
    reject(error);
  });

  return result;
}
