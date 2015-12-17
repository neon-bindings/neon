#include <nan.h>
#include <stdint.h>
#include <v8.h>

typedef struct {
  void* data;
  size_t len;
} buf_t;

// analog Rust enum `Tag` defined in lib.rs
typedef enum {
  tag_null,
  tag_undefined,
  tag_boolean,
  tag_integer,
  tag_number,
  tag_string,
  tag_object,
  tag_array,
  tag_function,
  tag_other
} tag_t;

extern "C" {

  void NeonSys_Call_SetReturn(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value);
  void *NeonSys_Call_GetIsolate(Nan::FunctionCallbackInfo<v8::Value> *info);
  bool NeonSys_Call_IsConstruct(Nan::FunctionCallbackInfo<v8::Value> *info);
  void NeonSys_Call_This(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Object> *out);
  void NeonSys_Call_Callee(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Function> *out);
  void NeonSys_Call_Data(Nan::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> *out);
  int32_t NeonSys_Call_Length(Nan::FunctionCallbackInfo<v8::Value> *info);
  void NeonSys_Call_Get(Nan::FunctionCallbackInfo<v8::Value> *info, int32_t i, v8::Local<v8::Value> *out);

  void NeonSys_Escape(v8::Local<v8::Value> *out, Nan::EscapableHandleScope *scope, v8::Local<v8::Value> value);

  void NeonSys_NewObject(v8::Local<v8::Object> *out);
  void NeonSys_NewInteger(v8::Local<v8::Integer> *out, v8::Isolate *isolate, int32_t x);
  bool NeonSys_NewString(v8::Local<v8::String> *out, v8::Isolate *isolate, const uint8_t *data, int32_t len);
  void NeonSys_NewNumber(v8::Local<v8::Number> *out, v8::Isolate *isolate, double value);
  void NeonSys_NewArray(v8::Local<v8::Array> *out, v8::Isolate *isolate, uint32_t length);
  void NeonSys_NewUndefined(v8::Local<v8::Primitive> *out);
  void NeonSys_NewNull(v8::Local<v8::Primitive> *out);
  void NeonSys_NewBoolean(v8::Local<v8::Boolean> *out, bool b);
  bool NeonSys_NewBuffer(v8::Local<v8::Object> *out, uint32_t size);
  bool NeonSys_NewFunction(v8::Local<v8::Function> *out, v8::Isolate *isolate, Nan::FunctionCallback callback, void *kernel);
  bool NeonSys_NewTypeError(v8::Local<v8::Value> *out, const char *msg);

  bool NeonSys_Object_GetOwnPropertyNames(v8::Local<v8::Array> *out, v8::Local<v8::Object> obj);
  void *NeonSys_Object_GetIsolate(v8::Local<v8::Object> obj);
  bool NeonSys_Object_Get_Index(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, uint32_t index);
  bool NeonSys_Object_Set_Index(bool *out, v8::Local<v8::Object> object, uint32_t index, v8::Local<v8::Value> val);
  bool NeonSys_Object_Get_Bytes(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, const uint8_t *key, int32_t len);
  bool NeonSys_Object_Set_Bytes(bool *out, v8::Local<v8::Object> object, const uint8_t *key, int32_t len, v8::Local<v8::Value> val);
  bool NeonSys_Object_Get(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, v8::Local<v8::Value> key);
  bool NeonSys_Object_Set(bool *out, v8::Local<v8::Object> obj, v8::Local<v8::Value> key, v8::Local<v8::Value> val);

  uint32_t NeonSys_Array_Length(v8::Local<v8::Array> array);

  int32_t NeonSys_String_Utf8Length(v8::Local<v8::String> str);

  bool NeonSys_Value_ToString(v8::Local<v8::String> *out, v8::Local<v8::Value> value);
  bool NeonSys_Value_ToObject(v8::Local<v8::Object> *out, v8::Local<v8::Value> *value);

  void NeonSys_Buffer_Data(buf_t *out, v8::Local<v8::Object> obj);
  bool NeonSys_IsBuffer(v8::Local<v8::Value> obj);

  typedef void(*NeonSys_ChainedScopeCallback)(void *, void *, void *, void *);
  typedef void(*NeonSys_NestedScopeCallback)(void *, void *, void *);
  typedef void(*NeonSys_RootScopeCallback)(void *, void *, void *);
  typedef void(*NeonSys_ModuleScopeCallback)(void *, v8::Local<v8::Object>, void *);

  void NeonSys_Nested(void *out, void *closure, NeonSys_NestedScopeCallback callback, void *realm);
  void NeonSys_Chained(void *out, void *closure, NeonSys_ChainedScopeCallback callback, void *parent_scope);

  void NeonSys_ExecFunctionBody(void *closure, NeonSys_RootScopeCallback callback, Nan::FunctionCallbackInfo<v8::Value> *info, void *scope);
  void NeonSys_ExecModuleBody(void *kernel, NeonSys_ModuleScopeCallback callback, v8::Local<v8::Object> exports, void *scope);

  void *NeonSys_FunctionKernel(v8::Local<v8::Object> obj);

  tag_t NeonSys_TagOf(v8::Local<v8::Value> val);
  bool NeonSys_IsUndefined(v8::Local<v8::Value> val);
  bool NeonSys_IsNull(v8::Local<v8::Value> val);
  bool NeonSys_IsBoolean(v8::Local<v8::Value> val);
  bool NeonSys_IsInteger(v8::Local<v8::Value> val);
  bool NeonSys_IsNumber(v8::Local<v8::Value> val);
  bool NeonSys_IsString(v8::Local<v8::Value> val);
  bool NeonSys_IsObject(v8::Local<v8::Value> val);
  bool NeonSys_IsArray(v8::Local<v8::Value> val);
  bool NeonSys_IsFunction(v8::Local<v8::Value> val);
  bool NeonSys_IsTypeError(v8::Local<v8::Value> val);

  void NeonSys_ThrowAny(v8::Local<v8::Value> val);
  void NeonSys_ThrowTypeError(const char *msg);

  bool NeonSys_SameHandle(v8::Local<v8::Value> v1, v8::Local<v8::Value> v2);
}
