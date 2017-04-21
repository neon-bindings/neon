let logger = () => { };

export function setup(newLogger) {
  logger = newLogger;
}

export default function log(msg) {
  logger(msg);
}
