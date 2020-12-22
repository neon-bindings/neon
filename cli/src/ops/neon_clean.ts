import Project from '../project';

export default async function neon_clean(root: string, crate: string = 'native') {
  let project = await Project.create(root, {crate});
  await project.clean();
}
