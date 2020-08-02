import chalk from 'chalk';

export let project = chalk.cyan.bold;
export let command = chalk.green.bold;
export let path = chalk.cyan;

export function error(msg: string): string {
  return chalk.bgBlack.cyan("neon") + " " +
         chalk.bgBlack.red("ERR!") + " " +
         msg;
}

export function info(msg: string): string {
  return chalk.bgBlack.cyan("neon") + " " +
         chalk.bgBlack.gray("info") + " " +
         chalk.gray(msg);
}
