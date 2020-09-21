#ifndef NEON_EVENTHANDLER_H_
#define NEON_EVENTHANDLER_H_

#include <uv.h>
#include "neon.h"
#include "v8.h"
#include <mutex>

namespace neon {

    // THe implementation of this class was adapted from
    // https://github.com/mika-fischer/napi-thread-safe-callback
    class EventHandler {
    public:
        EventHandler(v8::Isolate *isolate,
                           v8::Local<v8::Value> self,
                           v8::Local<v8::Function> callback): isolate_(isolate), close_(false)
        {
            async_.data = this;
            uv_async_init(uv_default_loop(), &async_, async_complete);
            // Save the this argument and the callback to be invoked.
            self_.Reset(isolate, self);
            callback_.Reset(isolate, callback);
            // Save the context (aka realm) to be used when invoking the callback.
            context_.Reset(isolate, isolate->GetCurrentContext());
        }

        ~EventHandler() {
            self_.Reset();
            callback_.Reset();
            context_.Reset();
        }

        void schedule(void *rust_callback, Neon_EventHandler handler) {
            {
                std::lock_guard<std::mutex> lock(mutex_);
                handlers_.push_back({ rust_callback, handler });
            }
            uv_async_send(&async_);
        }

        void close() {
            // close is called when the rust struct is dropped
            // this guarantees that it's called only once and
            // that no other method (call) will be called after it.
            close_ = true;
            uv_async_send(&async_);
        }

        void complete() {
            // Ensure that we have all the proper scopes installed on the C++ stack before
            // invoking the callback, and use the context (i.e. realm) we saved with the task.
            v8::Isolate::Scope isolate_scope(isolate_);
            v8::HandleScope handle_scope(isolate_);
            v8::Local<v8::Context> context = v8::Local<v8::Context>::New(isolate_, context_);
            v8::Context::Scope context_scope(context);

            v8::Local<v8::Value> self = v8::Local<v8::Value>::New(isolate_, self_);
            v8::Local<v8::Function> callback = v8::Local<v8::Function>::New(isolate_, callback_);

            while (true) {
                std::vector<HandlerData> handlers;
                {
                    std::lock_guard<std::mutex> lock(mutex_);
                    if (handlers_.empty()) {
                        break;
                    } else {
                        handlers.swap(handlers_);
                    }
                }
                for (const HandlerData &data : handlers) {
                    data.handler(self, callback, data.rust_callback);
                }
            }

            if (close_) {
                uv_close(reinterpret_cast<uv_handle_t*>(&async_), [](uv_handle_t* handle) {
                    delete static_cast<EventHandler*>(handle->data);
                });
            }
        }

    private:
        static void async_complete(uv_async_t* handle) {
            EventHandler* cb = static_cast<EventHandler*>(handle->data);
            cb->complete();
        }

        uv_async_t async_;
        v8::Isolate *isolate_;
        v8::Persistent<v8::Value> self_;
        v8::Persistent<v8::Function> callback_;
        v8::Persistent<v8::Context> context_;

        struct HandlerData {
            void *rust_callback;
            Neon_EventHandler handler;
        };

        std::mutex mutex_;
        std::vector<HandlerData> handlers_;

        bool close_;
    };
}

#endif
