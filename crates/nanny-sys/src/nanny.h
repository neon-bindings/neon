#include <nan.h>
#include <stdint.h>
#include <v8.h>

typedef struct {
    void* data;
    size_t len;
} buf_t;

extern "C" {

  void Nan_FunctionCallbackInfo_SetReturnValue(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value);
  void *Nan_FunctionCallbackInfo_GetIsolate(Nan::FunctionCallbackInfo<v8::Value> *info);
  bool Nan_FunctionCallbackInfo_IsConstructCall(Nan::FunctionCallbackInfo<v8::Value> *info);
  void Nan_FunctionCallbackInfo_This(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Object> *out);
  void Nan_FunctionCallbackInfo_Callee(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Function> *out);
  void Nan_FunctionCallbackInfo_Data(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> *out);
  int32_t Nan_FunctionCallbackInfo_Length(Nan::FunctionCallbackInfo<v8::Value> *info);
  void Nan_FunctionCallbackInfo_Get(Nan::FunctionCallbackInfo<v8::Value> *info, int32_t i, v8::Local<v8::Value> *out);

  void Nan_EscapableHandleScope_Escape(v8::Local<v8::Value> *out, Nan::EscapableHandleScope *scope, v8::Local<v8::Value> value);

  void Nan_NewObject(v8::Local<v8::Object> *out);
  bool Nan_GetOwnPropertyNames(v8::Local<v8::Array> *out, v8::Local<v8::Object> *obj);
  void *Nan_Object_GetIsolate(v8::Local<v8::Object> *obj);

  void Nan_NewInteger(v8::Local<v8::Integer> *out, v8::Isolate *isolate, int32_t x);
  bool Nan_NewString(v8::Local<v8::String> *out, v8::Isolate *isolate, const uint8_t *data, int32_t len);
  void Nan_NewNumber(v8::Local<v8::Number> *out, v8::Isolate *isolate, double value);
  void Nan_NewArray(v8::Local<v8::Array> *out, v8::Isolate *isolate, uint32_t length);
  void Nan_NewUndefined(v8::Local<v8::Primitive> *out);
  void Nan_NewNull(v8::Local<v8::Primitive> *out);
  void Nan_NewBoolean(v8::Local<v8::Boolean> *out, bool b);

  bool Node_ArraySet(v8::Local<v8::Array> *array, uint32_t index, v8::Local<v8::Value> value);
  uint32_t Node_ArrayLength(v8::Local<v8::Array> *array);
  bool Nan_Get_Index(v8::Local<v8::Value> *out, v8::Local<v8::Object> *object, uint32_t index);
  bool Nan_Get(v8::Local<v8::Value> *out, v8::Local<v8::Object> *object, v8::Local<v8::Value> *key);
  bool Nan_Set(bool *out, v8::Local<v8::Object> *obj, v8::Local<v8::Value> *key, v8::Local<v8::Value> *val);

  int32_t Nan_String_Utf8Length(v8::Local<v8::String> *str);

  bool Nan_Value_ToString(v8::Local<v8::String> *out, v8::Local<v8::Value> *value);
  bool Nan_Value_ToObject(v8::Local<v8::Object> *out, v8::Local<v8::Value> *value);

  bool Nan_NewBuffer(v8::Local<v8::Object> *out, uint32_t size);
  void Node_Buffer_Data(buf_t *out, v8::Local<v8::Object> *obj);
  bool Node_Buffer_Object_HasInstance(v8::Local<v8::Object> *obj);
  bool Node_Buffer_Value_HasInstance(v8::Local<v8::Value> *obj);

  typedef void(*Nan_ChainedScopeCallback)(void *, void *, void *, void *);
  typedef void(*Nan_NestedScopeCallback)(void *, void *, void *);
  typedef void(*Nan_RootScopeCallback)(void *, void *, void *);
  typedef void(*Nan_ModuleScopeCallback)(void *, void *, void *);

  void Nanny_ExecFunctionBody(void *closure, Nan_RootScopeCallback callback, Nan::FunctionCallbackInfo<v8::Value> *info, void *scope);
  void Nan_Nested(void *out, void *closure, Nan_NestedScopeCallback callback, void *realm);
  void Nan_Chained(void *out, void *closure, Nan_ChainedScopeCallback callback, void *parent_scope);

  void Nanny_ExecModuleBody(void *kernel, Nan_ModuleScopeCallback callback, v8::Local<v8::Object> *exports, void *scope);

  bool Nanny_NewFunction(v8::Local<v8::Function> *out, v8::Isolate *isolate, Nan::FunctionCallback callback, void *kernel);
  void *Nanny_FunctionKernel(v8::Local<v8::Object> *obj);
}
