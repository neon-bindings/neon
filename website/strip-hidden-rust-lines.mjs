/**
 * Remark plugin: hide rustdoc-style hidden lines from rendered Rust code blocks.
 *
 * Mirrors rustdoc's convention exactly:
 *   - Lines whose first non-whitespace character is `#` followed by a space
 *     (or `#` at end-of-line) are removed from the rendered output.
 *   - Lines starting with `##` are unescaped to a literal `#`.
 *   - Other lines pass through unchanged.
 *
 * Applies to fences whose language is `rust` or `rust,<attr>` (e.g.
 * `rust,compile_fail`, `rust,ignore`, `rust,no_run`, `rust,should_panic`).
 * The trailing attributes are rustdoc directives and don't change the
 * fact that the body is Rust.
 */
export function remarkStripHiddenRustLines() {
  return (tree) => {
    visit(tree, "code", (node) => {
      if (!isRustFence(node.lang)) return;
      const lines = node.value.split("\n");
      const out = [];
      for (const line of lines) {
        const trimmed = line.trimStart();
        if (trimmed === "#" || trimmed.startsWith("# ")) {
          continue;
        }
        if (trimmed.startsWith("##")) {
          // Unescape ## → # at the same leading-whitespace position.
          const leading = line.slice(0, line.length - trimmed.length);
          out.push(leading + trimmed.slice(1));
          continue;
        }
        out.push(line);
      }
      node.value = out.join("\n");
    });
  };
}

function isRustFence(lang) {
  if (typeof lang !== "string") return false;
  if (lang === "rust") return true;
  return lang.startsWith("rust,");
}

// Inline visit() to avoid adding a dependency for ~10 lines.
function visit(node, type, fn) {
  if (node.type === type) fn(node);
  if (Array.isArray(node.children)) {
    for (const child of node.children) visit(child, type, fn);
  }
}
