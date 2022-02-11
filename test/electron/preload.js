const native = require("./index.node");

window.addEventListener("DOMContentLoaded", () => {
  document.getElementById("greeting").innerText = native.hello();
});
