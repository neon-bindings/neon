const assert = require("assert");

assert.throws(
  () => require("./index.node"),
  (err) => {
    assert.strictEqual(
      err.message,
      [
        "Neon module exports more than one item under the same name, so only one of each could be exported:",
        "  - `Counter` is exported by static `duplicate_exports::COUNTER` and class `duplicate_exports::Counter`",
        "  - `greet` is exported by function `duplicate_exports::greet_a` and function `duplicate_exports::greet_b`",
        "  - `parse` is exported by function `duplicate_exports::v1::parse` and function `duplicate_exports::v2::parse`",
        "  - `version` is exported by const `duplicate_exports::VERSION` and function `duplicate_exports::version`",
        'Rename all but one item in each with `#[neon::export(name = "...")]`.',
      ].join("\n")
    );
    return true;
  }
);

console.log("ok: duplicate export names fail to load");
