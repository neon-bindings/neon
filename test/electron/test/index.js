'use strict';

const assert = require('assert');
const path = require('path');

const electron = require('electron');
const { Application } = require('spectron');

const app = new Application({
    path: electron,
    args: [path.join(__dirname, '..')]
})

async function tests() {
    const isVisible = await app.browserWindow.isVisible();

    assert.equal(isVisible, true);

    const title = await app.client.getTitle();

    assert.strictEqual(title, 'Neon Electron Test');

    const header = await app.client.getText('#header');

    assert.strictEqual(header, 'Hello, World!');
}

async function runTests() {
    await app.start();

    try {
        await tests();
        console.log('Electron tests passed!');
    } catch (err) {
        console.error(err);
        console.log('Electron tests failed. :\'(');
        process.exitCode = -1;
    } finally {
        app.stop();
    }
}

runTests();
