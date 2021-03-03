import { readFile } from 'fs/promises';
import npmInit from '../npm-init';

async function main() {
  await npmInit();

  let name: string;

  try {
    let json = JSON.parse(await readFile('package.json', 'utf8'));
    name = json.name;
  } catch (err) {
    console.error("Could not read `package.json`: " + err.message);
    process.exit(1);
  }

  console.log(name);  
}

main();
