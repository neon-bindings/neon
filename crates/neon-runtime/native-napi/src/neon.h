#ifndef NEON_H
#define NEON_H

#include <node_api.h>

// corresponding Rust struct `CCallback` defined in fun.rs
typedef struct {
  void* static_callback;
  void* dynamic_callback;
} callback_t;

// Defined by the Rust library's use of `register_module!`.
extern "C" void neon_init_module();

napi_value init(napi_env env, napi_value exports) {
  neon_init_module();
  return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, init)

#endif
