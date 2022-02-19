"use strict";

const assert = require("assert");
const path = require("path");

const electron = require("electron");
const { Application } = require("spectron");
const { window } = require("globalthis/implementation");

const app = new Application({
  path: electron,
  args: [path.join(__dirname, "..")],
});

async function tests() {
  const header = await app.client.$("#greeting");
  const text = await header.getText();

  assert.strictEqual(text, "Hello, World!");
}

async function runTests() {
  await app.start();

  try {
    await tests();
    console.log("Electron tests passed!");
  } catch (err) {
    console.error(err);
    console.log("Electron tests failed. :'(");
    process.exitCode = -1;
  } finally {
    // app.stop does not work with a secure window
    // https://github.com/electron-userland/spectron/issues/347
    await app.client.executeAsync(() => window.close());
    await app.chromeDriver.stop();
  }
}

runTests();
