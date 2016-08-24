#ifndef NEON_H
#define NEON_H

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

  void NeonSys_Call_SetReturn(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value);
  void *NeonSys_Call_GetIsolate(v8::FunctionCallbackInfo<v8::Value> *info);
  void *NeonSys_Call_CurrentIsolate();
  bool NeonSys_Call_IsConstruct(v8::FunctionCallbackInfo<v8::Value> *info);
  void NeonSys_Call_This(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Object> *out);
  void NeonSys_Call_Callee(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Function> *out);
  void NeonSys_Call_Data(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> *out);
  int32_t NeonSys_Call_Length(v8::FunctionCallbackInfo<v8::Value> *info);
  void NeonSys_Call_Get(v8::FunctionCallbackInfo<v8::Value> *info, int32_t i, v8::Local<v8::Value> *out);

  void NeonSys_Primitive_Integer(v8::Local<v8::Integer> *out, v8::Isolate *isolate, int32_t x);
  void NeonSys_Primitive_Number(v8::Local<v8::Number> *out, v8::Isolate *isolate, double value);
  void NeonSys_Primitive_Undefined(v8::Local<v8::Primitive> *out);
  void NeonSys_Primitive_Null(v8::Local<v8::Primitive> *out);
  void NeonSys_Primitive_Boolean(v8::Local<v8::Boolean> *out, bool b);
  bool NeonSys_Primitive_IsUint32(v8::Local<v8::Primitive> p);
  bool NeonSys_Primitive_IsInt32(v8::Local<v8::Primitive> p);
  int64_t NeonSys_Primitive_IntegerValue(v8::Local<v8::Integer> i);

  void NeonSys_Object_New(v8::Local<v8::Object> *out);
  bool NeonSys_Object_GetOwnPropertyNames(v8::Local<v8::Array> *out, v8::Local<v8::Object> obj);
  void *NeonSys_Object_GetIsolate(v8::Local<v8::Object> obj);
  bool NeonSys_Object_Get_Index(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, uint32_t index);
  bool NeonSys_Object_Set_Index(bool *out, v8::Local<v8::Object> object, uint32_t index, v8::Local<v8::Value> val);
  bool NeonSys_Object_Get_String(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, const uint8_t *key, int32_t len);
  bool NeonSys_Object_Set_String(bool *out, v8::Local<v8::Object> object, const uint8_t *key, int32_t len, v8::Local<v8::Value> val);
  bool NeonSys_Object_Get(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, v8::Local<v8::Value> key);
  bool NeonSys_Object_Set(bool *out, v8::Local<v8::Object> obj, v8::Local<v8::Value> key, v8::Local<v8::Value> val);

  void NeonSys_Array_New(v8::Local<v8::Array> *out, v8::Isolate *isolate, uint32_t length);
  uint32_t NeonSys_Array_Length(v8::Local<v8::Array> array);

  bool NeonSys_String_New(v8::Local<v8::String> *out, v8::Isolate *isolate, const uint8_t *data, int32_t len);
  int32_t NeonSys_String_Utf8Length(v8::Local<v8::String> str);
  size_t NeonSys_String_Data(char *out, size_t len, v8::Local<v8::Value> str);

  bool NeonSys_Convert_ToString(v8::Local<v8::String> *out, v8::Local<v8::Value> value);
  bool NeonSys_Convert_ToObject(v8::Local<v8::Object> *out, v8::Local<v8::Value> *value);

  bool NeonSys_Buffer_New(v8::Local<v8::Object> *out, uint32_t size);
  void NeonSys_Buffer_Data(buf_t *out, v8::Local<v8::Object> obj);

  typedef void(*NeonSys_ChainedScopeCallback)(void *, void *, void *, void *);
  typedef void(*NeonSys_NestedScopeCallback)(void *, void *, void *);
  typedef void(*NeonSys_RootScopeCallback)(void *, void *, void *);
  typedef void(*NeonSys_ModuleScopeCallback)(void *, v8::Local<v8::Object>, void *);

  void NeonSys_Scope_Escape(v8::Local<v8::Value> *out, Nan::EscapableHandleScope *scope, v8::Local<v8::Value> value);
  void NeonSys_Scope_Nested(void *out, void *closure, NeonSys_NestedScopeCallback callback, void *realm);
  void NeonSys_Scope_Chained(void *out, void *closure, NeonSys_ChainedScopeCallback callback, void *parent_scope);
  void NeonSys_Scope_Enter(v8::HandleScope *scope, v8::Isolate *isolate);
  void NeonSys_Scope_Exit(v8::HandleScope *scope);
  size_t NeonSys_Scope_Sizeof();
  size_t NeonSys_Scope_SizeofEscapable();

  bool NeonSys_Fun_New(v8::Local<v8::Function> *out, v8::Isolate *isolate, v8::FunctionCallback callback, void *kernel);
  void NeonSys_Fun_ExecKernel(void *kernel, NeonSys_RootScopeCallback callback, v8::FunctionCallbackInfo<v8::Value> *info, void *scope);
  void *NeonSys_Fun_GetKernel(v8::Local<v8::External> obj);
  bool NeonSys_Fun_Call(v8::Local<v8::Value> *out, v8::Isolate *isolate, v8::Local<v8::Function> fun, v8::Local<v8::Value> self, int32_t argc, v8::Local<v8::Value> argv[]);
  bool NeonSys_Fun_Construct(v8::Local<v8::Object> *out, v8::Isolate *isolate, v8::Local<v8::Function> fun, int32_t argc, v8::Local<v8::Value> argv[]);

  typedef void *(*NeonSys_AllocateCallback)(const v8::FunctionCallbackInfo<v8::Value> *info);
  typedef bool (*NeonSys_ConstructCallback)(const v8::FunctionCallbackInfo<v8::Value> *info);

  void NeonSys_Class_ForConstructor(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::FunctionTemplate> *out);
  void NeonSys_Class_ForMethod(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::FunctionTemplate> *out);

  typedef void (*NeonSys_DropCallback)(void *);

  void *NeonSys_Class_GetClassMap(v8::Isolate *isolate);
  void NeonSys_Class_SetClassMap(v8::Isolate *isolate, void *map, NeonSys_DropCallback free_map);
  void *NeonSys_Class_CreateBase(v8::Isolate *isolate,
                                 NeonSys_AllocateCallback allocate_callback,
                                 void *allocate_kernel,
                                 NeonSys_ConstructCallback construct_callback,
                                 void *construct_kernel,
                                 v8::FunctionCallback call_callback,
                                 void *call_kernel,
                                 NeonSys_DropCallback drop);
  void *NeonSys_Class_GetCallKernel(v8::Local<v8::External> wrapper);
  void *NeonSys_Class_GetConstructKernel(v8::Local<v8::External> wrapper);
  void *NeonSys_Class_GetAllocateKernel(v8::Local<v8::External> wrapper);
  bool NeonSys_Class_Constructor(v8::Local<v8::Function> *out, v8::Local<v8::FunctionTemplate> ft);
  bool NeonSys_Class_Check(v8::Local<v8::FunctionTemplate> ft, v8::Local<v8::Value> v);
  bool NeonSys_Class_HasInstance(void *metadata, v8::Local<v8::Value> v);
  bool NeonSys_Class_SetName(v8::Isolate *isolate, void *metadata, const char *name, uint32_t byte_length);
  void NeonSys_Class_ThrowThisError(v8::Isolate *isolate, void *metadata_pointer);
  bool NeonSys_Class_AddMethod(v8::Isolate *isolate, void *metadata, const char *name, uint32_t byte_length, v8::Local<v8::Function> method);
  void NeonSys_Class_MetadataToClass(v8::Local<v8::FunctionTemplate> *out, v8::Isolate *isolate, void *metadata);
  void *NeonSys_Class_GetInstanceInternals(v8::Local<v8::Object> obj);

  void NeonSys_Module_ExecKernel(void *kernel, NeonSys_ModuleScopeCallback callback, v8::Local<v8::Object> exports, void *scope);

  tag_t NeonSys_Tag_Of(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsUndefined(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsNull(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsBoolean(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsInteger(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsNumber(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsString(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsObject(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsArray(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsFunction(v8::Local<v8::Value> val);
  bool NeonSys_Tag_IsBuffer(v8::Local<v8::Value> obj);
  bool NeonSys_Tag_IsError(v8::Local<v8::Value> val);

  void NeonSys_Error_NewError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg);
  void NeonSys_Error_NewTypeError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg);
  void NeonSys_Error_NewReferenceError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg);
  void NeonSys_Error_NewRangeError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg);
  void NeonSys_Error_NewSyntaxError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg);
  void NeonSys_Error_Throw(v8::Local<v8::Value> val);
  void NeonSys_Error_ThrowErrorFromCString(const char *msg);
  void NeonSys_Error_ThrowTypeErrorFromCString(const char *msg);
  void NeonSys_Error_ThrowReferenceErrorFromCString(const char *msg);
  void NeonSys_Error_ThrowRangeErrorFromCString(const char *msg);
  void NeonSys_Error_ThrowSyntaxErrorFromCString(const char *msg);

  bool NeonSys_Mem_SameHandle(v8::Local<v8::Value> v1, v8::Local<v8::Value> v2);
}

#endif
