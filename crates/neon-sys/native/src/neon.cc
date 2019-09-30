#include <new>
#include <nan.h>
#include <stdint.h>
#include <stdio.h>
#include "node.h"
#include "node_version.h"
#include "neon.h"
#include "neon_string.h"
#include "neon_class_metadata.h"
#include "neon_task.h"

extern "C" void Neon_Call_SetReturn(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value) {
  info->GetReturnValue().Set(value);
}

extern "C" void *Neon_Call_GetIsolate(v8::FunctionCallbackInfo<v8::Value> *info) {
  return (void *)info->GetIsolate();
}

extern "C" void *Neon_Call_CurrentIsolate() {
  return (void *)v8::Isolate::GetCurrent();
}

extern "C" bool Neon_Call_IsConstruct(v8::FunctionCallbackInfo<v8::Value> *info) {
  return info->IsConstructCall();
}

extern "C" void Neon_Call_This(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Object> *out) {
  *out = info->This();
}

extern "C" void Neon_Call_Data(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> *out) {
  /*
  printf("Call_Data: v8 info  = %p\n", *(void **)info);
  dump((void *)info, 3);
  printf("Call_Data: v8 info implicit:\n");
  dump_implicit((void *)info);
  */
  *out = info->Data();
}

extern "C" int32_t Neon_Call_Length(v8::FunctionCallbackInfo<v8::Value> *info) {
  return info->Length();
}

extern "C" void Neon_Call_Get(v8::FunctionCallbackInfo<v8::Value> *info, int32_t i, v8::Local<v8::Value> *out) {
  *out = (*info)[i];
}

extern "C" void Neon_Object_New(v8::Local<v8::Object> *out) {
  *out = Nan::New<v8::Object>();
}

extern "C" bool Neon_Object_GetOwnPropertyNames(v8::Local<v8::Array> *out, v8::Local<v8::Object> obj) {
  Nan::MaybeLocal<v8::Array> maybe = Nan::GetOwnPropertyNames(obj);
  return maybe.ToLocal(out);
}

extern "C" void *Neon_Object_GetIsolate(v8::Local<v8::Object> obj) {
  return obj->GetIsolate();
}

extern "C" void Neon_Primitive_Undefined(v8::Local<v8::Primitive> *out) {
  *out = Nan::Undefined();
}

extern "C" void Neon_Primitive_Null(v8::Local<v8::Primitive> *out) {
  *out = Nan::Null();
}

extern "C" void Neon_Primitive_Boolean(v8::Local<v8::Boolean> *out, bool b) {
  *out = b ? Nan::True() : Nan::False();
}

extern "C" bool Neon_Primitive_BooleanValue(v8::Local<v8::Boolean> p) {
  return p->Value();
}

extern "C" void Neon_Primitive_Number(v8::Local<v8::Number> *out, v8::Isolate *isolate, double value) {
  *out = v8::Number::New(isolate, value);
}

extern "C" double Neon_Primitive_NumberValue(v8::Local<v8::Number> n) {
  return n->Value();
}

extern "C" bool Neon_Primitive_IsUint32(v8::Local<v8::Primitive> p) {
  return p->IsUint32();
}

extern "C" bool Neon_Primitive_IsInt32(v8::Local<v8::Primitive> p) {
  return p->IsInt32();
}

extern "C" bool Neon_Object_Get_Index(v8::Local<v8::Value> *out, v8::Local<v8::Object> obj, uint32_t index) {
  Nan::MaybeLocal<v8::Value> maybe = Nan::Get(obj, index);
  return maybe.ToLocal(out);
}

extern "C" bool Neon_Object_Set_Index(bool *out, v8::Local<v8::Object> object, uint32_t index, v8::Local<v8::Value> val) {
  Nan::Maybe<bool> maybe = Nan::Set(object, index, val);
  return maybe.IsJust() && (*out = maybe.FromJust(), true);
}

bool Neon_ASCII_Key(v8::Local<v8::String> *key, const uint8_t *data, int32_t len) {
  Nan::MaybeLocal<v8::String> maybe_key = v8::String::NewFromUtf8(v8::Isolate::GetCurrent(), (const char*)data, v8::NewStringType::kNormal, len);
  return maybe_key.ToLocal(key);
}

extern "C" bool Neon_Object_Get_String(v8::Local<v8::Value> *out, v8::Local<v8::Object> obj, const uint8_t *data, int32_t len) {
  Nan::EscapableHandleScope scope;
  v8::Local<v8::String> key;
  if (!Neon_ASCII_Key(&key, data, len)) {
    return false;
  }
  Nan::MaybeLocal<v8::Value> maybe = Nan::Get(obj, key);
  v8::Local<v8::Value> result;
  if (!maybe.ToLocal(&result)) {
    return false;
  }
  *out = scope.Escape(result);
  return true;
}

extern "C" bool Neon_Object_Set_String(bool *out, v8::Local<v8::Object> obj, const uint8_t *data, int32_t len, v8::Local<v8::Value> val) {
  Nan::HandleScope scope;
  v8::Local<v8::String> key;
  if (!Neon_ASCII_Key(&key, data, len)) {
    return false;
  }
  Nan::Maybe<bool> maybe = Nan::Set(obj, key, val);
  return maybe.IsJust() && (*out = maybe.FromJust(), true);
}

extern "C" bool Neon_Object_Get(v8::Local<v8::Value> *out, v8::Local<v8::Object> obj, v8::Local<v8::Value> key) {
  Nan::MaybeLocal<v8::Value> maybe = Nan::Get(obj, key);
  return maybe.ToLocal(out);
}

extern "C" bool Neon_Object_Set(bool *out, v8::Local<v8::Object> obj, v8::Local<v8::Value> key, v8::Local<v8::Value> val) {
  Nan::Maybe<bool> maybe = Nan::Set(obj, key, val);
  if (maybe.IsJust()) {
    *out = maybe.FromJust();
    return true;
  }
  return false;
}

extern "C" void Neon_Array_New(v8::Local<v8::Array> *out, v8::Isolate *isolate, uint32_t length) {
  *out = v8::Array::New(isolate, length);
}

extern "C" uint32_t Neon_Array_Length(v8::Local<v8::Array> array) {
  return array->Length();
}

extern "C" bool Neon_String_New(v8::Local<v8::String> *out, v8::Isolate *isolate, const uint8_t *data, int32_t len) {
  Nan::MaybeLocal<v8::String> maybe = v8::String::NewFromUtf8(isolate, (const char*)data, v8::NewStringType::kNormal, len);
  return maybe.ToLocal(out);
}

extern "C" int32_t Neon_String_Utf8Length(v8::Local<v8::String> str) {
  #if NODE_MODULE_VERSION >= NODE_11_0_MODULE_VERSION
    return str->Utf8Length(v8::Isolate::GetCurrent());
  #else
    return str->Utf8Length();
  #endif
}

extern "C" size_t Neon_String_Data(char *out, size_t len, v8::Local<v8::Value> str) {
  return Nan::DecodeWrite(out, len, str, Nan::UTF8);
}

extern "C" bool Neon_Convert_ToString(v8::Local<v8::String> *out, v8::Local<v8::Value> value) {
  Nan::MaybeLocal<v8::String> maybe = Nan::To<v8::String>(value);
  return maybe.ToLocal(out);
}

extern "C" bool Neon_Convert_ToObject(v8::Local<v8::Object> *out, v8::Local<v8::Value> *value) {
  Nan::MaybeLocal<v8::Object> maybe = Nan::To<v8::Object>(*value);
  return maybe.ToLocal(out);
}

extern "C" bool Neon_Buffer_New(v8::Local<v8::Object> *out, uint32_t size) {
  Nan::MaybeLocal<v8::Object> maybe = Nan::NewBuffer(size);
  if (!maybe.ToLocal(out)) {
    return false;
  }

  void *data = node::Buffer::Data(*out);
  memset(data, 0, size);
  return true;
}

extern "C" bool Neon_Buffer_Uninitialized(v8::Local<v8::Object> *out, uint32_t size) {
  Nan::MaybeLocal<v8::Object> maybe = Nan::NewBuffer(size);
  return maybe.ToLocal(out);
}

extern "C" size_t Neon_Buffer_Data(void **base_out, v8::Local<v8::Object> obj) {
  *base_out = node::Buffer::Data(obj);

  return node::Buffer::Length(obj);
}

extern "C" bool Neon_Tag_IsBuffer(v8::Local<v8::Value> obj) {
  return node::Buffer::HasInstance(obj);
}

extern "C" bool Neon_ArrayBuffer_New(v8::Local<v8::ArrayBuffer> *out, v8::Isolate *isolate, uint32_t size) {
  *out = v8::ArrayBuffer::New(isolate, size);
  return true;
}

extern "C" size_t Neon_ArrayBuffer_Data(void **base_out, v8::Local<v8::ArrayBuffer> buffer) {
  v8::ArrayBuffer::Contents contents = buffer->GetContents();
  *base_out = contents.Data();

  return contents.ByteLength();
}


extern "C" bool Neon_Tag_IsArrayBuffer(v8::Local<v8::Value> value) {
  return value->IsArrayBuffer();
}

extern "C" void Neon_Scope_Escape(v8::Local<v8::Value> *out, Nan::EscapableHandleScope *scope, v8::Local<v8::Value> value) {
  *out = scope->Escape(value);
}

extern "C" void Neon_Scope_Chained(void *out, void *closure, Neon_ChainedScopeCallback callback, void *parent_scope) {
  Nan::EscapableHandleScope v8_scope;
  callback(out, parent_scope, &v8_scope, closure);
}

extern "C" void Neon_Scope_Nested(void *out, void *closure, Neon_NestedScopeCallback callback, void *realm) {
  Nan::HandleScope v8_scope;
  callback(out, realm, closure);
}

extern "C" void Neon_Scope_Enter(v8::HandleScope *scope, v8::Isolate *isolate) {
  void *p = scope;
  ::new (p) v8::HandleScope(isolate);
}

extern "C" void Neon_Scope_Exit(v8::HandleScope *scope) {
  scope->HandleScope::~HandleScope();
}
extern "C" void Neon_Scope_Enter_Escapable(v8::EscapableHandleScope *scope, v8::Isolate *isolate) {
  void *p = scope;
  ::new (p) v8::EscapableHandleScope(isolate);
}

extern "C" void Neon_Scope_Exit_Escapable(v8::EscapableHandleScope *scope) {
  scope->EscapableHandleScope::~EscapableHandleScope();
}

extern "C" size_t Neon_Scope_Sizeof() {
  return sizeof(v8::HandleScope);
}

extern "C" size_t Neon_Scope_Alignof() {
  return alignof(v8::HandleScope);
}

extern "C" size_t Neon_Scope_SizeofEscapable() {
  return sizeof(v8::EscapableHandleScope);
}

extern "C" size_t Neon_Scope_AlignofEscapable() {
  return alignof(v8::EscapableHandleScope);
}

extern "C" void Neon_Scope_GetGlobal(v8::Isolate *isolate, v8::Local<v8::Value> *out) {
  auto ctx = isolate->GetCurrentContext();
  *out = ctx->Global();
}

extern "C" uint32_t Neon_Module_GetVersion() {
  return NODE_MODULE_VERSION;
}

extern "C" void Neon_Class_ConstructBaseCallback(const v8::FunctionCallbackInfo<v8::Value>& info) {
  Nan::HandleScope scope;
  v8::Local<v8::External> wrapper = v8::Local<v8::External>::Cast(info.Data());
  neon::BaseClassMetadata *metadata = static_cast<neon::BaseClassMetadata *>(wrapper->Value());
  if (info.IsConstructCall()) {
    metadata->construct(info);
  } else {
    metadata->call(info);
  }
}

extern "C" void *Neon_Class_CreateBase(v8::Isolate *isolate,
                                       callback_t allocate,
                                       callback_t construct,
                                       callback_t call,
                                       Neon_DropCallback drop)
{
  Neon_AllocateCallback allocate_callback = reinterpret_cast<Neon_AllocateCallback>(allocate.static_callback);
  Neon_ConstructCallback construct_callback = reinterpret_cast<Neon_ConstructCallback>(construct.static_callback);
  v8::FunctionCallback call_callback = reinterpret_cast<v8::FunctionCallback>(call.static_callback);
  neon::BaseClassMetadata *metadata = new neon::BaseClassMetadata(construct_callback, construct.dynamic_callback, call_callback, call.dynamic_callback, allocate_callback, allocate.dynamic_callback, drop);
  v8::Local<v8::External> data = v8::External::New(isolate, metadata);
  v8::Local<v8::FunctionTemplate> constructor_template = v8::FunctionTemplate::New(isolate, Neon_Class_ConstructBaseCallback, data);
  metadata->SetTemplate(isolate, constructor_template);
  v8::Local<v8::ObjectTemplate> instance_template = constructor_template->InstanceTemplate();
  instance_template->SetInternalFieldCount(1); // index 0: an aligned, owned pointer to the internals (a user-defined Rust data structure)
  return metadata;
}

extern "C" void *Neon_Class_GetClassMap(v8::Isolate *isolate) {
  neon::ClassMapHolder *holder = static_cast<neon::ClassMapHolder *>(isolate->GetData(NEON_ISOLATE_SLOT));
  return (holder == nullptr)
       ? nullptr
       : holder->GetMap();
}

void cleanup_class_map(void *arg) {
  neon::ClassMapHolder *holder = static_cast<neon::ClassMapHolder *>(arg);
  delete holder;
}

extern "C" void Neon_Class_SetClassMap(v8::Isolate *isolate, void *map, Neon_DropCallback drop_map) {
  neon::ClassMapHolder *holder = new neon::ClassMapHolder(map, drop_map);
  isolate->SetData(NEON_ISOLATE_SLOT, holder);
  // ISSUE(#77): When workers land in node, this will need to be generalized to a per-worker version.
  node::AtExit(cleanup_class_map, holder);
}

extern "C" void *Neon_Class_GetCallKernel(v8::Local<v8::External> wrapper) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(wrapper->Value());
  return metadata->GetCallKernel();
}

extern "C" void *Neon_Class_GetConstructKernel(v8::Local<v8::External> wrapper) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(wrapper->Value());
  return metadata->GetConstructKernel();
}

extern "C" void *Neon_Class_GetAllocateKernel(v8::Local<v8::External> wrapper) {
  neon::BaseClassMetadata *metadata = static_cast<neon::BaseClassMetadata *>(wrapper->Value());
  return metadata->GetAllocateKernel();
}

extern "C" bool Neon_Class_Constructor(v8::Local<v8::Function> *out, v8::Local<v8::FunctionTemplate> ft) {
  v8::MaybeLocal<v8::Function> maybe = Nan::GetFunction(ft);
  return maybe.ToLocal(out);
}

extern "C" bool Neon_Class_HasInstance(void *metadata_pointer, v8::Local<v8::Value> v) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  return metadata->GetTemplate(v8::Isolate::GetCurrent())->HasInstance(v);
}

extern "C" bool Neon_Class_SetName(v8::Isolate *isolate, void *metadata_pointer, const char *name, uint32_t byte_length) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  v8::Local<v8::FunctionTemplate> ft = metadata->GetTemplate(isolate);
  v8::MaybeLocal<v8::String> maybe_class_name = v8::String::NewFromUtf8(isolate, name, v8::NewStringType::kNormal, byte_length);
  v8::Local<v8::String> class_name;
  if (!maybe_class_name.ToLocal(&class_name)) {
    return false;
  }
  ft->SetClassName(class_name);
  metadata->SetName(neon::Slice(name, byte_length));
  return true;
}

extern "C" size_t Neon_Class_GetName(const char **chars_out, v8::Isolate *isolate, void *metadata_pointer) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  neon::Slice name = metadata->GetName();
  *chars_out = name.GetBuffer();

  return name.GetLength();
}

extern "C" void Neon_Class_ThrowCallError(v8::Isolate *isolate, void *metadata_pointer) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  Nan::ThrowTypeError(metadata->GetCallError().ToJsString(isolate, "constructor called without new."));
}


extern "C" void Neon_Class_ThrowThisError(v8::Isolate *isolate, void *metadata_pointer) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  Nan::ThrowTypeError(metadata->GetThisError().ToJsString(isolate, "this is not an object of the expected type."));
}

extern "C" bool Neon_Class_AddMethod(v8::Isolate *isolate, void *metadata_pointer, const char *name, uint32_t byte_length, v8::Local<v8::FunctionTemplate> method) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  v8::Local<v8::FunctionTemplate> ft = metadata->GetTemplate(isolate);
  v8::Local<v8::ObjectTemplate> pt = ft->PrototypeTemplate();
  Nan::HandleScope scope;
  v8::MaybeLocal<v8::String> maybe_key = v8::String::NewFromUtf8(isolate, name, v8::NewStringType::kNormal, byte_length);
  v8::Local<v8::String> key;
  if (!maybe_key.ToLocal(&key)) {
    return false;
  }
  pt->Set(key, method);
  return true;
}

extern "C" bool Neon_Class_MetadataToConstructor(v8::Local<v8::Function> *out, v8::Isolate *isolate, void *metadata) {
  v8::Local<v8::FunctionTemplate> ft = static_cast<neon::ClassMetadata *>(metadata)->GetTemplate(isolate);
  v8::MaybeLocal<v8::Function> maybe = Nan::GetFunction(ft);
  return maybe.ToLocal(out);
}

extern "C" void *Neon_Class_GetInstanceInternals(v8::Local<v8::Object> obj) {
  return static_cast<neon::BaseClassInstanceMetadata *>(obj->GetAlignedPointerFromInternalField(0))->GetInternals();
}

extern "C" bool Neon_Fun_Template_New(v8::Local<v8::FunctionTemplate> *out, v8::Isolate *isolate, callback_t callback) {
  v8::Local<v8::External> wrapper = v8::External::New(isolate, callback.dynamic_callback);
  if (wrapper.IsEmpty()) {
    return false;
  }

  v8::FunctionCallback static_callback = reinterpret_cast<v8::FunctionCallback>(callback.static_callback);
  v8::MaybeLocal<v8::FunctionTemplate> maybe_result = v8::FunctionTemplate::New(isolate, static_callback, wrapper);
  return maybe_result.ToLocal(out);
}

extern "C" bool Neon_Fun_New(v8::Local<v8::Function> *out, v8::Isolate *isolate, callback_t callback) {
  v8::Local<v8::External> wrapper = v8::External::New(isolate, callback.dynamic_callback);
  if (wrapper.IsEmpty()) {
    return false;
  }

  v8::FunctionCallback static_callback = reinterpret_cast<v8::FunctionCallback>(callback.static_callback);
  v8::MaybeLocal<v8::Function> maybe_result = v8::Function::New(isolate->GetCurrentContext(), static_callback, wrapper);
  return maybe_result.ToLocal(out);
}

extern "C" void *Neon_Fun_GetDynamicCallback(v8::Local<v8::External> data) {
  return data->Value();
}

extern "C" bool Neon_Fun_Call(v8::Local<v8::Value> *out, v8::Isolate *isolate, v8::Local<v8::Function> fun, v8::Local<v8::Value> self, int32_t argc, v8::Local<v8::Value> argv[]) {
  v8::MaybeLocal<v8::Value> maybe_result = fun->Call(isolate->GetCurrentContext(), self, argc, argv);
  return maybe_result.ToLocal(out);
}

extern "C" bool Neon_Fun_Construct(v8::Local<v8::Object> *out, v8::Isolate *isolate, v8::Local<v8::Function> fun, int32_t argc, v8::Local<v8::Value> argv[]) {
  v8::MaybeLocal<v8::Object> maybe_result = fun->NewInstance(isolate->GetCurrentContext(), argc, argv);
  return maybe_result.ToLocal(out);
}

extern "C" bool Neon_Tag_IsUndefined(v8::Local<v8::Value> val) {
  return val->IsUndefined();
}

extern "C" bool Neon_Tag_IsNull(v8::Local<v8::Value> val) {
  return val->IsNull();
}

extern "C" bool Neon_Tag_IsNumber(v8::Local<v8::Value> val) {
  return val->IsNumber();
}

extern "C" bool Neon_Tag_IsBoolean(v8::Local<v8::Value> val) {
  return val->IsBoolean();
}

extern "C" bool Neon_Tag_IsString(v8::Local<v8::Value> val) {
  return val->IsString();
}

extern "C" bool Neon_Tag_IsObject(v8::Local<v8::Value> val) {
  return val->IsObject();
}

extern "C" bool Neon_Tag_IsArray(v8::Local<v8::Value> val) {
  return val->IsArray();
}

extern "C" bool Neon_Tag_IsFunction(v8::Local<v8::Value> val) {
  return val->IsFunction();
}

extern "C" bool Neon_Tag_IsError(v8::Local<v8::Value> val) {
  return val->IsNativeError();
}

extern "C" void Neon_Error_Throw(v8::Local<v8::Value> val) {
  Nan::ThrowError(val);
}

extern "C" void Neon_Error_NewError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg) {
  *out = v8::Exception::Error(msg);
}

extern "C" void Neon_Error_NewTypeError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg) {
  *out = v8::Exception::TypeError(msg);
}

extern "C" void Neon_Error_NewRangeError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg) {
  *out = v8::Exception::RangeError(msg);
}

extern "C" void Neon_Error_ThrowErrorFromUtf8(const uint8_t *data, int32_t len) {
  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  Nan::MaybeLocal<v8::String> maybe = v8::String::NewFromUtf8(isolate, (const char*)data, v8::NewStringType::kNormal, len);

  v8::Local<v8::String> msg;
  if (!maybe.ToLocal(&msg)) {
    Nan::ThrowError("an unknown Neon error occurred");
    return;
  }

  v8::Local<v8::Value> err = v8::Exception::Error(msg);
  Nan::ThrowError(err);
}

extern "C" bool Neon_Mem_SameHandle(v8::Local<v8::Value> v1, v8::Local<v8::Value> v2) {
  return v1 == v2;
}

extern "C" void Neon_Task_Schedule(void *task, Neon_TaskPerformCallback perform, Neon_TaskCompleteCallback complete, v8::Local<v8::Function> callback) {
  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  neon::Task *internal_task = new neon::Task(isolate, task, perform, complete, callback);
  neon::queue_task(internal_task);
}
