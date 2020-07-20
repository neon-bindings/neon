declare module 'git-config' {
  export type Config = {
    user: {
      name?: string,
      email?: string
    },
    github: {
      user?: string
    }
  }
  export function sync(gitFile?: string): Config;
}
