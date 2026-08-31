import { defineCollection, z } from "astro:content";
import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from "@astrojs/starlight/schema";

export const collections = {
  docs: defineCollection({
    loader: docsLoader(),
    schema: (context) =>
      docsSchema({
        extend: z.object({
          // Publication status of an unfinished page:
          //
          //   status: draft  -- written, but pending review
          //   status: todo   -- placeholder awaiting content
          //
          // Either value excludes the page from production builds (via
          // Starlight's built-in `draft` mechanism). In `astro dev`, the
          // page renders with Starlight's draft notice and a sidebar badge
          // (see ./route-data.ts).
          status: z.enum(["draft", "todo"]).optional(),
        }),
      })(context).transform((data) =>
        data.status ? { ...data, draft: true } : data
      ),
  }),
};
