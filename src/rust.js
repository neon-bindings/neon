import * as async from './async/child_process';
import * as child_process from 'child_process';

function toolchainPrefix(toolchain) {
  return toolchain === 'default' ? [] : ["+" + toolchain];
}

export function spawnSync(tool, args, toolchain = 'default', options) {
  return child_process.spawnSync(tool, toolchainPrefix(toolchain).concat(args), options);
}

export function spawn(tool, args, toolchain = 'default', options) {
  return async.spawn(tool, toolchainPrefix(toolchain).concat(args), options);
}
