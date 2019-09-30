#ifndef NEON_H
#define NEON_H

#include <nan.h>
#include <stdint.h>
#include <v8.h>

// corresponding Rust struct `CCallback` defined in fun.rs
typedef struct {
  void* static_callback;
  void* dynamic_callback;
} callback_t;

extern "C" {

  void Neon_Call_SetReturn(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value);
  void *Neon_Call_GetIsolate(v8::FunctionCallbackInfo<v8::Value> *info);
  void *Neon_Call_CurrentIsolate();
  bool Neon_Call_IsConstruct(v8::FunctionCallbackInfo<v8::Value> *info);
  void Neon_Call_This(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Object> *out);
  void Neon_Call_Data(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> *out);
  int32_t Neon_Call_Length(v8::FunctionCallbackInfo<v8::Value> *info);
  void Neon_Call_Get(v8::FunctionCallbackInfo<v8::Value> *info, int32_t i, v8::Local<v8::Value> *out);

  void Neon_Primitive_Number(v8::Local<v8::Number> *out, v8::Isolate *isolate, double value);
  void Neon_Primitive_Undefined(v8::Local<v8::Primitive> *out);
  void Neon_Primitive_Null(v8::Local<v8::Primitive> *out);
  void Neon_Primitive_Boolean(v8::Local<v8::Boolean> *out, bool b);
  bool Neon_Primitive_IsUint32(v8::Local<v8::Primitive> p);
  bool Neon_Primitive_IsInt32(v8::Local<v8::Primitive> p);

  void Neon_Object_New(v8::Local<v8::Object> *out);
  bool Neon_Object_GetOwnPropertyNames(v8::Local<v8::Array> *out, v8::Local<v8::Object> obj);
  void *Neon_Object_GetIsolate(v8::Local<v8::Object> obj);
  bool Neon_Object_Get_Index(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, uint32_t index);
  bool Neon_Object_Set_Index(bool *out, v8::Local<v8::Object> object, uint32_t index, v8::Local<v8::Value> val);
  bool Neon_Object_Get_String(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, const uint8_t *key, int32_t len);
  bool Neon_Object_Set_String(bool *out, v8::Local<v8::Object> object, const uint8_t *key, int32_t len, v8::Local<v8::Value> val);
  bool Neon_Object_Get(v8::Local<v8::Value> *out, v8::Local<v8::Object> object, v8::Local<v8::Value> key);
  bool Neon_Object_Set(bool *out, v8::Local<v8::Object> obj, v8::Local<v8::Value> key, v8::Local<v8::Value> val);

  void Neon_Array_New(v8::Local<v8::Array> *out, v8::Isolate *isolate, uint32_t length);
  uint32_t Neon_Array_Length(v8::Local<v8::Array> array);

  bool Neon_String_New(v8::Local<v8::String> *out, v8::Isolate *isolate, const uint8_t *data, int32_t len);
  int32_t Neon_String_Utf8Length(v8::Local<v8::String> str);
  size_t Neon_String_Data(char *out, size_t len, v8::Local<v8::Value> str);

  bool Neon_Convert_ToString(v8::Local<v8::String> *out, v8::Local<v8::Value> value);
  bool Neon_Convert_ToObject(v8::Local<v8::Object> *out, v8::Local<v8::Value> *value);

  bool Neon_Buffer_New(v8::Local<v8::Object> *out, uint32_t size);
  size_t Neon_Buffer_Data(void **base_out, v8::Local<v8::Object> obj);

  bool Neon_ArrayBuffer_New(v8::Local<v8::ArrayBuffer> *out, v8::Isolate *isolate, uint32_t size);
  bool Neon_ArrayBuffer_Uninitialized(v8::Local<v8::ArrayBuffer> *out, v8::Isolate *isolate, uint32_t size);
  size_t Neon_ArrayBuffer_Data(void **base_out, v8::Local<v8::ArrayBuffer> buffer);

  typedef void(*Neon_ChainedScopeCallback)(void *, void *, void *, void *);
  typedef void(*Neon_NestedScopeCallback)(void *, void *, void *);
  typedef void(*Neon_RootScopeCallback)(void *, void *, void *);

  void Neon_Scope_Escape(v8::Local<v8::Value> *out, Nan::EscapableHandleScope *scope, v8::Local<v8::Value> value);
  void Neon_Scope_Nested(void *out, void *closure, Neon_NestedScopeCallback callback, void *realm);
  void Neon_Scope_Chained(void *out, void *closure, Neon_ChainedScopeCallback callback, void *parent_scope);
  void Neon_Scope_Enter(v8::HandleScope *scope, v8::Isolate *isolate);
  void Neon_Scope_Exit(v8::HandleScope *scope);
  void Neon_Scope_Enter_Escapable(v8::EscapableHandleScope *scope, v8::Isolate *isolate);
  void Neon_Scope_Exit_Escapable(v8::EscapableHandleScope *scope);
  size_t Neon_Scope_Sizeof();
  size_t Neon_Scope_Alignof();
  size_t Neon_Scope_SizeofEscapable();
  size_t Neon_Scope_AlignofEscapable();
  void Neon_Scope_GetGlobal(v8::Isolate *isolate, v8::Local<v8::Value> *out);

  bool Neon_Fun_New(v8::Local<v8::Function> *out, v8::Isolate *isolate, callback_t callback);
  bool Neon_Fun_Template_New(v8::Local<v8::FunctionTemplate> *out, v8::Isolate *isolate, callback_t callback);
  void *Neon_Fun_GetDynamicCallback(v8::Local<v8::External> obj);
  bool Neon_Fun_Call(v8::Local<v8::Value> *out, v8::Isolate *isolate, v8::Local<v8::Function> fun, v8::Local<v8::Value> self, int32_t argc, v8::Local<v8::Value> argv[]);
  bool Neon_Fun_Construct(v8::Local<v8::Object> *out, v8::Isolate *isolate, v8::Local<v8::Function> fun, int32_t argc, v8::Local<v8::Value> argv[]);

  typedef void *(*Neon_AllocateCallback)(const v8::FunctionCallbackInfo<v8::Value> *info);
  typedef bool (*Neon_ConstructCallback)(const v8::FunctionCallbackInfo<v8::Value> *info);

  void Neon_Class_ForConstructor(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::FunctionTemplate> *out);
  void Neon_Class_ForMethod(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::FunctionTemplate> *out);

  typedef void (*Neon_DropCallback)(void *);

  void *Neon_Class_GetClassMap(v8::Isolate *isolate);
  void Neon_Class_SetClassMap(v8::Isolate *isolate, void *map, Neon_DropCallback free_map);
  void *Neon_Class_CreateBase(v8::Isolate *isolate,
                              callback_t allocate,
                              callback_t construct,
                              callback_t call,
                              Neon_DropCallback drop);
  // FIXME: get rid of all the "kernel" nomenclature
  void *Neon_Class_GetCallKernel(v8::Local<v8::External> wrapper);
  void *Neon_Class_GetConstructKernel(v8::Local<v8::External> wrapper);
  void *Neon_Class_GetAllocateKernel(v8::Local<v8::External> wrapper);
  bool Neon_Class_Constructor(v8::Local<v8::Function> *out, v8::Local<v8::FunctionTemplate> ft);
  bool Neon_Class_HasInstance(void *metadata, v8::Local<v8::Value> v);
  bool Neon_Class_SetName(v8::Isolate *isolate, void *metadata, const char *name, uint32_t byte_length);
  size_t Neon_Class_GetName(const char **chars_out, v8::Isolate *isolate, void *metadata);
  void Neon_Class_ThrowThisError(v8::Isolate *isolate, void *metadata_pointer);
  bool Neon_Class_AddMethod(v8::Isolate *isolate, void *metadata, const char *name, uint32_t byte_length, v8::Local<v8::FunctionTemplate> method);
  bool Neon_Class_MetadataToConstructor(v8::Local<v8::Function> *out, v8::Isolate *isolate, void *metadata);
  void *Neon_Class_GetInstanceInternals(v8::Local<v8::Object> obj);

  uint32_t Neon_Module_GetVersion();

  bool Neon_Tag_IsUndefined(v8::Local<v8::Value> val);
  bool Neon_Tag_IsNull(v8::Local<v8::Value> val);
  bool Neon_Tag_IsBoolean(v8::Local<v8::Value> val);
  bool Neon_Tag_IsNumber(v8::Local<v8::Value> val);
  bool Neon_Tag_IsString(v8::Local<v8::Value> val);
  bool Neon_Tag_IsObject(v8::Local<v8::Value> val);
  bool Neon_Tag_IsArray(v8::Local<v8::Value> val);
  bool Neon_Tag_IsFunction(v8::Local<v8::Value> val);
  bool Neon_Tag_IsBuffer(v8::Local<v8::Value> obj);
  bool Neon_Tag_IsArrayBuffer(v8::Local<v8::Value> obj);
  bool Neon_Tag_IsError(v8::Local<v8::Value> val);

  void Neon_Error_NewError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg);
  void Neon_Error_NewTypeError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg);
  void Neon_Error_NewRangeError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg);
  void Neon_Error_Throw(v8::Local<v8::Value> val);
  void Neon_Error_ThrowErrorFromUtf8(const uint8_t *data, int32_t len);

  bool Neon_Mem_SameHandle(v8::Local<v8::Value> v1, v8::Local<v8::Value> v2);

  typedef void* (*Neon_TaskPerformCallback)(void *);
  typedef void (*Neon_TaskCompleteCallback)(void *, void *, v8::Local<v8::Value> *out);

  void Neon_Task_Schedule(void *task, Neon_TaskPerformCallback perform, Neon_TaskCompleteCallback complete, v8::Local<v8::Function> callback);
}

#endif
