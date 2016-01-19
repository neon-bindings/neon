#ifndef NEON_CLASS_METADATA_H_
#define NEON_CLASS_METADATA_H_

#include <stdint.h>
#include "v8.h"

// FIXME: error propagation
typedef void *(*NeonSys_CreateInternalsCallback)(int32_t argc, v8::Local<v8::Value> argv[]);

// FIXME: is bool the right way to do error propagation?
typedef bool(*NeonSys_ConstructorKernelCallback)(v8::Local<v8::Object> self, int32_t argc, v8::Local<v8::Value> argv[]);


class NeonClassConstructorMetadata {
public:

  NeonClassConstructorMetadata(NeonSys_ConstructorKernelCallback kernel) {
    is_allocating_ = false;
    kernel_ = kernel;
  }

  void SetTemplate(v8::Isolate *isolate, v8::Local<v8::FunctionTemplate> t) {
    template_.Reset(isolate, t);
    template_.SetWeak(this, Finalize, v8::WeakCallbackType::kParameter);
  }

  v8::Local<v8::FunctionTemplate> GetTemplate(v8::Isolate *isolate) {
    return v8::Local<v8::FunctionTemplate>::New(isolate, template_);
  }

  // FIXME: is bool the right way to propagate errors?
  virtual bool construct(v8::Local<v8::Object> self, int argc, v8::Local<v8::Value> argv[]) = 0;

  NeonSys_ConstructorKernelCallback GetKernel() {
    return kernel_;
  }

  // FIXME: return MaybeLocal
  v8::Local<v8::Object> allocate(v8::Isolate *isolate) {
    v8::Local<v8::Context> cx = isolate->GetCurrentContext();
    v8::Local<v8::Function> f = GetTemplate(isolate)->GetFunction();
    is_allocating_ = true;
    // FIXME: error propagation
    v8::Local<v8::Object> instance = f->NewInstance(cx).ToLocalChecked();
    is_allocating_ = false;
    return instance;
  }

protected:

  virtual ~NeonClassConstructorMetadata() {
    template_.Reset();
  }

  bool is_allocating_;

  NeonSys_ConstructorKernelCallback kernel_;

private:

  static void Finalize(const v8::WeakCallbackInfo<NeonClassConstructorMetadata>& data) {
    NeonClassConstructorMetadata *metadata = data.GetParameter();
    delete metadata;
  }

  v8::Global<v8::FunctionTemplate> template_;
  
};

class NeonBaseClassConstructorMetadata: public NeonClassConstructorMetadata {
public:

  NeonBaseClassConstructorMetadata(NeonSys_ConstructorKernelCallback kernel, NeonSys_CreateInternalsCallback cb)
    : NeonClassConstructorMetadata(kernel)
  {
    create_internals_callback_ = cb;
  }

  virtual bool construct(v8::Local<v8::Object> self, int argc, v8::Local<v8::Value> argv[]) {
    void *internals = create_internals_callback_(argc, argv);
    self->SetAlignedPointerInInternalField(0, internals);
    return kernel_(self, argc, argv);
  }

private:

  NeonSys_CreateInternalsCallback create_internals_callback_;
};

class NeonDerivedClassConstructorMetadata: public NeonClassConstructorMetadata {
public:

  NeonDerivedClassConstructorMetadata(NeonSys_ConstructorKernelCallback kernel, NeonClassConstructorMetadata *super_metadata)
    : NeonClassConstructorMetadata(kernel)
  {
    super_metadata_ = super_metadata;
  }

  virtual bool construct(v8::Local<v8::Object> self, int argc, v8::Local<v8::Value> argv[]) {
    return super_metadata_->construct(self, argc, argv) &&
           kernel_(self, argc, argv);
  }

private:

  NeonClassConstructorMetadata *super_metadata_;
};

#endif
