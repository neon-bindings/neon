const addon = require('../native');

console.log(`this is null: ${addon.getNullSync()}`);
console.log(`this is undefined: ${addon.getUndefinedSync()}`);
console.log(`this is pi: ${addon.getNumberSync()}`);
console.log(`this is a string: ${addon.getStringSync()}`);
console.log(`this is a boolean: ${addon.getBooleanSync()}`);
console.log(`this is an array: ${addon.getArraySync()}`);
console.log(`this is an object: ${JSON.stringify(addon.getObjectSync())}`);
const returnFive = addon.getFunctionSync()
console.log(returnFive(), returnFive(), returnFive())