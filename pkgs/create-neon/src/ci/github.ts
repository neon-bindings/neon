import { CI } from '../ci.js';
import path from 'node:path';

const TEMPLATES: Record<string, string> = {
  ".env.hbs": path.join(".github", "workflows", ".env"),
  "build.yml.hbs": path.join(".github", "workflows", "build.yml"),
  "comments.yml.hbs": path.join(".github", "workflows", "comments.yml"),
  "publish.yml.hbs": path.join(".github", "workflows", "publish.yml"),
  "test.yml.hbs": path.join(".github", "workflows", "test.yml")
};

export class GitHub implements CI {
  constructor() { }

  readonly type: string = "github";

  templates(): Record<string, string> {
    return TEMPLATES;
  }
}
