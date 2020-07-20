import * as async from './async/child_process';
import * as childProcess from 'child_process';

export type Toolchain = 'default' | 'stable' | 'nightly' | 'beta';

function toolchainPrefix(toolchain: Toolchain = 'default') {
  return toolchain === 'default' ? [] : ["+" + toolchain];
}

export function spawnSync(tool: string,
                          args: string[],
                          toolchain: Toolchain = 'default',
                          options?: childProcess.SpawnOptions)
{
  return childProcess.spawnSync(tool, toolchainPrefix(toolchain).concat(args), options);
}

export function spawn(tool: string,
                      args: string[],
                      toolchain: Toolchain = 'default',
                      options?: childProcess.SpawnOptions)
{
  return async.spawn(tool, toolchainPrefix(toolchain).concat(args), options);
}
