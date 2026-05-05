// Vite plugin that serves /api/* from ../target/doc/* during `astro dev`.
//
// In production builds we copy `target/doc` into `public/api` via the
// prebuild step (see scripts/copy-rustdoc.mjs). That snapshot doesn't
// exist in dev (the user typically runs with SKIP_RUSTDOC=1), so links
// from docs into the API reference 404 in dev.
//
// This plugin closes that gap: while the dev server is running, any
// request under /api/ is satisfied by reading the corresponding file
// from `target/doc/` on disk. If `target/doc/` is empty (rustdoc
// hasn't been generated), the request falls through and Starlight's
// usual 404 logic handles it.
//
// The plugin is a no-op during build — the production HTML is served
// from the bundled assets in public/api/ as before.

import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
const websiteDir = resolve(here, "..");
const repoRoot = resolve(websiteDir, "..");
const rustdocRoot = resolve(repoRoot, "target", "doc");

const MIME_TYPES = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".woff": "font/woff",
  ".woff2": "font/woff2",
  ".ttf": "font/ttf",
  ".ico": "image/x-icon",
  ".txt": "text/plain; charset=utf-8",
};

export function rustdocDevPlugin() {
  return {
    name: "neon:rustdoc-dev",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        if (!req.url) return next();

        // Strip query string / fragment.
        const [pathname] = req.url.split("?", 2);

        // We only handle /api/* requests.
        if (!pathname.startsWith("/api/")) return next();

        // Resolve to a path under target/doc. Reject anything that
        // tries to escape the rustdoc tree (defense in depth — the
        // dev server only runs locally, but no reason to be sloppy).
        const rel = decodeURIComponent(pathname.slice("/api/".length));
        const filePath = resolve(rustdocRoot, rel);
        if (!filePath.startsWith(rustdocRoot)) {
          return next();
        }

        try {
          const stats = await stat(filePath);
          let target = filePath;
          if (stats.isDirectory()) {
            // Mirror rustdoc's convention: directories serve index.html.
            target = resolve(filePath, "index.html");
            await stat(target);
          }
          const mime = MIME_TYPES[extname(target).toLowerCase()] ??
            "application/octet-stream";
          res.statusCode = 200;
          res.setHeader("content-type", mime);
          createReadStream(target).pipe(res);
        } catch {
          // Missing rustdoc file: fall through to the rest of the
          // middleware chain so Starlight can render its 404 page.
          next();
        }
      });
    },
  };
}
