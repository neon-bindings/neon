import fs from 'fs';
import quicklyCopyFile from 'quickly-copy-file';
import mkdirp from 'mkdirp';
import rimraf from 'rimraf';
import { denodeify } from 'rsvp';

export let readFile = denodeify(fs.readFile);
export let writeFile = denodeify(fs.writeFile);

export let mkdirs = denodeify(mkdirp);
export let copy = quicklyCopyFile;
export let remove = denodeify(rimraf);
