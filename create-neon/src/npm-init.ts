import shell from './shell';

const NPM: string = process.env.npm_execpath || die('create-neon must be run from `npm init`');
const NODE: string = process.env.npm_node_execpath || die('create-neon must be run from `npm init`');

function die(msg: string): never {
  console.error(msg);
  process.exit(1);
}

export default async function npmInit(): Promise<number> {
  let code = await shell(NODE, [NPM, 'init']);
  
  if (code == null) {
    process.exit(1);
  }
  
  if (code !== 0) {
    process.exit(code);
  }
  
  return 0;
}
