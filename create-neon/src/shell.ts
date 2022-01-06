import { spawn } from "child_process";
import fs from "fs/promises";
/**
 * Transparently shell out to an executable with a list of arguments.
 * All stdio is inherited directly from the current process.
 */
export default function shell(
  cmd: string,
  args: string[],
  cwd: string
): Promise<undefined> {
  let child = spawn(cmd, args, { stdio: "inherit", shell: true, cwd });

  let resolve: (result: undefined) => void;
  let reject: (error: Error) => void;

  let result: Promise<undefined> = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });

  function deleteNeonDir(): Promise<void> {
    return fs.rm(cwd, { force: true, recursive: true });
  }

  child.on("exit", async (code) => {
    if (code == null) {
      await deleteNeonDir();
      process.exit();
    }
    if (code !== 0) {
      await deleteNeonDir();
      process.exit(code);
    }

    if (code === 0) {
      try {
          let data = await fs.readFile(`${cwd}/package.json`,'utf8')
          let { description, author, license } = JSON.parse(data);
          if ([description, author, license].includes(undefined)) {
               throw new Error("Ctrl+C pressed");
               }
        } catch (e) {
           await deleteNeonDir();
           process.exit(e);
      }
    }
    resolve(undefined);
  });

  child.on("error", async (error) => {
    await deleteNeonDir();
    reject(error);
  });
  return result;
}
