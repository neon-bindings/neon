var fs = require('fs');
var path = require('path');
var handlebars = require('handlebars');

var TEMPLATES_DIR = path.resolve(path.resolve(__dirname, ".."), "templates");

var TEMPLATE = handlebars.compile(fs.readFileSync(path.resolve(TEMPLATES_DIR, "binding.cc.hbs"), 'utf8'), { noEscape: true });

function Addon(project) {
  this.project = project;
  this.context = { project: { name: project.libName } };
}

Addon.prototype.generate = function generate(filename) {
  fs.writeFileSync(filename, TEMPLATE(this.context));
};

module.exports = exports = function(build) {
  return new Addon(build);
};
