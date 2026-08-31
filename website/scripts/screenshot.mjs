// Capture light + dark home-page screenshots from the preview server.
// Usage: node website/scripts/screenshot.mjs [outputDir]
import { chromium } from "playwright";
import path from "node:path";
import { mkdirSync } from "node:fs";

const outputDir = path.resolve(process.argv[2] ?? "/tmp/neon-shots");
mkdirSync(outputDir, { recursive: true });

const url = process.env.URL ?? "http://127.0.0.1:4321/";

const browser = await chromium.launch({ channel: "chrome" });
try {
  for (const theme of ["dark", "light"]) {
    const ctx = await browser.newContext({
      viewport: { width: 1440, height: 900 },
      colorScheme: theme,
      deviceScaleFactor: 2,
    });
    const page = await ctx.newPage();
    await page.goto(url, { waitUntil: "networkidle" });
    await page.evaluate((t) => {
      document.documentElement.dataset.theme = t;
      localStorage.setItem("starlight-theme", t);
    }, theme);
    await page.waitForTimeout(300);

    const fullPath = path.join(outputDir, `home-${theme}-full.png`);
    const topPath = path.join(outputDir, `home-${theme}-top.png`);
    await page.screenshot({ path: fullPath, fullPage: true });
    await page.screenshot({ path: topPath, fullPage: false });
    console.log(`wrote ${fullPath}`);
    console.log(`wrote ${topPath}`);
    await ctx.close();
  }
} finally {
  await browser.close();
}
