/**
 * Remark plugin: hide rustdoc-style hidden lines from rendered Rust code blocks.
 *
 * Mirrors rustdoc's convention exactly:
 *   - Lines whose first non-whitespace character is `#` followed by a space
 *     (or `#` at end-of-line) are removed from the rendered output.
 *   - Lines starting with `##` are unescaped to a literal `#`.
 *   - Other lines pass through unchanged.
 *
 * Applies only to fences whose language is exactly `rust`.
 */
export function remarkStripHiddenRustLines() {
  return (tree) => {
    visit(tree, "code", (node) => {
      if (node.lang !== "rust") return;
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

// Inline visit() to avoid adding a dependency for ~10 lines.
function visit(node, type, fn) {
  if (node.type === type) fn(node);
  if (Array.isArray(node.children)) {
    for (const child of node.children) visit(child, type, fn);
  }
}
