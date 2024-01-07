import commandLineUsage from 'command-line-usage';
import chalk from 'chalk';

function pink(text: string): string {
  return chalk.bold.hex('#e75480')(text);
}

function green(text: string): string {
  return chalk.bold.greenBright(text);
}

function blue(text: string): string {
  return chalk.bold.cyanBright(text);
}

function yellow(text: string): string {
  return chalk.bold.yellowBright(text);
}

function bold(text: string): string {
  return chalk.bold(text);
}

function mainUsage(): string {
  const sections = [
    {
      content: `✨ ${pink('create-neon:')} create a new Neon project with zero configuration ✨`,
      raw: true
    },
    {
      header: green('Examples:'),
      content: [
        `${blue('$')} ${bold('npm init neon my-package')}`,
        '',
        'Create a Neon project `my-package`.',
        '',
        `${blue('$')} ${bold('npm init neon --lib my-lib')}`,
        '',
        'Create a Neon library `my-lib`, pre-configured to publish pre-builds for common Node target platforms as binary packages under the `@my-lib` org. The generated project includes GitHub CI/CD actions for testing and publishing.',
        '',
        `${blue('$')} ${bold('npm init neon --lib my-library --target desktop')}`,
        '',
        'Similar but configured to target just common Node desktop platforms.'
      ]
    },
    {
      header: blue('Usage:'),
      content: `${blue('$')} npm init neon [--lib] [--bins <bins>] [--ci <ci>] [--target <tgt>]* <pkg>`
    },
    {
      header: yellow('Options:'),
      content: [
        { name: '--lib', summary: 'Configure package as a library. (Implied defaults: `--bins npm` and `--ci github`)' },
        { name: '--bins npm[:@<org>]', summary: 'Configure for pre-built binaries published to npm. (Default org: <pkg>)' },
        { name: '--bins none', summary: 'Do not configure for pre-built binaries. (Default)' },
        { name: '--target <tgt>', summary: 'May be used to specify one or more targets for pre-built binaries. (Default: common)' },
        { name: '--ci github', summary: 'Generate CI/CD configuration for GitHub Actions. (Default)' },
        { name: '--ci none', summary: 'Do not generate CI/CD configuration.' },
        { name: '<pkg>', summary: 'Package name.' }
      ]
    }
  ];

  return commandLineUsage(sections).trim();
}

export function printMainUsage() {
  console.error(mainUsage());
}

export function printErrorWithUsage(e: any) {
  console.error(mainUsage());
  console.error();
  printError(e);
}

export function printError(e: any) {
  console.error(chalk.bold.red("error:") + " " + ((e instanceof Error) ? e.message : String(e)));
}
