import { promisify } from 'util';
import child from 'child_process';

export function spawn(command: string, args?: string[], options?: child.SpawnOptions): Promise<number> {
  return new Promise((resolve, reject) => {
      let ps = child.spawn(command, args || [], options!);
      ps.on('error', reject);
      ps.on('close', resolve);
  });
}

export const execFile = promisify(child.execFile);
