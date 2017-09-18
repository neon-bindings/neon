import { readFileSync } from 'fs';
import { resolve } from 'path';

export function readFile(...args: string[]) {
  return readFileSync(resolve(...args), 'utf8');
}
