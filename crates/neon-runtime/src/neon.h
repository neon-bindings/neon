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

  void Neon_Call_SetReturn(v8::FunctionCallbackInfo<v8::Value> *info, v8::Persistent<v8::Value> *value);
  void *Neon_Call_GetIsolate(v8::FunctionCallbackInfo<v8::Value> *info);
  void *Neon_Call_CurrentIsolate();
  bool Neon_Call_IsConstruct(v8::FunctionCallbackInfo<v8::Value> *info);
  void Neon_Call_This(v8::FunctionCallbackInfo<v8::Value> *info, v8::Persistent<v8::Value> *out, v8::Isolate *isolate);
  void Neon_Call_Data(v8::FunctionCallbackInfo<v8::Value> *info, v8::Persistent<v8::Value> *out, v8::Isolate *isolate);
  int32_t Neon_Call_Length(v8::FunctionCallbackInfo<v8::Value> *info);
  void Neon_Call_Get(v8::FunctionCallbackInfo<v8::Value> *info, v8::Isolate *isolate, int32_t i, v8::Persistent<v8::Value> *out);

  void Neon_Primitive_Number(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, double value);
  void Neon_Primitive_Undefined(v8::Persistent<v8::Value> *out, v8::Isolate *isolate);
  void Neon_Primitive_Null(v8::Persistent<v8::Value> *out, v8::Isolate *isolate);
  void Neon_Primitive_Boolean(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, bool b);
  bool Neon_Primitive_IsUint32(v8::Local<v8::Primitive> p);
  bool Neon_Primitive_IsInt32(v8::Local<v8::Primitive> p);

  void Neon_Object_New(v8::Persistent<v8::Value> *out, v8::Isolate *isolate);
  bool Neon_Object_GetOwnPropertyNames(v8::Persistent<v8::Array> *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *obj);
  bool Neon_Object_Get_Index(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *object, uint32_t index);
  bool Neon_Object_Set_Index(bool *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *object, uint32_t index, v8::Persistent<v8::Value> *val);
  bool Neon_Object_Get_String(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *object, const uint8_t *key, int32_t len);
  bool Neon_Object_Set_String(bool *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *object, const uint8_t *key, int32_t len, v8::Persistent<v8::Value> *val);
  bool Neon_Object_Get(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *object, v8::Persistent<v8::Value> *key);
  bool Neon_Object_Set(bool *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *obj, v8::Persistent<v8::Value> *key, v8::Persistent<v8::Value> *val);

  void Neon_Array_New(v8::Persistent<v8::Array> *out, v8::Isolate *isolate, uint32_t length);
  uint32_t Neon_Array_Length(v8::Persistent<v8::Array> *array);

  bool Neon_String_New(v8::Persistent<v8::String> *out, v8::Isolate *isolate, const uint8_t *data, int32_t len);
  int32_t Neon_String_Utf8Length(v8::Persistent<v8::String> *str);
  size_t Neon_String_Data(char *out, size_t len, v8::Persistent<v8::String> *str);
  bool Neon_String_ToString(v8::Persistent<v8::String> *out, v8::Isolate *isolate, v8::Persistent<v8::Value> *value);

  bool Neon_Buffer_New(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, uint32_t len);
  bool Neon_Buffer_Uninitialized(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, uint32_t len);
  void Neon_Buffer_Data(void **base_out, size_t *len_out, v8::Persistent<v8::Object> *obj);

  bool Neon_ArrayBuffer_New(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, uint32_t len);
  void Neon_ArrayBuffer_Data(void **base_out, size_t *len_out, v8::Persistent<v8::ArrayBuffer> *buffer);

  typedef void(*Neon_ChainedScopeCallback)(void *, void *, void *, void *);
  typedef void(*Neon_NestedScopeCallback)(void *, void *, void *);
  typedef void(*Neon_RootScopeCallback)(void *, void *, void *);

  void Neon_Scope_ClonePersistent(v8::Isolate *isolate, v8::Persistent<v8::Value> *to, v8::Persistent<v8::Value> *from);
  void Neon_Scope_Nested(void *out, void *closure, Neon_NestedScopeCallback callback, void *realm);
  void Neon_Scope_GetGlobal(v8::Isolate *isolate, v8::Persistent<v8::Value> *out);

  bool Neon_Fun_New(v8::Persistent<v8::Function> *out, v8::Isolate *isolate, callback_t callback);
  bool Neon_Fun_Template_New(v8::Persistent<v8::FunctionTemplate> *out, v8::Isolate *isolate, callback_t callback);
  void *Neon_Fun_GetDynamicCallback(v8::Persistent<v8::External> *obj);
  bool Neon_Fun_Call(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::Function> *fun, v8::Persistent<v8::Value> *self, int32_t argc, v8::Persistent<v8::Value> *argv[]);
  bool Neon_Fun_Construct(v8::Persistent<v8::Object> *out, v8::Isolate *isolate, v8::Persistent<v8::Function> *fun, int32_t argc, v8::Persistent<v8::Value> *argv[]);

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
  void *Neon_Class_GetCallKernel(v8::Persistent<v8::External> *wrapper);
  void *Neon_Class_GetConstructKernel(v8::Persistent<v8::External> *wrapper);
  void *Neon_Class_GetAllocateKernel(v8::Persistent<v8::External> *wrapper);
  bool Neon_Class_HasInstance(void *metadata, v8::Persistent<v8::Value> *v);
  bool Neon_Class_SetName(v8::Isolate *isolate, void *metadata, const char *name, uint32_t byte_length);
  void Neon_Class_GetName(const char **chars_out, size_t *len_out, v8::Isolate *isolate, void *metadata);
  void Neon_Class_ThrowThisError(v8::Isolate *isolate, void *metadata_pointer);
  bool Neon_Class_AddMethod(v8::Isolate *isolate, void *metadata, const char *name, uint32_t byte_length, v8::Persistent<v8::FunctionTemplate> *method);
  bool Neon_Class_MetadataToConstructor(v8::Persistent<v8::Function> *out, v8::Isolate *isolate, void *metadata);
  void *Neon_Class_GetInstanceInternals(v8::Persistent<v8::Object> *obj);

  uint32_t Neon_Module_GetVersion();

  bool Neon_Tag_IsUndefined(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsNull(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsBoolean(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsNumber(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsString(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsObject(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsArray(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsFunction(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsBuffer(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsArrayBuffer(v8::Persistent<v8::Value> *val);
  bool Neon_Tag_IsError(v8::Persistent<v8::Value> *val);

  void Neon_Error_NewError(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::String> *msg);
  void Neon_Error_NewTypeError(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::String> *msg);
  void Neon_Error_NewRangeError(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::String> *msg);
  void Neon_Error_Throw(v8::Persistent<v8::Value> *val);
  void Neon_Error_ThrowErrorFromUtf8(const uint8_t *data, int32_t len);

  void Neon_Mem_NewPersistent(v8::Persistent<v8::Value> *out);
  void Neon_Mem_DropPersistent(v8::Persistent<v8::Value> *p);
  void Neon_Mem_ResetPersistent(v8::Persistent<v8::Value> *p, v8::Local<v8::Value> h);

  typedef void* (*Neon_TaskPerformCallback)(void *);
  typedef void (*Neon_TaskCompleteCallback)(void *, void *, v8::Local<v8::Value> *out);

  void Neon_Task_Schedule(void *task, Neon_TaskPerformCallback perform, Neon_TaskCompleteCallback complete, v8::Persistent<v8::Function> *callback);
}

#endif
