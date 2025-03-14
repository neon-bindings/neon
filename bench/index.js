const { Suite, jsonReport } = require("bench-node");
const addon = require("./index.node");

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const n = sorted.length;
  return n % 2 === 0
    ? (sorted[n / 2 - 1] + sorted[n / 2]) / 2
    : sorted[Math.floor(n / 2)];
}

// A custom reporter for the bencher.dev benchmarking platform.
// Format: https://bencher.dev/docs/reference/bencher-metric-format/
//
// The reporter provides two measures for each benchmark:
// - "throughput": The number of operations per second.
// - "latency": The time taken to perform an operation, in ns.
//   * "value": The median value of all samples.
//   * "lower_value": The minimum value of all samples.
//   * "upper_value": The maximum value of all samples.
function reportBencherDev(results) {
  const bmf = Object.create(null);
  for (const result of results) {
    bmf[result.name] = {
      throughput: {
        value: result.opsSec,
      },
      latency: {
        value: median(result.histogram.sampleData),
        lower_value: result.histogram.min,
        upper_value: result.histogram.max,
      },
    };
  }
  console.log(JSON.stringify(bmf, null, 2));
}

const suite = new Suite({ reporter: reportBencherDev });

suite.add("hello-world", () => {
  addon.hello();
});

suite.add("manually-exported-noop", () => {
  addon.manualNoop();
});

suite.add("auto-exported-noop", () => {
  addon.exportNoop();
});

function triple(s, n, b) {
  return [s, n, b];
}

suite.add("JsFunction::call", () => {
  addon.callCallbackWithCall(triple);
});

suite.add("JsFunction::call_with", () => {
  addon.callCallbackWithCallWith(triple);
});

suite.add("JsFunction::bind", () => {
  addon.callCallbackWithBind(triple);
});

suite.run();
