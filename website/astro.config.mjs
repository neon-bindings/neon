import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
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
    starlight({
      title: "Neon",
      description: "Write Node addons in Rust.",
      logo: {
        src: "./public/logo-mark.png",
        replacesTitle: true,
      },
      favicon: "/favicon.svg",
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
        { label: "Getting started", autogenerate: { directory: "getting-started" } },
        {
          label: "Tutorials",
          items: [
            { label: "Your first Neon addon", link: "/tutorials/first-module/" },
            { label: "Move work off the main thread", link: "/tutorials/move-work-off-the-main-thread/" },
            { label: "Async functions with tokio", link: "/tutorials/async-tokio/" },
          ],
        },
        {
          label: "How-to guides",
          collapsed: true,
          autogenerate: { directory: "how-to" },
        },
        {
          label: "Reference",
          items: [
            { label: "API reference", link: "/api/neon/", attrs: { target: "_blank" } },
            { label: "Supported platforms", link: "/reference/supported-platforms/" },
            { label: "CLI reference", link: "/reference/cli/" },
          ],
        },
        { label: "Explanation", autogenerate: { directory: "explanation" } },
        { label: "Changelog", link: "/changelog/" },
        { label: "Contributing", link: "/contributing/" },
      ],
    }),
  ],
});
