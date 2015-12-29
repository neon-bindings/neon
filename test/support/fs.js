import { readFileSync } from 'fs';
import { resolve } from 'path';

export function readFile() {
  return readFileSync(resolve(...arguments), 'utf8');
}
