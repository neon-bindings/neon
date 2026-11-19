"use strict";

const path = require("path");

const { _electron: electron } = require("playwright");
const { test, expect } = require("@playwright/test");

test("greeting", async () => {
  const app = await electron.launch({
    args: [path.join(__dirname, "main.js")],
  });
  const page = await app.firstWindow();

  // `preload.js` fills in the greeting on `DOMContentLoaded`, which may not
  // have fired yet; `toHaveText` retries until it does.
  await expect(page.locator("#greeting")).toHaveText("Hello, World!");
});
