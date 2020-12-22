import Project from '../project';
import * as rust from '../rust';

export default async function neon_build(root: string,
                                         toolchain: rust.Toolchain = 'default',
                                         crate: string = 'native',
                                         release: boolean,
                                         args: string[]) {
  let project = await Project.create(root, {crate});
  await project.build(toolchain, release, args);
}
