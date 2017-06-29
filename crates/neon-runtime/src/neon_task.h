#ifndef NEON_TASK_H_
#define NEON_TASK_H_

#include <nan.h>
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
    callback_.Reset(isolate, callback);
  }

  ~Task() {
    callback_.Reset();
  }

  void execute() {
    result_ = perform_(rust_task_);
  }

  void complete() {
    Nan::HandleScope scope;

    v8::TryCatch trycatch(isolate_);

    v8::Local<v8::Value> argv[2];
    v8::Local<v8::Value> completion;

    complete_(rust_task_, result_, &completion);

    if (trycatch.HasCaught()) {
      argv[0] = trycatch.Exception();
      argv[1] = v8::Undefined(isolate_);
    } else {
      argv[0] = v8::Null(isolate_);
      argv[1] = completion;
    }

    v8::Local<v8::Function> callback = v8::Local<v8::Function>::New(isolate_, callback_);
    callback->Call(isolate_->GetCurrentContext(), v8::Null(isolate_), 2, argv);
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
