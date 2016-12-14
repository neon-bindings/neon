#include <new>
#include <nan.h>
#include <stdint.h>
#include <stdio.h>
#include "node.h"
#include "neon.h"
#include "neon_string.h"
#include "neon_class_metadata.h"

extern "C" void NeonSys_Call_SetReturn(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> value) {
  info->GetReturnValue().Set(value);
}

extern "C" void *NeonSys_Call_GetIsolate(v8::FunctionCallbackInfo<v8::Value> *info) {
  return (void *)info->GetIsolate();
}

extern "C" void *NeonSys_Call_CurrentIsolate() {
  return (void *)v8::Isolate::GetCurrent();
}

extern "C" bool NeonSys_Call_IsConstruct(v8::FunctionCallbackInfo<v8::Value> *info) {
  return info->IsConstructCall();
}

extern "C" void NeonSys_Call_This(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Object> *out) {
  *out = info->This();
}

extern "C" void NeonSys_Call_Callee(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Function> *out) {
  *out = info->Callee();
}

extern "C" void NeonSys_Call_Data(v8::FunctionCallbackInfo<v8::Value> *info, v8::Local<v8::Value> *out) {
  /*
  printf("Call_Data: v8 info  = %p\n", *(void **)info);
  dump((void *)info, 3);
  printf("Call_Data: v8 info implicit:\n");
  dump_implicit((void *)info);
  */
  *out = info->Data();
}

extern "C" int32_t NeonSys_Call_Length(v8::FunctionCallbackInfo<v8::Value> *info) {
  return info->Length();
}

extern "C" void NeonSys_Call_Get(v8::FunctionCallbackInfo<v8::Value> *info, int32_t i, v8::Local<v8::Value> *out) {
  *out = (*info)[i];
}

extern "C" void NeonSys_Object_New(v8::Local<v8::Object> *out) {
  *out = Nan::New<v8::Object>();
}

extern "C" bool NeonSys_Object_GetOwnPropertyNames(v8::Local<v8::Array> *out, v8::Local<v8::Object> obj) {
  Nan::MaybeLocal<v8::Array> maybe = Nan::GetOwnPropertyNames(obj);
  return maybe.ToLocal(out);
}

extern "C" void *NeonSys_Object_GetIsolate(v8::Local<v8::Object> obj) {
  return obj->GetIsolate();
}

extern "C" void NeonSys_Primitive_Undefined(v8::Local<v8::Primitive> *out) {
  *out = Nan::Undefined();
}

extern "C" void NeonSys_Primitive_Null(v8::Local<v8::Primitive> *out) {
  *out = Nan::Null();
}

extern "C" void NeonSys_Primitive_Boolean(v8::Local<v8::Boolean> *out, bool b) {
  *out = b ? Nan::True() : Nan::False();
}

extern "C" bool NeonSys_Primitive_BooleanValue(v8::Local<v8::Boolean> p) {
  return p->Value();
}

extern "C" void NeonSys_Primitive_Integer(v8::Local<v8::Integer> *out, v8::Isolate *isolate, int32_t x) {
  *out = v8::Integer::New(isolate, x);
}

extern "C" int64_t NeonSys_Primitive_IntegerValue(v8::Local<v8::Integer> i) {
  return i->Value();
}

extern "C" void NeonSys_Primitive_Number(v8::Local<v8::Number> *out, v8::Isolate *isolate, double value) {
  *out = v8::Number::New(isolate, value);
}

extern "C" double NeonSys_Primitive_NumberValue(v8::Local<v8::Number> n) {
  return n->Value();
}

extern "C" bool NeonSys_Primitive_IsUint32(v8::Local<v8::Primitive> p) {
  return p->IsUint32();
}

extern "C" bool NeonSys_Primitive_IsInt32(v8::Local<v8::Primitive> p) {
  return p->IsInt32();
}

extern "C" bool NeonSys_Object_Get_Index(v8::Local<v8::Value> *out, v8::Local<v8::Object> obj, uint32_t index) {
  Nan::MaybeLocal<v8::Value> maybe = Nan::Get(obj, index);
  return maybe.ToLocal(out);
}

extern "C" bool NeonSys_Object_Set_Index(bool *out, v8::Local<v8::Object> object, uint32_t index, v8::Local<v8::Value> val) {
  Nan::Maybe<bool> maybe = Nan::Set(object, index, val);
  return maybe.IsJust() && (*out = maybe.FromJust(), true);
}

bool NeonSys_ASCII_Key(v8::Local<v8::String> *key, const uint8_t *data, int32_t len) {
  Nan::MaybeLocal<v8::String> maybe_key = v8::String::NewFromUtf8(v8::Isolate::GetCurrent(), (const char*)data, v8::NewStringType::kNormal, len);
  return maybe_key.ToLocal(key);
}

extern "C" bool NeonSys_Object_Get_String(v8::Local<v8::Value> *out, v8::Local<v8::Object> obj, const uint8_t *data, int32_t len) {
  Nan::EscapableHandleScope scope;
  v8::Local<v8::String> key;
  if (!NeonSys_ASCII_Key(&key, data, len)) {
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

extern "C" bool NeonSys_Object_Set_String(bool *out, v8::Local<v8::Object> obj, const uint8_t *data, int32_t len, v8::Local<v8::Value> val) {
  Nan::HandleScope scope;
  v8::Local<v8::String> key;
  if (!NeonSys_ASCII_Key(&key, data, len)) {
    return false;
  }
  Nan::Maybe<bool> maybe = Nan::Set(obj, key, val);
  return maybe.IsJust() && (*out = maybe.FromJust(), true);
}

extern "C" bool NeonSys_Object_Get(v8::Local<v8::Value> *out, v8::Local<v8::Object> obj, v8::Local<v8::Value> key) {
  Nan::MaybeLocal<v8::Value> maybe = Nan::Get(obj, key);
  return maybe.ToLocal(out);
}

extern "C" bool NeonSys_Object_Set(bool *out, v8::Local<v8::Object> obj, v8::Local<v8::Value> key, v8::Local<v8::Value> val) {
  Nan::Maybe<bool> maybe = Nan::Set(obj, key, val);
  if (maybe.IsJust()) {
    *out = maybe.FromJust();
    return true;
  }
  return false;
}

extern "C" void NeonSys_Array_New(v8::Local<v8::Array> *out, v8::Isolate *isolate, uint32_t length) {
  *out = v8::Array::New(isolate, length);
}

extern "C" uint32_t NeonSys_Array_Length(v8::Local<v8::Array> array) {
  return array->Length();
}

extern "C" bool NeonSys_String_New(v8::Local<v8::String> *out, v8::Isolate *isolate, const uint8_t *data, int32_t len) {
  Nan::MaybeLocal<v8::String> maybe = v8::String::NewFromUtf8(isolate, (const char*)data, v8::NewStringType::kNormal, len);
  return maybe.ToLocal(out);
}

extern "C" int32_t NeonSys_String_Utf8Length(v8::Local<v8::String> str) {
  return str->Utf8Length();
}

extern "C" size_t NeonSys_String_Data(char *out, size_t len, v8::Local<v8::Value> str) {
  return Nan::DecodeWrite(out, len, str, Nan::UTF8);
}

extern "C" bool NeonSys_Convert_ToString(v8::Local<v8::String> *out, v8::Local<v8::Value> value) {
  Nan::MaybeLocal<v8::String> maybe = Nan::To<v8::String>(value);
  return maybe.ToLocal(out);
}

extern "C" bool NeonSys_Convert_ToObject(v8::Local<v8::Object> *out, v8::Local<v8::Value> *value) {
  Nan::MaybeLocal<v8::Object> maybe = Nan::To<v8::Object>(*value);
  return maybe.ToLocal(out);
}

extern "C" bool NeonSys_Buffer_New(v8::Local<v8::Object> *out, uint32_t size) {
  Nan::MaybeLocal<v8::Object> maybe = Nan::NewBuffer(size);
  return maybe.ToLocal(out);
}

extern "C" void NeonSys_Buffer_Data(buf_t *out, v8::Local<v8::Object> obj) {
  out->data = node::Buffer::Data(obj);
  out->len = node::Buffer::Length(obj);
}

extern "C" bool NeonSys_Tag_IsBuffer(v8::Local<v8::Value> obj) {
  return node::Buffer::HasInstance(obj);
}

extern "C" void NeonSys_Scope_Escape(v8::Local<v8::Value> *out, Nan::EscapableHandleScope *scope, v8::Local<v8::Value> value) {
  *out = scope->Escape(value);
}

extern "C" void NeonSys_Scope_Chained(void *out, void *closure, NeonSys_ChainedScopeCallback callback, void *parent_scope) {
  Nan::EscapableHandleScope v8_scope;
  callback(out, parent_scope, &v8_scope, closure);
}

extern "C" void NeonSys_Scope_Nested(void *out, void *closure, NeonSys_NestedScopeCallback callback, void *realm) {
  Nan::HandleScope v8_scope;
  callback(out, realm, closure);
}

extern "C" void NeonSys_Scope_Enter(v8::HandleScope *scope, v8::Isolate *isolate) {
  void *p = scope;
  ::new (p) v8::HandleScope(isolate);
}

extern "C" void NeonSys_Scope_Exit(v8::HandleScope *scope) {
  scope->~HandleScope();
}

extern "C" size_t NeonSys_Scope_Sizeof() {
  return sizeof(v8::HandleScope);
}

extern "C" size_t NeonSys_Scope_Alignof() {
  return alignof(v8::HandleScope);
}

extern "C" size_t NeonSys_Scope_SizeofEscapable() {
  return sizeof(v8::EscapableHandleScope);
}

extern "C" size_t NeonSys_Scope_AlignofEscapable() {
  return alignof(v8::EscapableHandleScope);
}

extern "C" void NeonSys_Fun_ExecKernel(void *kernel, NeonSys_RootScopeCallback callback, v8::FunctionCallbackInfo<v8::Value> *info, void *scope) {
  Nan::HandleScope v8_scope;
  callback(info, kernel, scope);
}

extern "C" void NeonSys_Module_ExecKernel(void *kernel, NeonSys_ModuleScopeCallback callback, v8::Local<v8::Object> exports, void *scope) {
  Nan::HandleScope v8_scope;
  callback(kernel, exports, scope);
}

extern "C" void NeonSys_Class_ConstructBaseCallback(const v8::FunctionCallbackInfo<v8::Value>& info) {
  Nan::HandleScope scope;
  v8::Local<v8::External> wrapper = v8::Local<v8::External>::Cast(info.Data());
  neon::BaseClassMetadata *metadata = static_cast<neon::BaseClassMetadata *>(wrapper->Value());
  if (info.IsConstructCall()) {
    metadata->construct(info);
  } else {
    metadata->call(info);
  }
}

extern "C" void *NeonSys_Class_CreateBase(v8::Isolate *isolate,
                                          NeonSys_AllocateCallback allocate_callback,
                                          void *allocate_kernel,
                                          NeonSys_ConstructCallback construct_callback,
                                          void *construct_kernel,
                                          v8::FunctionCallback call_callback,
                                          void *call_kernel,
                                          NeonSys_DropCallback drop)
{
  neon::BaseClassMetadata *metadata = new neon::BaseClassMetadata(construct_callback, construct_kernel, call_callback, call_kernel, allocate_callback, allocate_kernel, drop);
  v8::Local<v8::External> data = v8::External::New(isolate, metadata);
  v8::Local<v8::FunctionTemplate> constructor_template = v8::FunctionTemplate::New(isolate, NeonSys_Class_ConstructBaseCallback, data);
  metadata->SetTemplate(isolate, constructor_template);
  v8::Local<v8::ObjectTemplate> instance_template = constructor_template->InstanceTemplate();
  instance_template->SetInternalFieldCount(1); // index 0: an aligned, owned pointer to the internals (a user-defined Rust data structure)
  return metadata;
}

extern "C" void *NeonSys_Class_GetClassMap(v8::Isolate *isolate) {
  neon::ClassMapHolder *holder = static_cast<neon::ClassMapHolder *>(isolate->GetData(NEON_ISOLATE_SLOT));
  return (holder == nullptr)
       ? nullptr
       : holder->GetMap();
}

void cleanup_class_map(void *arg) {
  neon::ClassMapHolder *holder = static_cast<neon::ClassMapHolder *>(arg);
  delete holder;
}

extern "C" void NeonSys_Class_SetClassMap(v8::Isolate *isolate, void *map, NeonSys_DropCallback drop_map) {
  neon::ClassMapHolder *holder = new neon::ClassMapHolder(map, drop_map);
  isolate->SetData(NEON_ISOLATE_SLOT, holder);
  // ISSUE(#77): When workers land in node, this will need to be generalized to a per-worker version.
  node::AtExit(cleanup_class_map, holder);
}

extern "C" void *NeonSys_Class_GetCallKernel(v8::Local<v8::External> wrapper) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(wrapper->Value());
  return metadata->GetCallKernel();
}

extern "C" void *NeonSys_Class_GetConstructKernel(v8::Local<v8::External> wrapper) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(wrapper->Value());
  return metadata->GetConstructKernel();
}

extern "C" void *NeonSys_Class_GetAllocateKernel(v8::Local<v8::External> wrapper) {
  neon::BaseClassMetadata *metadata = static_cast<neon::BaseClassMetadata *>(wrapper->Value());
  return metadata->GetAllocateKernel();
}

extern "C" bool NeonSys_Class_Constructor(v8::Local<v8::Function> *out, v8::Local<v8::FunctionTemplate> ft) {
  v8::MaybeLocal<v8::Function> maybe = ft->GetFunction();
  return maybe.ToLocal(out);
}

extern "C" bool NeonSys_Class_Check(v8::Local<v8::FunctionTemplate> ft, v8::Local<v8::Value> v) {
  return ft->HasInstance(v);
}

extern "C" bool NeonSys_Class_HasInstance(void *metadata_pointer, v8::Local<v8::Value> v) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  return metadata->GetTemplate(v8::Isolate::GetCurrent())->HasInstance(v);
}

extern "C" bool NeonSys_Class_SetName(v8::Isolate *isolate, void *metadata_pointer, const char *name, uint32_t byte_length) {
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

extern "C" void NeonSys_Class_ThrowCallError(v8::Isolate *isolate, void *metadata_pointer) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  Nan::ThrowTypeError(metadata->GetCallError().ToJsString(isolate, "constructor called without new."));
}


extern "C" void NeonSys_Class_ThrowThisError(v8::Isolate *isolate, void *metadata_pointer) {
  neon::ClassMetadata *metadata = static_cast<neon::ClassMetadata *>(metadata_pointer);
  Nan::ThrowTypeError(metadata->GetThisError().ToJsString(isolate, "this is not an object of the expected type."));
}

extern "C" bool NeonSys_Class_AddMethod(v8::Isolate *isolate, void *metadata_pointer, const char *name, uint32_t byte_length, v8::Local<v8::Function> method) {
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

extern "C" void NeonSys_Class_MetadataToClass(v8::Local<v8::FunctionTemplate> *out, v8::Isolate *isolate, void *metadata) {
  *out = static_cast<neon::ClassMetadata *>(metadata)->GetTemplate(isolate);
}

extern "C" void *NeonSys_Class_GetInstanceInternals(v8::Local<v8::Object> obj) {
  return static_cast<neon::BaseClassInstanceMetadata *>(obj->GetAlignedPointerFromInternalField(0))->GetInternals();
}


extern "C" bool NeonSys_Fun_New(v8::Local<v8::Function> *out, v8::Isolate *isolate, v8::FunctionCallback callback, void *kernel) {
  v8::Local<v8::External> wrapper = v8::External::New(isolate, kernel);
  if (wrapper.IsEmpty()) {
    return false;
  }

  v8::MaybeLocal<v8::Function> maybe_result = v8::Function::New(isolate->GetCurrentContext(), callback, wrapper);
  return maybe_result.ToLocal(out);
}

extern "C" void *NeonSys_Fun_GetKernel(v8::Local<v8::External> data) {
  return data->Value();
}

extern "C" bool NeonSys_Fun_Call(v8::Local<v8::Value> *out, v8::Isolate *isolate, v8::Local<v8::Function> fun, v8::Local<v8::Value> self, int32_t argc, v8::Local<v8::Value> argv[]) {
  v8::MaybeLocal<v8::Value> maybe_result = fun->Call(isolate->GetCurrentContext(), self, argc, argv);
  return maybe_result.ToLocal(out);
}

extern "C" bool NeonSys_Fun_Construct(v8::Local<v8::Object> *out, v8::Isolate *isolate, v8::Local<v8::Function> fun, int32_t argc, v8::Local<v8::Value> argv[]) {
  v8::MaybeLocal<v8::Object> maybe_result = fun->NewInstance(isolate->GetCurrentContext(), argc, argv);
  return maybe_result.ToLocal(out);
}

extern "C" tag_t NeonSys_Tag_Of(v8::Local<v8::Value> val) {
  return val->IsNull()                    ? tag_null
    : val->IsUndefined()                  ? tag_undefined
    : (val->IsTrue() || val->IsFalse())   ? tag_boolean
    // ISSUE(#78): kill this
    : (val->IsInt32() || val->IsUint32()) ? tag_integer
    : val->IsNumber()                     ? tag_number
    : val->IsString()                     ? tag_string
    : val->IsArray()                      ? tag_array
    : val->IsFunction()                   ? tag_function
    : val->IsObject()                     ? tag_object
                                          : tag_other;
}

extern "C" bool NeonSys_Tag_IsUndefined(v8::Local<v8::Value> val) {
  return val->IsUndefined();
}

extern "C" bool NeonSys_Tag_IsNull(v8::Local<v8::Value> val) {
  return val->IsNull();
}

extern "C" bool NeonSys_Tag_IsInteger(v8::Local<v8::Value> val) {
  return val->IsInt32() || val->IsUint32();
}

extern "C" bool NeonSys_Tag_IsNumber(v8::Local<v8::Value> val) {
  return val->IsNumber();
}

extern "C" bool NeonSys_Tag_IsBoolean(v8::Local<v8::Value> val) {
  return val->IsBoolean();
}

extern "C" bool NeonSys_Tag_IsString(v8::Local<v8::Value> val) {
  return val->IsString();
}

extern "C" bool NeonSys_Tag_IsObject(v8::Local<v8::Value> val) {
  return val->IsObject();
}

extern "C" bool NeonSys_Tag_IsArray(v8::Local<v8::Value> val) {
  return val->IsArray();
}

extern "C" bool NeonSys_Tag_IsFunction(v8::Local<v8::Value> val) {
  return val->IsFunction();
}

extern "C" bool NeonSys_Tag_IsError(v8::Local<v8::Value> val) {
  return val->IsNativeError();
}

extern "C" void NeonSys_Error_Throw(v8::Local<v8::Value> val) {
  Nan::ThrowError(val);
}

extern "C" void NeonSys_Error_NewError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg) {
  *out = v8::Exception::Error(msg);
}

extern "C" void NeonSys_Error_NewTypeError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg) {
  *out = v8::Exception::TypeError(msg);
}

extern "C" void NeonSys_Error_NewReferenceError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg) {
  *out = v8::Exception::ReferenceError(msg);
}

extern "C" void NeonSys_Error_NewRangeError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg) {
  *out = v8::Exception::RangeError(msg);
}

extern "C" void NeonSys_Error_NewSyntaxError(v8::Local<v8::Value> *out, v8::Local<v8::String> msg) {
  *out = v8::Exception::SyntaxError(msg);
}

extern "C" void NeonSys_Error_ThrowErrorFromCString(const char *msg) {
  Nan::ThrowError(msg);
}

extern "C" void NeonSys_Error_ThrowTypeErrorFromCString(const char *msg) {
  Nan::ThrowTypeError(msg);
}

extern "C" void NeonSys_Error_ThrowReferenceErrorFromCString(const char *msg) {
  Nan::ThrowReferenceError(msg);
}

extern "C" void NeonSys_Error_ThrowRangeErrorFromCString(const char *msg) {
  Nan::ThrowRangeError(msg);
}

extern "C" void NeonSys_Error_ThrowSyntaxErrorFromCString(const char *msg) {
  Nan::ThrowSyntaxError(msg);
}

extern "C" bool NeonSys_Mem_SameHandle(v8::Local<v8::Value> v1, v8::Local<v8::Value> v2) {
  return v1 == v2;
}
