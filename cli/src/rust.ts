import * as async from './async/child_process';
import * as child_process from 'child_process';

export type Toolchain = string | null;

function toolchainPrefix(toolchain: Toolchain = 'default') {
  return toolchain ? ["+" + toolchain] : [];
}

export function spawnSync(tool: string,
                          args: string[],
                          toolchain: Toolchain = 'default',
                          options?: child_process.SpawnOptions)
{
  return child_process.spawnSync(tool, toolchainPrefix(toolchain).concat(args), options);
}

export function spawn(tool: string,
                      args: string[],
                      toolchain: Toolchain = 'default',
                      options?: child_process.SpawnOptions)
{
  return async.spawn(tool, toolchainPrefix(toolchain).concat(args), options);
}
