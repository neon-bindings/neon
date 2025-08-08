const addon = require("..");
const { expect } = require("chai");
const assert = require("chai").assert;

class Person {
  constructor(name) {
    this._person = addon.person_new(name);
  }

  greet() {
    return addon.person_greet(this._person);
  }
}

class RefPerson {
  constructor(name) {
    this._person = addon.ref_person_new(name);
  }

  greet() {
    return addon.ref_person_greet(this._person);
  }

  setName(name) {
    addon.ref_person_set_name(this._person, name);

    return this;
  }

  fail() {
    addon.ref_person_fail(this._person);
  }
}

describe("boxed", function () {
  it("can call methods", function () {
    const person = new Person("World");
    const greeting = person.greet();

    assert.strictEqual(greeting, "Hello, World!");
  });

  it("can call methods wrapped in a RefCell", function () {
    const person = new RefPerson("World");
    const greeting = person.greet();

    assert.strictEqual(greeting, "Hello, World!");
  });

  it("can mutate with ref cell", function () {
    const person = new RefPerson("World").setName("Universe");
    const greeting = person.greet();

    assert.strictEqual(greeting, "Hello, Universe!");
  });

  it("should dynamically check borrowing rules", function () {
    assert.throws(() => new RefPerson("World").fail(), /already borrowed/);
  });

  it("should type check externals", function () {
    // `any::type_name` does not guarantee exact format
    // failed downcast to neon::types::boxed::JsBox<napi::js::boxed::Person>
    assert.throws(
      () => addon.person_greet({}),
      /failed to downcast.*JsBox.*Person/
    );
  });

  it("should type check dynamic type", function () {
    const unit = addon.external_unit();

    assert.throws(() => addon.person_greet(unit), /failed to downcast/);
  });

  it("should be able to wrap a Rust value in an object", () => {
    const msg = "Hello, World!";
    const o = {};

    addon.wrapString(o, msg);
    assert.strictEqual(addon.unwrapString(o), msg);
  });

  it("should not be able to wrap an object twice", () => {
    const o = {};

    addon.wrapString(o, "Hello, World!");
    assert.throws(() => addon.wrapString(o, "nope"), /already wrapped/);
  });

  it("should not be able to unwrap an object that was not wrapped", () => {
    const o = {};

    assert.throws(() => addon.unwrapString(o), /not wrapped/);
  });
});
