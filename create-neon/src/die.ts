import { promises as fs } from "fs";

function deleteNeonDir(dir: string): Promise<void> {
  return fs.rm(dir, { force: true, recursive: true });
}

export default async function die(
  message: string,
  tmpFolderName: string
): Promise<never> {
  console.error(`❌ ${message}`);
  if (tmpFolderName) {
    await deleteNeonDir(tmpFolderName);
  }
  process.exit(1);
}
