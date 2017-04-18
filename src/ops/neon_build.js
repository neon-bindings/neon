import Project from '../project';

export default async function neon_build(root, toolchain, release, abi) {
  let project = new Project(root);
  await project.build(toolchain, release, abi);
}
