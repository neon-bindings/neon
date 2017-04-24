import * as RSVP from 'rsvp';
import gitconfig = require('git-config');

let gc: () => Promise<gitconfig.Dict>
  = RSVP.denodeify<gitconfig.Dict>(gitconfig);

export default gc;
