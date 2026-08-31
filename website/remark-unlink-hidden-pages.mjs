// Remark plugin supporting the `status: draft | todo` front matter flag
// (see src/content.config.ts): in production builds, links pointing at
// flagged pages are replaced with their plain text, since the target pages
// are excluded from the build and would 404. Dev builds keep the links so
// draft content stays navigable while writing. When a page's flag is
// removed, links to it come back automatically on the next build.
import { readdirSync, readFileSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

const DOCS_ROOT = fileURLToPath(new URL("./src/content/docs", import.meta.url));

/** Route paths (e.g. "/how-to/async-fn/") of pages flagged draft or todo. */
function hiddenRoutes() {
  const hidden = new Set();
  for (const entry of readdirSync(DOCS_ROOT, {
    recursive: true,
    withFileTypes: true,
  })) {
    if (!entry.isFile() || !/\.mdx?$/.test(entry.name)) continue;
    const path = join(entry.parentPath, entry.name);
    const frontmatter = readFileSync(path, "utf8").match(
      /^---\r?\n(.*?)\r?\n---(\r?\n|$)/s
    );
    if (
      !frontmatter ||
      !/^status:\s*["']?(draft|todo)["']?\s*$/m.test(frontmatter[1])
    )
      continue;
    const parts = relative(DOCS_ROOT, path)
      .replace(/\.mdx?$/, "")
      .split(sep);
    // index.md routes as its parent directory.
    if (parts[parts.length - 1] === "index") parts.pop();
    hidden.add(parts.length === 0 ? "/" : `/${parts.join("/")}/`);
  }
  return hidden;
}

export function remarkUnlinkHiddenPages() {
  if (process.env.NODE_ENV !== "production") return () => {};
  const hidden = hiddenRoutes();

  const isHidden = (url) =>
    url.startsWith("/") &&
    hidden.has(url.replace(/[#?].*$/, "").replace(/\/?$/, "/"));

  // Replace hidden-page links with their children, bottom-up. Inline the
  // walk to avoid adding a dependency for ~10 lines.
  function unlink(node) {
    if (!node.children) return;
    for (const child of node.children) unlink(child);
    node.children = node.children.flatMap((child) =>
      child.type === "link" && isHidden(child.url) ? child.children : [child]
    );
  }

  return unlink;
}
