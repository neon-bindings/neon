const addon = require('..');
const { assert } = require('chai');

describe('JsSymbol', function() {
    it('should return a JsSymbol with a description built in Rust', function () {
        const sym = addon.return_js_symbol_with_description();
        assert.equal(typeof sym, 'symbol');
        assert.equal(sym.description, "neon:description");
    });
    it('should return a JsSymbol without a description built in Rust', function () {
        const sym = addon.return_js_symbol();
        assert.equal(typeof sym, 'symbol');
        assert.equal(sym.description, undefined);
    });
    it('should read the description property in Rust', function () {
        const sym = Symbol('neon:description');
        const description = addon.read_js_symbol_description(sym);
        assert.equal(description, 'neon:description');
    });
    it('should read an undefined description property in Rust', function () {
        const sym = Symbol();
        const description = addon.read_js_symbol_description(sym);
        assert.equal(description, undefined);
    });
    it('accepts and returns symbols', function () {
        const symDesc = Symbol('neon:description');
        const symNoDesc = Symbol();
        assert.equal(addon.accept_and_return_js_symbol(symDesc), symDesc);
        assert.equal(addon.accept_and_return_js_symbol(symNoDesc), symNoDesc);
    });
});
