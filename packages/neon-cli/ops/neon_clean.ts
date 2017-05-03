import Project from '../project';

export default async function neon_clean(root: string) {
  let project = new Project(root);
  await project.clean();
}
