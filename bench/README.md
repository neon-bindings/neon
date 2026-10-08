# bench: Neon performance regression suite

## Building bench

To run the build, run:

```sh
$ npm run build
```

## Running the benchmarks

To run the benchmarks, run:

```sh
$ npm run benchmark
```

To compare against a build with Node-API 10 (e.g. interned property keys), run:

```sh
$ npm run build -- --features=napi-10
$ npm run benchmark
```
