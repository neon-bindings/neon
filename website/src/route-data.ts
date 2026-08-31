// Starlight route middleware supporting the `status: draft | todo` front
// matter flag (see ./content.config.ts).
//
// Flagged pages set `draft: true`, so Starlight already excludes their
// routes from production builds. The sidebar needs matching treatment,
// though: explicitly-linked entries (e.g. the Tutorials group) are not
// filtered by Starlight and would 404 in production, while in dev it is
// easy to lose track of which pages are unfinished. So:
//
//   - production: prune sidebar entries that point at excluded pages
//     (dropping any groups left empty) and recompute the prev/next links,
//     which Starlight derived from the unpruned sidebar.
//   - dev: badge flagged entries with "Draft" or "TODO".
import {
  defineRouteMiddleware,
  type StarlightRouteData,
} from "@astrojs/starlight/route-data";
import { getCollection } from "astro:content";

type SidebarEntry = StarlightRouteData["sidebar"][number];
type SidebarLink = Extract<SidebarEntry, { type: "link" }>;

// Page slug -> `status` front matter, for every flagged page.
const statuses = new Map(
  (await getCollection("docs", ({ data }) => data.status !== undefined)).map(
    (entry) => [entry.id === "index" ? "" : entry.id, entry.data.status!]
  )
);

const BADGES = {
  draft: { text: "Draft", variant: "note" },
  todo: { text: "TODO", variant: "caution" },
} as const;

function statusOf(link: SidebarLink) {
  return statuses.get(link.href.replace(/^\//, "").replace(/\/$/, ""));
}

function prune(entries: SidebarEntry[]): SidebarEntry[] {
  return entries.flatMap((entry): SidebarEntry[] => {
    if (entry.type === "link") return statusOf(entry) ? [] : [entry];
    const pruned = prune(entry.entries);
    return pruned.length === 0 ? [] : [{ ...entry, entries: pruned }];
  });
}

function flatten(entries: SidebarEntry[]): SidebarLink[] {
  return entries.flatMap((entry) =>
    entry.type === "link" ? entry : flatten(entry.entries)
  );
}

function badge(entries: SidebarEntry[]) {
  for (const entry of entries) {
    if (entry.type === "group") {
      badge(entry.entries);
    } else {
      const status = statusOf(entry);
      if (status) entry.badge ??= { ...BADGES[status] };
    }
  }
}

export const onRequest = defineRouteMiddleware(({ locals }) => {
  const route = locals.starlightRoute;
  // The same condition Starlight uses to exclude draft routes.
  if (import.meta.env.MODE === "production") {
    route.sidebar = prune(route.sidebar);
    // Note: recomputed purely from sidebar order; front matter prev/next
    // overrides (unused on this site) are not consulted.
    const links = flatten(route.sidebar);
    const current = links.findIndex((link) => link.isCurrent);
    route.pagination = {
      prev: current > 0 ? links[current - 1] : undefined,
      next: current === -1 ? undefined : links[current + 1],
    };
  } else {
    badge(route.sidebar);
  }
});
