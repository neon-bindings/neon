import Project from '../project';

export default async function neon_clean(root) {
  let project = new Project(root);
  await project.clean();
}
