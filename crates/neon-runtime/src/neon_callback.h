#ifndef NEON_CALLBACK_H
#define NEON_CALLBACK_H

#include <uv.h>
#include "neon.h"
#include "v8.h"

namespace neon {

class Callback {
public:
  Callback(
       v8::Local<v8::Function> callback,
       v8::Isolate *isolate)
    : isolate_(isolate)
  {
    // Save the callback to be invoked when the Callback completes.
    callback_.Reset(isolate, callback);
    // Save the context (aka realm) to be used when invoking the callback.
    context_.Reset(isolate, isolate->GetCurrentContext());
  }

  void call(int argc, v8::Local<v8::Value> argv[]) {
    // Ensure that we have all the proper scopes installed on the C++ stack before
    // invoking the callback, and use the context (i.e. realm) we saved with the Callback.
    v8::Isolate::Scope isolate_scope(isolate_);
    v8::HandleScope handle_scope(isolate_);
    v8::Local<v8::Context> context = v8::Local<v8::Context>::New(isolate_, context_);
    v8::Context::Scope context_scope(context);
    v8::Local<v8::Function> callback = v8::Local<v8::Function>::New(isolate_, callback_);

    callback->Call(isolate_->GetCurrentContext(), v8::Null(isolate_), argc, argv);
  }

private:
  v8::Isolate *isolate_;
  v8::Persistent<v8::Function> callback_;
  v8::Persistent<v8::Context> context_;
};

}

#endif
