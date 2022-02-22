import rm from "rimraf";

/**
 * A shim for Node's newer builtin `{ recursive: true }` option. When
 * we only support new enough Node versions, we should be able to drop
 * this shim and directly use the builtin `rmdir` stdlib function.
 *
 * See: https://nodejs.org/api/fs.html#fs_fspromises_rmdir_path_options
 */
export default function rimraf(
  pattern: string,
  opts: rm.Options
): Promise<void> {
  let resolve: (result: void) => void;
  let reject: (error: Error) => void;
  let result: Promise<void> = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  rm(pattern, opts, (error) => {
    if (error) {
      reject(error);
    } else {
      resolve(undefined);
    }
  });
  return result;
}
