import fs from 'fs';
import extra from 'fs-extra';
import { denodeify } from 'rsvp';

export let readFile = denodeify(fs.readFile);
export let writeFile = denodeify(fs.writeFile);

export let mkdirs = denodeify(extra.mkdirs);
export let copy = denodeify(extra.copy);
export let remove = denodeify(extra.remove);
