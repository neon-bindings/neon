import Project from '../project';
import * as rust from '../rust';

export default async function neon_build(root: string,
                                         toolchain: rust.Toolchain,
                                         release: boolean,
                                         abi: string) {
  let project = new Project(root);
  await project.build(toolchain, release, abi);
}
