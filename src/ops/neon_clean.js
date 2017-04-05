import { remove, copy, readFile } from '../async/fs';
import { spawn } from '../async/child_process';
import path from 'path';
import * as style from './style';

function cargo_clean(root) {
  let macos = process.platform === 'darwin';

  console.log(style.info("cargo clean"));

  return spawn("cargo", ["clean"], { cwd: path.resolve(root, 'native'), stdio: 'inherit', env: env });
}

export default async function neon_clean(root, profile) {
  let index = path.resolve(root, 'native', 'index.node');

  await remove(index);

  if (profile === 'all') {
    await cargo_clean(root);
  } else {
    await remove(path.resolve(root, 'native', 'target', profile));
  }
}
