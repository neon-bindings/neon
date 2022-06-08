"use strict";

class ParseError extends Error {}

const NPM_ENV = "npm_package_name";
const EXPECTED_COMMAND = [
  "Missing command to execute.",
  [
    "cargo-cp-artifct -a cdylib my-crate index.node",
    "--",
    "cargo build --message-format=json-render-diagnostics",
  ].join(" "),
].join("\n");

function validateArtifactType(artifactType) {
  switch (artifactType) {
    case "b":
    case "bin":
      return "bin";
    case "c":
    case "cdylib":
      return "cdylib";
    case "d":
    case "dylib":
      return "dylib";
    default:
  }

  throw new ParseError(`Unexpected artifact type: ${artifactType}`);
}

function getArtifactName({ artifactType, crateName }) {
  return `${artifactType}:${crateName}`;
}

function getCrateNameFromEnv(env) {
  if (!env.hasOwnProperty(NPM_ENV)) {
    throw new ParseError(
      [
        `Could not find the \`${NPM_ENV}\` environment variable.`,
        "Expected to be executed from an `npm` command.",
      ].join(" ")
    );
  }

  const name = env[NPM_ENV];
  const firstSlash = name.indexOf("/");

  // This is a namespaced package; assume the crate is the un-namespaced version
  if (name[0] === "@" && firstSlash > 0) {
    return name.slice(firstSlash + 1);
  }

  return name;
}

function parse(argv, env) {
  const artifacts = {};
  let tokens = argv;

  function getNext() {
    if (!tokens.length) {
      throw new ParseError(EXPECTED_COMMAND);
    }

    const next = tokens[0];
    tokens = tokens.slice(1);
    return next;
  }

  function getArtifactType(token) {
    if (token[1] !== "-" && token.length === 3) {
      return validateArtifactType(token[2]);
    }

    return validateArtifactType(getNext());
  }

  function pushArtifact(artifact) {
    const name = getArtifactName(artifact);

    artifacts[name] = artifacts[name] || [];
    artifacts[name].push(artifact.outputFile);
  }

  while (tokens.length) {
    const token = getNext();

    // End of CLI arguments
    if (token === "--") {
      break;
    }

    if (
      token === "--artifact" ||
      (token.length <= 3 && token.startsWith("-a"))
    ) {
      const artifactType = getArtifactType(token);
      const crateName = getNext();
      const outputFile = getNext();

      pushArtifact({ artifactType, crateName, outputFile });
      continue;
    }

    if (token === "--npm" || (token.length <= 3 && token.startsWith("-n"))) {
      const artifactType = getArtifactType(token);
      const crateName = getCrateNameFromEnv(env);
      const outputFile = getNext();

      pushArtifact({ artifactType, crateName, outputFile });
      continue;
    }

    throw new ParseError(`Unexpected option: ${token}`);
  }

  if (!tokens.length) {
    throw new ParseError(EXPECTED_COMMAND);
  }

  const cmd = getNext();

  return {
    artifacts,
    cmd,
    args: tokens,
  };
}

module.exports = { ParseError, getArtifactName, parse };
