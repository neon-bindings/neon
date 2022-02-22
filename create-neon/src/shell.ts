import { spawn } from "child_process";
import { promises as fs } from "fs";
import path from "path";

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

  child.on("exit", async (code) => {
    if (code == null) {
      reject(Error(`error code: ${code}`));
    }
    if (code !== 0) {
      //This will catch answering no and many other failures
      reject(Error(`error code: ${code}`));
    }

    if (code === 0) {
      try {
        let data = await fs.readFile(path.join(cwd, "package.json"), "utf8");
        //Testing whether npm init was successful.
        //It will catch Ctrl+C and many other failures
        let { description, author, license } = JSON.parse(data);
        if ([description, author, license].includes(undefined)) {
          reject(Error(`error code: ${code}`));
        }
      } catch (e: any) {
        reject(e);
      }
    }

    resolve(undefined);
  });

  child.on("error", async (error) => {
    reject(error);
  });
  return result;
}
