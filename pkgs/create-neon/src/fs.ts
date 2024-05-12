import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import { existsSync, rmSync } from 'node:fs';

export async function assertCanMkdir(dir: string) {
  // pretty lightweight way to check both that folder doesn't exist and
  // that the user has write permissions.
  await fs.mkdir(dir);
  await fs.rmdir(dir);
}

export async function mktemp(): Promise<string> {
  const tmpFolderName = await fs.mkdtemp("neon-");
  const tmpFolderAbsPath = path.join(process.cwd(), tmpFolderName);
  function cleanupTmp() {
    try {
      if (existsSync(tmpFolderAbsPath)) {
        rmSync(tmpFolderAbsPath, { recursive: true });
      }
    } catch (e) {
      console.error(`warning: could not delete ${tmpFolderName}: ${e}`);
    }
  }
  process.on('exit', cleanupTmp);
  process.on('SIGINT', cleanupTmp);
  process.on('uncaughtException', cleanupTmp);
  return tmpFolderName;
}
