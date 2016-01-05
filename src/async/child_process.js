import child from 'child_process';
import { Promise } from 'rsvp';

export function spawn(...args) {
  return new Promise((resolve, reject) => {
    let ps = child.spawn(...args);
    ps.on('error', reject);
    ps.on('close', resolve);
  });
}
