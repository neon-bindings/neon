import inquirer from 'inquirer';
import { Promise } from 'rsvp';

export function prompt(questions) {
  return new Promise(function(resolve, reject) {
    inquirer.prompt(questions, resolve).process.subscribe(() => {}, reject, () => {});
  });
}
