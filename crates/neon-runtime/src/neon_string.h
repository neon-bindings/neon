#ifndef NEON_STRING_H_
#define NEON_STRING_H_

namespace neon {

class Slice {
public:
  Slice(const char *buffer, uint32_t length)
    : buffer_(buffer), length_(length)
  {
  }

  v8::Local<v8::String> ToJsString(v8::Isolate *isolate, const char *fallback) {
    v8::MaybeLocal<v8::String> maybe;
    v8::Local<v8::String> result;

    maybe = v8::String::NewFromUtf8(isolate, buffer_, v8::NewStringType::kNormal, length_);
    if (maybe.ToLocal(&result)) {
      return result;
    }

    maybe = v8::String::NewFromOneByte(isolate, (const uint8_t *)fallback, v8::NewStringType::kNormal);
    if (maybe.ToLocal(&result)) {
      return result;
    }

    maybe = v8::String::NewFromOneByte(isolate, (const uint8_t *)"?", v8::NewStringType::kNormal);
    maybe.ToLocal(&result);
    return result;
  }

  const char *GetBuffer() {
    return buffer_;
  }

  uint32_t GetLength() {
    return length_;
  }

private:

  const char *buffer_;
  uint32_t length_;
};


class String {
public:
  String(uint32_t length) {
    length_ = length;
    buffer_ = new char [length];
    cursor_ = 0;
  }

  ~String() {
    delete buffer_;
    buffer_ = nullptr;
    length_ = 0;
  }

  Slice Borrow() {
    return Slice(buffer_, length_);
  }

  char *GetBuffer() {
    return buffer_;
  }

  uint32_t GetLength() {
    return length_;
  }

  String& operator<<(const char *s) {
    while (*s) {
      buffer_[cursor_] = *s;
      cursor_++;
      s++;
    }
    return *this;
  }

  String& operator<<(Slice s) {
    uint32_t length = s.GetLength();
    memcpy(buffer_ + cursor_, s.GetBuffer(), length);
    cursor_ += length;
    return *this;
  }

private:

  char *buffer_;
  uint32_t length_;
  uint32_t cursor_;
};

}; // end namespace neon

#endif
