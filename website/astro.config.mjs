import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

export default defineConfig({
  site: "https://neon-rs.dev",
  integrations: [
    starlight({
      title: "Neon",
      description: "Write Node addons in Rust.",
      logo: {
        src: "./public/logo.png",
        replacesTitle: false,
      },
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/neon-bindings/neon" },
        { icon: "slack", label: "Slack", href: "https://rust-bindings.slack.com" },
      ],
      sidebar: [
        { label: "Getting started", autogenerate: { directory: "getting-started" } },
        { label: "Tutorials", autogenerate: { directory: "tutorials" } },
        { label: "How-to guides", autogenerate: { directory: "how-to" } },
        {
          label: "Reference",
          items: [
            { label: "API reference", link: "/api/neon/", attrs: { target: "_blank" } },
            { label: "Supported platforms", link: "/reference/supported-platforms/" },
            { label: "CLI reference", link: "/reference/cli/" },
          ],
        },
        { label: "Explanation", autogenerate: { directory: "explanation" } },
      ],
    }),
  ],
});
