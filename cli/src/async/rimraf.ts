import _rimraf from 'rimraf';
import { promisify } from 'util';

export const rimraf = promisify(_rimraf);
