import chalk from 'chalk';

export let project = chalk.cyan.bold;
export let command = chalk.green.bold;
export let path = chalk.cyan;

export function error(msg) {
  return chalk.bgBlack.cyan("neon") + " " +
         chalk.bgBlack.red("ERR!") + " " +
         msg;
}

export function info(msg) {
  return chalk.bgBlack.cyan("neon") + " " +
         chalk.bgBlack.gray("info") + " " +
         chalk.gray(msg);
}
