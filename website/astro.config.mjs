import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import mermaid from "astro-mermaid";
import { remarkStripHiddenRustLines } from "./strip-hidden-rust-lines.mjs";
import { rustdocDevPlugin } from "./scripts/rustdoc-dev-plugin.mjs";

export default defineConfig({
  site: "https://neon-rs.dev",
  markdown: {
    remarkPlugins: [remarkStripHiddenRustLines],
  },
  vite: {
    plugins: [rustdocDevPlugin()],
  },
  integrations: [
    mermaid({ theme: "neutral", autoTheme: true }),
    starlight({
      title: "Neon",
      description: "Write Node addons in Rust.",
      expressiveCode: {
        shiki: {
          // Map rustdoc fence attributes (compile_fail, ignore, no_run,
          // should_panic) to the `rust` highlighter. The doctest harness
          // still sees the full attribute string; this only governs how
          // expressive-code looks up a syntax for highlighting.
          langAlias: {
            "rust,compile_fail": "rust",
            "rust,ignore": "rust",
            "rust,no_run": "rust",
            "rust,should_panic": "rust",
          },
        },
      },
      logo: {
        src: "./public/logo-mark.png",
        replacesTitle: true,
      },
      favicon: "/favicon.svg",
      // Hides `status: draft | todo` pages from the production sidebar and
      // badges them in dev. See src/route-data.ts.
      routeMiddleware: "./src/route-data.ts",
      head: [
        {
          tag: "link",
          attrs: {
            rel: "apple-touch-icon",
            sizes: "180x180",
            href: "/logo.png",
          },
        },
      ],
      customCss: ["./src/styles/neon.css"],
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/neon-bindings/neon" },
        { icon: "slack", label: "Slack", href: "https://rust-bindings.slack.com" },
      ],
      sidebar: [
        {
          label: "Getting started",
          items: [{ autogenerate: { directory: "getting-started" } }],
        },
        {
          label: "Tutorials",
          items: [
            { label: "Your first Neon addon", link: "/tutorials/first-addon/" },
            { label: "Move work off the main thread", link: "/tutorials/move-work-off-the-main-thread/" },
            { label: "Build a database addon", link: "/tutorials/build-a-database-addon/" },
            { label: "Publish your addon to npm", link: "/tutorials/publish-your-addon-to-npm/" },
          ],
        },
        {
          label: "How-to guides",
          collapsed: true,
          items: [{ autogenerate: { directory: "how-to" } }],
        },
        {
          label: "Reference",
          items: [
            { label: "API reference", link: "/api/neon/", attrs: { target: "_blank" } },
            { label: "Supported platforms", link: "/reference/supported-platforms/" },
            { label: "CLI reference", link: "/reference/cli/" },
          ],
        },
        {
          label: "Explanation",
          items: [{ autogenerate: { directory: "explanation" } }],
        },
        { label: "Changelog", link: "/changelog/" },
        { label: "Contributing", link: "/contributing/" },
      ],
    }),
  ],
});
