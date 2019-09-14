import Project from '../project';

export default async function neon_clean(root: string) {
  let project = await Project.create(root);
  await project.clean();
}
