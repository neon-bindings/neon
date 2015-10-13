#include <nan.h>
#include <stdint.h>
#include <v8.h>

extern "C" {

  void Nan_FunctionCallbackInfo_SetReturnValue(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value);
  void *Nan_FunctionCallbackInfo_GetIsolate(Nan::FunctionCallbackInfo<v8::Value> *info);
  bool Nan_FunctionCallbackInfo_IsConstructCall(Nan::FunctionCallbackInfo<v8::Value> *info);
  void Nan_FunctionCallbackInfo_This(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Object> *out);
  int32_t Nan_FunctionCallbackInfo_Length(Nan::FunctionCallbackInfo<v8::Value> *info);
  void Nan_FunctionCallbackInfo_Get(Nan::FunctionCallbackInfo<v8::Value> *info, int32_t i, v8::Local<v8::Value> *out);

  void Nan_EscapableHandleScope_Escape(v8::Local<v8::Value> *out, Nan::EscapableHandleScope *scope, v8::Local<v8::Value> value);

  void Nan_Export(v8::Local<v8::Object> *target, const char *name, Nan::FunctionCallback f);

  void Nan_NewObject(v8::Local<v8::Object> *out);
  void Nan_NewInteger(v8::Local<v8::Integer> *out, v8::Isolate *isolate, int32_t x);
  void Nan_NewNumber(v8::Local<v8::Number> *out, v8::Isolate *isolate, double value);
  void Nan_NewArray(v8::Local<v8::Array> *out, v8::Isolate *isolate, uint32_t length);
  void Nan_NewUndefined(v8::Local<v8::Primitive> *out);
  void Nan_NewNull(v8::Local<v8::Primitive> *out);
  void Nan_NewBoolean(v8::Local<v8::Boolean> *out, bool b);

  bool Nan_ArraySet(v8::Local<v8::Array> *array, uint32_t index, v8::Local<v8::Value> value);

  // typedef void(*Nan_ScopedCallback)(void*, Nan::HandleScope*, void*);
  // typedef void(*Nan_EscapeScopedCallback)(void*, Nan::EscapableHandleScope*, void*);

  // void Nan_Scoped(void *out, void *closure, Nan_ScopedCallback callback);
  // void Nan_EscapeScoped(void *out, void *closure, Nan_EscapeScopedCallback callback);

  typedef void(*Nan_ChainedScopeCallback)(void *, void *, void *, void *);
  typedef void(*Nan_NestedScopeCallback)(void *, void *, void *);
  typedef void(*Nan_RootScopeCallback)(void *, void *, void *);

  void Nan_Root(void *out, void *closure, Nan_RootScopeCallback callback, void *isolate);
  void Nan_Nested(void *out, void *closure, Nan_NestedScopeCallback callback, void *realm);
  void Nan_Chained(void *out, void *closure, Nan_ChainedScopeCallback callback, void *parent_scope);

  //typedef void(*Nan_ChainedCallback)(void *, void *, void *);
  //void Nan_Chained(void *out, void *closure, Nan_ChainedCallback callback, void *parent_scope);


  // void Nan_UpcastArray(v8::Local<v8::Value> *out, v8::Local<v8::Array> array);
  // void Nan_UpcastPrimitive(v8::Local<v8::Value> *out, v8::Local<v8::Primitive> prim);

  /*
  void Nan_NewString(Nan::MaybeLocal<v8::String> *out, const char *value);
  void Nan_NewStringN(Nan::MaybeLocal<v8::String> *out, const char *value, int32_t length);
  bool Nan_MaybeLocalString_IsEmpty(Nan::MaybeLocal<v8::String> *maybe);
  bool Nan_MaybeLocalString_ToLocal(Nan::MaybeLocal<v8::String> *maybe, Nan_LocalString *out);
  */
}
