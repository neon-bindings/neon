import { spawn } from 'child_process';

/**
 * Transparently shell out to an executable with a list of arguments.
 * All stdio is inherited directly from the current process.
 */
export default function shell(cmd: string, args: string[]): Promise<number | null> {
  let child = spawn(cmd, args, { stdio: 'inherit' });

  let resolve: (result: number | null) => void;
  let reject: (error: Error) => void;

  let result: Promise<number | null> = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });

  child.on('exit', (code) => {
    resolve(code);
  });

  child.on('error', (error) => {
    reject(error);
  });

  return result;
}
