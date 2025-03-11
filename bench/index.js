const { Suite, jsonReport } = require('bench-node');
const addon = require('./index.node');

// A custom reporter for the bencher.dev benchmarking platform.
// Format: https://bencher.dev/docs/reference/bencher-metric-format/
function reportBencherDev(results) {
  const bmf = Object.create(null);
  for (const result of results) {
    // If https://github.com/RafaelGSS/bench-node/issues/66 is fixed, then we can use
    // result.histogram to report a "latency" measure with the median as the "value",
    // min as "lower_value", and max as "upper_value".
    bmf[result.name] = {
      "throughput": {
        "value": result.opsSec,
      },
    };
  }
  console.log(JSON.stringify(bmf, null, 2));
}

const suite = new Suite({ reporter: reportBencherDev });

suite.add('hello-world', () => {
  addon.hello();
});

suite.add('manually-exported-noop', () => {
  addon.manualNoop();
});

suite.add('auto-exported-noop', () => {
  addon.exportNoop();
});

function triple(s, n, b) {
  return [s, n, b];
}

suite.add('JsFunction::call', () => {
  addon.callCallbackWithCall(triple);
});

suite.add('JsFunction::call_with', () => {
  addon.callCallbackWithCallWith(triple);
});

suite.add('JsFunction::bind', () => {
  addon.callCallbackWithBind(triple);
});

suite.run();
