# Benchmark

These benchmarks are intended for evaluating the performance impact of Neon changes.

## Run

```shell
# All benchmarks
node bench

# Single benchmark
node bench serialize

# Execute only one part of a bench group
node bench --json
node bench --json serialize

# Generate a CPU profile
# Note: While it is not necessary to limit to a single bench function,
# the generated report will be easier to read.
node bench.js --neon --report=report.proto deserialize
```

## View Report

CPU reports are generated with `pprof` and can use the standard tooling for evaluation.

```shell
go tool pprof -http=:8080 report.proto
```