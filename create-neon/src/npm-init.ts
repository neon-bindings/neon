import shell from './shell';

export default async function npmInit(): Promise<number> {
  let code = await shell('npm', ['init']);
  
  if (code == null) {
    process.exit(1);
  }
  
  if (code !== 0) {
    process.exit(code);
  }
  
  return 0;
}
