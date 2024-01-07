import { CI } from '../ci.js';
import path from 'node:path';

const TEMPLATES: Record<string, string> = {
  "publish.yml.hbs": path.join(".github", "workflows", "publish.yml"),
  "test.yml.hbs": path.join(".github", "workflows", "test.yml")
};

export class GitHub implements CI {
  constructor() { }

  templates(): Record<string, string> {
    return TEMPLATES;
  }
}
