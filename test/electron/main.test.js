"use strict";

const assert = require("assert");
const path = require("path");

const { _electron: electron } = require("playwright");
const { test } = require("@playwright/test");

test("greeting", async () => {
  const app = await electron.launch({
    args: [path.join(__dirname, "main.js")],
  });
  const page = await app.firstWindow();
  const header = page.locator("#greeting");
  const text = await header.textContent();

  assert.strictEqual(text, "Hello, World!");
});
