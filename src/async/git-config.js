import gitconfig from 'git-config';
import { denodeify } from 'rsvp';

export default denodeify(gitconfig);
