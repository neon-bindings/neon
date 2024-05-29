import { Creator, ProjectOptions } from "./creator.js";

export class AppCreator extends Creator {
  constructor(options: ProjectOptions) {
    super(options);
  }

  scripts(): Record<string, string> {
    return {
      test: "cargo test",
      "cargo-build": "cargo build --message-format=json > cargo.log",
      "cross-build": "cross build --message-format=json > cross.log",
      "postcargo-build": "neon dist < cargo.log",
      "postcross-build": "neon dist -m /target < cross.log",
      debug: "npm run cargo-build --",
      build: "npm run cargo-build -- --release",
      cross: "npm run cross-build -- --release",
    };
  }
}
