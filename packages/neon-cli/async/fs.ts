import * as fs from 'fs';
import quicklyCopyFile = require('quickly-copy-file');
import RSVP = require('rsvp');
import rimraf = require('rimraf');
import mkdirp = require('mkdirp');

export let stat: (path: string) => Promise<fs.Stats>
  = RSVP.denodeify(fs.stat);

let rf: (path: string, options?: { encoding: string; flag?: string; }) => Promise<string | Buffer>
  = RSVP.denodeify<string | Buffer>(fs.readFile);

export function readFile(path: string, options: { encoding: string; flag?: string; }): Promise<string>;
export function readFile(path: string): Promise<Buffer>;
export function readFile(path: string, options?: { encoding: string; flag?: string; }): Promise<string | Buffer> {
  return rf(path, options);
}

export type WriteOptions = {
  encoding?: string,
  mode?: number,
  flag?: string
};

export let writeFile: (path: string, contents: string, options: WriteOptions) => Promise<void>
  = RSVP.denodeify<void>(fs.writeFile);

export let copy = quicklyCopyFile;

export let remove: (path: string) => Promise<void>
  = RSVP.denodeify<void>(rimraf);

export let mkdirs: (path: string) => Promise<void>
  = RSVP.denodeify<void>(mkdirp);
