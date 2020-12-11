'use strict';

const native = require('./index.node');

native.getNum(() => new Promise(resolve => setTimeout(resolve, 1000, 5)));

