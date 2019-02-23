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

extern "C" void Neon_Call_SetReturn(v8::FunctionCallbackInfo<v8::Value> *info, v8::Persistent<v8::Value> *value) {
  info->GetReturnValue().Set(*value);
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

extern "C" void Neon_Call_This(v8::FunctionCallbackInfo<v8::Value> *info, v8::Persistent<v8::Value> *out, v8::Isolate *isolate) {
  Nan::HandleScope scope;
  out->Reset(isolate, info->This());
}

extern "C" void Neon_Call_Data(v8::FunctionCallbackInfo<v8::Value> *info, v8::Persistent<v8::Value> *out, v8::Isolate *isolate) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> data = info->Data();
  out->Reset(isolate, data);
}

extern "C" int32_t Neon_Call_Length(v8::FunctionCallbackInfo<v8::Value> *info) {
  return info->Length();
}

extern "C" void Neon_Call_Get(v8::FunctionCallbackInfo<v8::Value> *info, v8::Isolate *isolate, int32_t i, v8::Persistent<v8::Value> *out) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = (*info)[i];
  out->Reset(isolate, local);
}

extern "C" void Neon_Object_New(v8::Persistent<v8::Value> *out, v8::Isolate *isolate) {
  Nan::HandleScope scope;
  v8::Local<v8::Object> local = Nan::New<v8::Object>();
  out->Reset(isolate, local);
}

extern "C" bool Neon_Object_GetOwnPropertyNames(v8::Persistent<v8::Array> *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *obj) {
  Nan::HandleScope scope;
  v8::Local<v8::Object> lobj = Nan::New(*obj);
  v8::Local<v8::Array> larr;
  Nan::MaybeLocal<v8::Array> maybe = Nan::GetOwnPropertyNames(lobj);
  if (!maybe.ToLocal(&larr)) {
    return false;
  }
  out->Reset(isolate, larr);
  return true;
}

extern "C" void Neon_Primitive_Undefined(v8::Persistent<v8::Value> *out, v8::Isolate *isolate) {
  Nan::HandleScope scope;
  out->Reset(isolate, Nan::Undefined());
}

extern "C" void Neon_Primitive_Null(v8::Persistent<v8::Value> *out, v8::Isolate *isolate) {
  Nan::HandleScope scope;
  out->Reset(isolate, Nan::Null());
}

extern "C" void Neon_Primitive_Boolean(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, bool b) {
  Nan::HandleScope scope;
  out->Reset(isolate, b ? Nan::True() : Nan::False());
}

extern "C" bool Neon_Primitive_BooleanValue(v8::Persistent<v8::Boolean> *p) {
  Nan::HandleScope scope;
  v8::Local<v8::Boolean> local = Nan::New(*p);
  return local->Value();
}

extern "C" double Neon_Primitive_NumberValue(v8::Persistent<v8::Number> *n) {
  Nan::HandleScope scope;
  v8::Local<v8::Number> local = Nan::New(*n);
  return local->Value();
}

extern "C" void Neon_Primitive_Number(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, double value) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> n = v8::Number::New(isolate, value);
  out->Reset(isolate, n);
}

extern "C" bool Neon_Primitive_IsUint32(v8::Local<v8::Primitive> p) {
  return p->IsUint32();
}

extern "C" bool Neon_Primitive_IsInt32(v8::Local<v8::Primitive> p) {
  return p->IsInt32();
}

extern "C" bool Neon_Object_Get_Index(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *obj, uint32_t index) {
  Nan::HandleScope scope;
  v8::Local<v8::Object> lobj = Nan::New(*obj);
  Nan::MaybeLocal<v8::Value> maybe = Nan::Get(lobj, index);
  v8::Local<v8::Value> local;
  if (!maybe.ToLocal(&local)) {
    return false;
  }
  out->Reset(isolate, local);
  return true;
}

extern "C" bool Neon_Object_Set_Index(bool *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *object, uint32_t index, v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Object> lobject = Nan::New(*object);
  v8::Local<v8::Value> lval = Nan::New(*val);
  Nan::Maybe<bool> maybe = Nan::Set(lobject, index, lval);
  return maybe.IsJust() && (*out = maybe.FromJust(), true);
}

bool Neon_ASCII_Key(v8::Local<v8::String> *key, const uint8_t *data, int32_t len) {
  Nan::MaybeLocal<v8::String> maybe_key = v8::String::NewFromUtf8(v8::Isolate::GetCurrent(), (const char*)data, v8::NewStringType::kNormal, len);
  return maybe_key.ToLocal(key);
}

extern "C" bool Neon_Object_Get_String(v8::Persistent<v8::Value> *p_out, v8::Isolate *isolate, v8::Persistent<v8::Object> *p_obj, const uint8_t *data, int32_t len) {
  Nan::HandleScope scope;
  v8::Local<v8::String> key;
  if (!Neon_ASCII_Key(&key, data, len)) {
    return false;
  }
  v8::Local<v8::Object> obj = Nan::New(*p_obj);
  Nan::MaybeLocal<v8::Value> maybe = Nan::Get(obj, key);
  v8::Local<v8::Value> result;
  if (!maybe.ToLocal(&result)) {
    return false;
  }
  p_out->Reset(isolate, result);
  return true;
}

extern "C" bool Neon_Object_Set_String(bool *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *p_obj, const uint8_t *data, int32_t len, v8::Persistent<v8::Value> *p_val) {
  Nan::HandleScope scope;
  v8::Local<v8::String> key;
  if (!Neon_ASCII_Key(&key, data, len)) {
    return false;
  }
  v8::Local<v8::Object> obj = Nan::New(*p_obj);
  v8::Local<v8::Value> val = Nan::New(*p_val);
  Nan::Maybe<bool> maybe = Nan::Set(obj, key, val);
  return maybe.IsJust() && (*out = maybe.FromJust(), true);
}

extern "C" bool Neon_Object_Get(v8::Persistent<v8::Value> *p_out, v8::Isolate *isolate, v8::Persistent<v8::Object> *p_obj, v8::Persistent<v8::Value> *p_key) {
  Nan::HandleScope scope;
  v8::Local<v8::Object> object = Nan::New(*p_obj);
  v8::Local<v8::Value> key = Nan::New(*p_key);
  Nan::MaybeLocal<v8::Value> maybe = Nan::Get(object, key);
  v8::Local<v8::Value> value;
  if (!maybe.ToLocal(&value)) {
    return false;
  }
  p_out->Reset(isolate, value);
  return true;
}

extern "C" bool Neon_Object_Set(bool *out, v8::Isolate *isolate, v8::Persistent<v8::Object> *p_obj, v8::Persistent<v8::Value> *p_key, v8::Persistent<v8::Value> *p_val) {
  Nan::HandleScope scope;
  v8::Local<v8::Object> object = Nan::New(*p_obj);
  v8::Local<v8::Value> key = Nan::New(*p_key);
  v8::Local<v8::Value> value = Nan::New(*p_val);
  Nan::Maybe<bool> maybe = Nan::Set(object, key, value);
  if (maybe.IsJust()) {
    *out = maybe.FromJust();
    return true;
  }
  return false;
}

extern "C" void Neon_Array_New(v8::Persistent<v8::Array> *out, v8::Isolate *isolate, uint32_t length) {
  Nan::HandleScope scope;
  v8::Local<v8::Array> local = v8::Array::New(isolate, length);
  out->Reset(isolate, local);
}

extern "C" uint32_t Neon_Array_Length(v8::Persistent<v8::Array> *array) {
  Nan::HandleScope scope;
  v8::Local<v8::Array> local = Nan::New(*array);
  return local->Length();
}

extern "C" bool Neon_String_New(v8::Persistent<v8::String> *out, v8::Isolate *isolate, const uint8_t *data, int32_t len) {
  Nan::HandleScope scope;
  Nan::MaybeLocal<v8::String> maybe = v8::String::NewFromUtf8(isolate, (const char*)data, v8::NewStringType::kNormal, len);
  v8::Local<v8::String> local;
  if (!maybe.ToLocal(&local)) {
    return false;
  }
  out->Reset(isolate, local);
  return true;
}

extern "C" int32_t Neon_String_Utf8Length(v8::Persistent<v8::String> *str) {
  Nan::HandleScope scope;
  v8::Local<v8::String> local = Nan::New(*str);
  return local->Utf8Length();
}

extern "C" size_t Neon_String_Data(char *out, size_t len, v8::Persistent<v8::String> *str) {
  Nan::HandleScope scope;
  v8::Local<v8::String> local = Nan::New(*str);
  return Nan::DecodeWrite(out, len, local, Nan::UTF8);
}

extern "C" bool Neon_String_ToString(v8::Persistent<v8::String> *out, v8::Isolate *isolate, v8::Persistent<v8::Value> *value) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> lvalue = Nan::New(*value);
  Nan::MaybeLocal<v8::String> maybe = Nan::To<v8::String>(lvalue);
  v8::Local<v8::String> lout;
  if (!maybe.ToLocal(&lout)) {
    return false;
  }
  out->Reset(isolate, lout);
  return true;
}

extern "C" bool Neon_Buffer_New(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, uint32_t len) {
  Nan::HandleScope scope;
  Nan::MaybeLocal<v8::Object> maybe = Nan::NewBuffer(len);
  v8::Local<v8::Object> buffer;
  if (!maybe.ToLocal(&buffer)) {
    return false;
  }
  void *data = node::Buffer::Data(buffer);
  memset(data, 0, len);
  out->Reset(isolate, buffer);
  return true;
}

extern "C" bool Neon_Buffer_Uninitialized(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, uint32_t len) {
  Nan::HandleScope scope;
  Nan::MaybeLocal<v8::Object> maybe = Nan::NewBuffer(len);
  v8::Local<v8::Object> buffer;
  if (!maybe.ToLocal(&buffer)) {
    return false;
  }
  out->Reset(isolate, buffer);
  return true;
}

extern "C" void Neon_Buffer_Data(void **base_out, size_t *len_out, v8::Persistent<v8::Object> *obj) {
  Nan::HandleScope scope;
  v8::Local<v8::Object> local = Nan::New(*obj);
  *base_out = node::Buffer::Data(local);
  *len_out = node::Buffer::Length(local);
}

extern "C" bool Neon_Tag_IsBuffer(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return node::Buffer::HasInstance(local);
}

extern "C" bool Neon_ArrayBuffer_New(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, uint32_t len) {
  Nan::HandleScope scope;
  v8::Local<v8::ArrayBuffer> local = v8::ArrayBuffer::New(isolate, len);
  out->Reset(isolate, local);
  return true;
}

extern "C" void Neon_ArrayBuffer_Data(void **base_out, size_t *len_out, v8::Persistent<v8::ArrayBuffer> *buffer) {
  Nan::HandleScope scope;
  v8::Local<v8::ArrayBuffer> local = Nan::New(*buffer);
  v8::ArrayBuffer::Contents contents = local->GetContents();
  *base_out = contents.Data();
  *len_out = contents.ByteLength();
}

extern "C" bool Neon_Tag_IsArrayBuffer(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsArrayBuffer();
}

extern "C" void Neon_Scope_ClonePersistent(v8::Isolate *isolate, v8::Persistent<v8::Value> *to, v8::Persistent<v8::Value> *from) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*from);
  to->Reset(isolate, local);
}

extern "C" void Neon_Scope_Nested(void *out, void *closure, Neon_NestedScopeCallback callback, void *realm) {
  Nan::HandleScope v8_scope;
  callback(out, realm, closure);
}

extern "C" void Neon_Scope_GetGlobal(v8::Isolate *isolate, v8::Persistent<v8::Value> *out) {
  Nan::HandleScope scope;
  auto ctx = isolate->GetCurrentContext();
  out->Reset(isolate, ctx->Global());
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

extern "C" void *Neon_Class_GetCallKernel(v8::Persistent<v8::External> *wrapper) {
  Nan::HandleScope scope;
  v8::Local<v8::External> local = Nan::New(*wrapper);
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(local->Value());
  return metadata->GetCallKernel();
}

extern "C" void *Neon_Class_GetConstructKernel(v8::Persistent<v8::External> *wrapper) {
  Nan::HandleScope scope;
  v8::Local<v8::External> local = Nan::New(*wrapper);
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(local->Value());
  return metadata->GetConstructKernel();
}

extern "C" void *Neon_Class_GetAllocateKernel(v8::Persistent<v8::External> *wrapper) {
  Nan::HandleScope scope;
  v8::Local<v8::External> local = Nan::New(*wrapper);
  neon::BaseClassMetadata *metadata = static_cast<neon::BaseClassMetadata *>(local->Value());
  return metadata->GetAllocateKernel();
}

extern "C" bool Neon_Class_HasInstance(void *metadata_pointer, v8::Persistent<v8::Value> *v) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*v);
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  return metadata->GetTemplate(v8::Isolate::GetCurrent())->HasInstance(local);
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

extern "C" void Neon_Class_GetName(const char **chars_out, size_t *len_out, v8::Isolate *isolate, void *metadata_pointer) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  neon::Slice name = metadata->GetName();
  *chars_out = name.GetBuffer();
  *len_out = name.GetLength();
}

extern "C" void Neon_Class_ThrowCallError(v8::Isolate *isolate, void *metadata_pointer) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  Nan::ThrowTypeError(metadata->GetCallError().ToJsString(isolate, "constructor called without new."));
}


extern "C" void Neon_Class_ThrowThisError(v8::Isolate *isolate, void *metadata_pointer) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  Nan::ThrowTypeError(metadata->GetThisError().ToJsString(isolate, "this is not an object of the expected type."));
}

extern "C" bool Neon_Class_AddMethod(v8::Isolate *isolate, void *metadata_pointer, const char *name, uint32_t byte_length, v8::Persistent<v8::FunctionTemplate> *method) {
  Nan::HandleScope scope;
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  v8::Local<v8::FunctionTemplate> ft = metadata->GetTemplate(isolate);
  v8::Local<v8::ObjectTemplate> pt = ft->PrototypeTemplate();
  v8::MaybeLocal<v8::String> maybe_key = v8::String::NewFromUtf8(isolate, name, v8::NewStringType::kNormal, byte_length);
  v8::Local<v8::String> key;
  if (!maybe_key.ToLocal(&key)) {
    return false;
  }
  v8::Local<v8::FunctionTemplate> lmethod = Nan::New(*method);
  pt->Set(key, lmethod);
  return true;
}

extern "C" bool Neon_Class_MetadataToConstructor(v8::Persistent<v8::Function> *out, v8::Isolate *isolate, void *metadata) {
  Nan::HandleScope scope;
  v8::Local<v8::FunctionTemplate> ft = static_cast<neon::ClassMetadata *>(metadata)->GetTemplate(isolate);
  v8::MaybeLocal<v8::Function> maybe = ft->GetFunction();
  v8::Local<v8::Function> local;
  if (!maybe.ToLocal(&local)) {
    return false;
  }
  out->Reset(isolate, local);
  return true;
}

extern "C" void *Neon_Class_GetInstanceInternals(v8::Persistent<v8::Object> *obj) {
  Nan::HandleScope scope;
  v8::Local<v8::Object> local = Nan::New(*obj);
  return static_cast<neon::BaseClassInstanceMetadata *>(local->GetAlignedPointerFromInternalField(0))->GetInternals();
}

extern "C" bool Neon_Fun_Template_New(v8::Persistent<v8::FunctionTemplate> *out, v8::Isolate *isolate, callback_t callback) {
  Nan::HandleScope scope;
  v8::Local<v8::External> wrapper = v8::External::New(isolate, callback.dynamic_callback);
  if (wrapper.IsEmpty()) {
    return false;
  }

  v8::FunctionCallback static_callback = reinterpret_cast<v8::FunctionCallback>(callback.static_callback);
  v8::MaybeLocal<v8::FunctionTemplate> maybe_result = v8::FunctionTemplate::New(isolate, static_callback, wrapper);
  v8::Local<v8::FunctionTemplate> local;
  if (!maybe_result.ToLocal(&local)) {
    return false;
  }
  out->Reset(isolate, local);
  return true;
}

extern "C" bool Neon_Fun_New(v8::Persistent<v8::Function> *out, v8::Isolate *isolate, callback_t callback) {
  Nan::HandleScope scope;
  v8::Local<v8::External> wrapper = v8::External::New(isolate, callback.dynamic_callback);
  if (wrapper.IsEmpty()) {
    return false;
  }

  v8::FunctionCallback static_callback = reinterpret_cast<v8::FunctionCallback>(callback.static_callback);
  v8::MaybeLocal<v8::Function> maybe_result = v8::Function::New(isolate->GetCurrentContext(), static_callback, wrapper);
  v8::Local<v8::Function> f;
  if (!maybe_result.ToLocal(&f)) {
    return false;
  }
  out->Reset(isolate, f);
  return true;
}

extern "C" void *Neon_Fun_GetDynamicCallback(v8::Persistent<v8::External> *data) {
  Nan::HandleScope scope;
  v8::Local<v8::External> local = Nan::New(*data);
  return local->Value();
}

extern "C" bool Neon_Fun_Call(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::Function> *fun, v8::Persistent<v8::Value> *self, int32_t argc, v8::Persistent<v8::Value> *argv[]) {
  Nan::HandleScope scope;
  v8::Local<v8::Function> lfun = Nan::New(*fun);
  v8::Local<v8::Value> lself = Nan::New(*self);
  v8::Local<v8::Value> *largv = new v8::Local<v8::Value>[argc];
  for (int32_t i = 0; i < argc; i++) {
    largv[i] = Nan::New(*argv[i]);
  }
  v8::MaybeLocal<v8::Value> maybe_result = lfun->Call(isolate->GetCurrentContext(), lself, argc, largv);
  delete[] largv;
  v8::Local<v8::Value> lout;
  if (!maybe_result.ToLocal(&lout)) {
    return false;
  }

  out->Reset(isolate, lout);
  return true;
}

extern "C" bool Neon_Fun_Construct(v8::Persistent<v8::Object> *out, v8::Isolate *isolate, v8::Persistent<v8::Function> *fun, int32_t argc, v8::Persistent<v8::Value> *argv[]) {
  Nan::HandleScope scope;
  v8::Local<v8::Function> lfun = Nan::New(*fun);
  v8::Local<v8::Value> *largv = new v8::Local<v8::Value>[argc];
  for (int32_t i = 0; i < argc; i++) {
    largv[i] = Nan::New(*argv[i]);
  }
  v8::MaybeLocal<v8::Object> maybe_result = lfun->NewInstance(isolate->GetCurrentContext(), argc, largv);
  delete[] largv;
  v8::Local<v8::Object> lout;
  if (!maybe_result.ToLocal(&lout)) {
    return false;
  }

  out->Reset(isolate, lout);
  return true;
}

extern "C" bool Neon_Tag_IsUndefined(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsUndefined();
}

extern "C" bool Neon_Tag_IsNull(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsNull();
}

extern "C" bool Neon_Tag_IsNumber(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsNumber();
}

extern "C" bool Neon_Tag_IsBoolean(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsBoolean();
}

extern "C" bool Neon_Tag_IsString(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsString();
}

extern "C" bool Neon_Tag_IsObject(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsObject();
}

extern "C" bool Neon_Tag_IsArray(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsArray();
}

extern "C" bool Neon_Tag_IsFunction(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsFunction();
}

extern "C" bool Neon_Tag_IsError(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  return local->IsNativeError();
}

extern "C" void Neon_Error_Throw(v8::Persistent<v8::Value> *val) {
  Nan::HandleScope scope;
  v8::Local<v8::Value> local = Nan::New(*val);
  Nan::ThrowError(local);
}

extern "C" void Neon_Error_NewError(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::String> *msg) {
  Nan::HandleScope scope;
  v8::Local<v8::String> lmsg = Nan::New(*msg);
  v8::Local<v8::Value> lerr = v8::Exception::Error(lmsg);
  out->Reset(isolate, lerr);
}

extern "C" void Neon_Error_NewTypeError(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::String> *msg) {
  Nan::HandleScope scope;
  v8::Local<v8::String> lmsg = Nan::New(*msg);
  v8::Local<v8::Value> lerr = v8::Exception::TypeError(lmsg);
  out->Reset(isolate, lerr);
}

extern "C" void Neon_Error_NewRangeError(v8::Persistent<v8::Value> *out, v8::Isolate *isolate, v8::Persistent<v8::String> *msg) {
  Nan::HandleScope scope;
  v8::Local<v8::String> lmsg = Nan::New(*msg);
  v8::Local<v8::Value> lerr = v8::Exception::RangeError(lmsg);
  out->Reset(isolate, lerr);
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

extern "C" void Neon_Mem_NewPersistent(v8::Persistent<v8::Value> *out) {
  ::new (out) v8::Persistent<v8::Value>();
}

extern "C" void Neon_Mem_DropPersistent(v8::Persistent<v8::Value> *p) {
  // FIXME: can we change the traits of the persistent to Reset automatically in the destructor?
  p->Reset();
  p->Persistent::~Persistent();
}

extern "C" void Neon_Mem_ReadPersistent(v8::Local<v8::Value> *out, v8::Persistent<v8::Value> *p) {
  *out = Nan::New(*p);
}

extern "C" void Neon_Mem_ResetPersistent(v8::Persistent<v8::Value> *p, v8::Local<v8::Value> h) {
  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  p->Reset(isolate, h);
}

extern "C" void Neon_Task_Schedule(void *task, Neon_TaskPerformCallback perform, Neon_TaskCompleteCallback complete, v8::Persistent<v8::Function> *callback) {
  v8::Isolate *isolate = v8::Isolate::GetCurrent();
  neon::Task *internal_task = new neon::Task(isolate, task, perform, complete, callback);
  neon::queue_task(internal_task);
}
