#include <new>
#include <nan.h>
#include <stdint.h>
#include <stdio.h>
#include "nanny.h"

extern "C" void Nan_FunctionCallbackInfo_SetReturnValue(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value) {
  info->GetReturnValue().Set(value);
}

extern "C" void *Nan_FunctionCallbackInfo_GetIsolate(Nan::FunctionCallbackInfo<v8::Value> *info) {
  return (void *)info->GetIsolate();
}

extern "C" bool Nan_FunctionCallbackInfo_IsConstructCall(Nan::FunctionCallbackInfo<v8::Value> *info) {
  return info->IsConstructCall();
}

extern "C" void Nan_FunctionCallbackInfo_This(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Object> *out) {
  *out = info->This();
}

extern "C" int32_t Nan_FunctionCallbackInfo_Length(Nan::FunctionCallbackInfo<v8::Value> *info) {
  return info->Length();
}

extern "C" void Nan_FunctionCallbackInfo_Get(Nan::FunctionCallbackInfo<v8::Value> *info, int32_t i, v8::Local<v8::Value> *out) {
  *out = (*info)[i];
}

extern "C" void Nan_EscapableHandleScope_Escape(v8::Local<v8::Value> *out, Nan::EscapableHandleScope *scope, v8::Local<v8::Value> value) {
  *out = scope->Escape(value);
}

extern "C" void Nan_Export(v8::Local<v8::Object> *target, const char *name, Nan::FunctionCallback f) {
  Nan::Export(*target, name, f);
}

// extern "C" void Nan_UpcastArray(v8::Local<v8::Value> *out, v8::Local<v8::Array> array) {
//   *out = v8::Local<v8::Value>::Cast(array);
// }

// extern "C" void Nan_UpcastPrimitive(v8::Local<v8::Value> *out, v8::Local<v8::Primitive> prim) {
//   *out = v8::Local<v8::Value>::Cast(prim);
// }

extern "C" void Nan_NewObject(v8::Local<v8::Object> *out) {
  *out = Nan::New<v8::Object>();
}

extern "C" void Nan_NewUndefined(v8::Local<v8::Primitive> *out) {
  *out = Nan::Undefined();
}

extern "C" void Nan_NewNull(v8::Local<v8::Primitive> *out) {
  *out = Nan::Null();
}

extern "C" void Nan_NewBoolean(v8::Local<v8::Boolean> *out, bool b) {
  *out = b ? Nan::True() : Nan::False();
}

extern "C" void Nan_NewInteger(v8::Local<v8::Integer> *out, v8::Isolate *isolate, int32_t x) {
  *out = v8::Integer::New(isolate, x);
}

extern "C" void Nan_NewNumber(v8::Local<v8::Number> *out, v8::Isolate *isolate, double value) {
  *out = v8::Number::New(isolate, value);
}

extern "C" void Nan_NewArray(v8::Local<v8::Array> *out, v8::Isolate *isolate, uint32_t length) {
  *out = v8::Array::New(isolate, length);
}

extern "C" bool Nan_ArraySet(v8::Local<v8::Array> *array, uint32_t index, v8::Local<v8::Value> value) {
  return (*array)->Set(index, value);
}

extern "C" void Nan_Chained(void *out, void *closure, Nan_ChainedScopeCallback callback, void *parent_scope) {
  Nan::EscapableHandleScope v8_scope;
  callback(out, parent_scope, &v8_scope, closure);
}

extern "C" void Nan_Nested(void *out, void *closure, Nan_NestedScopeCallback callback, void *realm) {
  Nan::HandleScope v8_scope;
  callback(out, realm, closure);
}

extern "C" void Nan_Root(void *out, void *closure, Nan_RootScopeCallback callback, void *isolate) {
  Nan::HandleScope v8_scope;
  callback(out, isolate, closure);
}

/*
extern "C" void Nan_NewString(Nan::MaybeLocal<v8::String> *out, const char *value) {
  *out = Nan::New<v8::String>(value);
}

extern "C" void Nan_NewStringN(Nan::MaybeLocal<v8::String> *out, const char *value, int32_t length) {
  *out = Nan::New<v8::String>(value, length);
}

extern "C" bool Nan_MaybeLocalString_IsEmpty(Nan::MaybeLocal<v8::String> *maybe) {
  return maybe->IsEmpty();
}

extern "C" bool Nan_MaybeLocalString_ToLocal(Nan::MaybeLocal<v8::String> *maybe, Nan::Local<v8::String> *out) {
  return maybe->ToLocal(out);
}
*/
