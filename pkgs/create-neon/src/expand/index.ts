import { promises as fs } from "fs";
import handlebars from "handlebars";
import helpers from "handlebars-helpers";
import * as path from "path";
import { Context } from "./context.js";

const TEMPLATES_DIR = new URL(
  path.join("..", "data", "templates", "/"),
  import.meta.url
);

const COMPARISON_HELPERS = helpers("comparison");

handlebars.registerHelper("eq", COMPARISON_HELPERS.eq);

export async function expand(
  source: string,
  cx: Context
): Promise<string> {
  let template = await fs.readFile(new URL(source, TEMPLATES_DIR), "utf8");
  let compiled = handlebars.compile(template, { noEscape: true });
  return compiled(cx);
}

export async function expandTo(
  source: string,
  target: string,
  cx: Context
) {
  await fs.mkdir(path.dirname(target), { recursive: true });
  const expanded = await expand(source, cx);
  // The 'wx' flag creates the file but fails if it already exists.
  await fs.writeFile(target, expanded, { flag: "wx" });
}
