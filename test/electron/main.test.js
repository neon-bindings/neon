"use strict";

const assert = require("assert");
const path = require("path");

const { _electron: electron } = require("playwright");
const { test } = require("@playwright/test");

test("greeting", async () => {
  let app;
  try {
    app = await electron.launch({
      args: [path.join(__dirname, "main.js")],
    });
  } catch (error) {
    console.error("Failed to launch Electron app:", error);
    console.trace();
    throw error;
  }
  const page = await app.firstWindow();
  const header = page.locator("#greeting");
  const text = await header.textContent();

  assert.strictEqual(text, "Hello, World!");
});
