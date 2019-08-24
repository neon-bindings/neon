#ifndef NEON_TASK_H_
#define NEON_TASK_H_

#include <uv.h>
#include "neon.h"
#include "v8.h"

namespace neon {

class Task {
public:
  Task(v8::Isolate *isolate,
       void *rust_task,
       Neon_TaskPerformCallback perform,
       Neon_TaskCompleteCallback complete,
       v8::Local<v8::Function> callback)
    : isolate_(isolate),
      rust_task_(rust_task),
      perform_(perform),
      complete_(complete)
  {
    request_.data = this;
    result_ = nullptr;
    // Save the callback to be invoked when the task completes.
    callback_.Reset(isolate, callback);
    // Save the context (aka realm) to be used when invoking the callback.
    context_.Reset(isolate, isolate->GetCurrentContext());
  }

  void execute() {
    result_ = perform_(rust_task_);
  }

  void complete() {
    // Ensure that we have all the proper scopes installed on the C++ stack before
    // invoking the callback, and use the context (i.e. realm) we saved with the task.
    v8::Isolate::Scope isolate_scope(isolate_);
    v8::HandleScope handle_scope(isolate_);
    v8::Local<v8::Context> context = v8::Local<v8::Context>::New(isolate_, context_);
    v8::Context::Scope context_scope(context);

    v8::Local<v8::Value> argv[2];

    argv[0] = v8::Null(isolate_);
    argv[1] = v8::Undefined(isolate_);

    {
      v8::TryCatch trycatch(isolate_);

      v8::Local<v8::Value> completion;

      complete_(rust_task_, result_, &completion);

      if (trycatch.HasCaught()) {
        argv[0] = trycatch.Exception();
      } else {
        argv[1] = completion;
      }
    }

    v8::Local<v8::Function> callback = v8::Local<v8::Function>::New(isolate_, callback_);
    node::MakeCallback(isolate_, context->Global(), callback, 2, argv);
    callback_.Reset();
    context_.Reset();
  }

  void *get_result() {
    return result_;
  }

  uv_work_t request_;

private:
  v8::Isolate *isolate_;
  void *rust_task_;
  Neon_TaskPerformCallback perform_;
  Neon_TaskCompleteCallback complete_;
  void *result_;
  v8::Persistent<v8::Function> callback_;
  v8::Persistent<v8::Context> context_;
};

void execute_task(uv_work_t *request) {
  Task *task = static_cast<Task*>(request->data);
  task->execute();
}

void complete_task(uv_work_t *request) {
  Task *task = static_cast<Task*>(request->data);
  task->complete();
  delete task;
}

void queue_task(Task *task) {
  uv_queue_work(uv_default_loop(),
                &task->request_,
                execute_task,
                (uv_after_work_cb)complete_task);
}

}

#endif
