var fs = require('fs');
var beautify = require('js-beautify').js_beautify;
var path = require('path');

function Generator(project) {
  this.project = project;
}

Generator.prototype.addon = function addon() {
  var src = "/* THIS FILE WAS AUTOMATICALLY GENERATED. DO NOT EDIT. */\n\n"
          + "#include <nan.h>\n\n"
          + "extern \"C\" void node_main(v8::Local<v8::Object> module);\n\n"
          + "NODE_MODULE(" + this.project.libName + ", node_main)\n";
  fs.writeFileSync(path.resolve(this.project.root, "rust_addon.cc"), src);
};

Generator.prototype.gyp = function gyp() {
  var src = beautify(JSON.stringify({
    "targets": [{
      "target_name": this.project.libName,
      "sources": ["rust_addon.cc"],
      "include_dirs": ["<!(rust-bindings include)"],
      "libraries": ["../<!(rust-bindings build)"]
    }]
  }), { indent_size: 4 }) + "\n";
  fs.writeFileSync(path.resolve(this.project.root, "binding.gyp"), src);
};

module.exports = exports = function(project) {
  return new Generator(project);
};
