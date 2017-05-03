export type Logger = (msg: string) => void;

let logger: Logger = () => { };

export function setup(newLogger: Logger) {
  logger = newLogger;
}

export default function log(msg: string) {
  logger(msg);
}
