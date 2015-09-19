#include <new>
#include <nan.h>
#include <stdint.h>
#include <stdio.h>
#include "nanny.h"

extern "C" void Nan_FunctionCallbackInfo_SetReturnValue(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value) {
  info->GetReturnValue().Set(value);
}

extern "C" void Nan_Export(v8::Local<v8::Object> *target, const char *name, Nan::FunctionCallback f) {
  Nan::Export(*target, name, f);
}

extern "C" void Nan_UpcastArray(v8::Local<v8::Value> *out, v8::Local<v8::Array> *array) {
  *out = v8::Local<v8::Value>::Cast(*array);
}

extern "C" void Nan_NewObject(v8::Local<v8::Object> *out) {
  *out = Nan::New<v8::Object>();
}

extern "C" void Nan_NewInteger(v8::Local<v8::Integer> *out, int32_t x) {
  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  *out = v8::Integer::New(isolate, x);
}

extern "C" void Nan_NewNumber(v8::Local<v8::Number> *out, double value) {
  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  *out = v8::Number::New(isolate, value);
}

extern "C" void Nan_NewArray(v8::Local<v8::Array> *out, uint32_t length) {
  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  *out = v8::Array::New(isolate, length);
}

extern "C" bool Nan_ArraySet(v8::Local<v8::Array> *array, uint32_t index, v8::Local<v8::Value> value) {
  return (*array)->Set(index, value);
}

extern "C" void Nan_Scoped(void *out, void *closure, Nan_ScopedCallback callback) {
  Nan::HandleScope scope;
  callback(out, &scope, closure);
}

extern "C" void Nan_EscapeScoped(void *out, void *closure, Nan_EscapeScopedCallback callback) {
  Nan::EscapableHandleScope scope;
  callback(out, &scope, closure);
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
