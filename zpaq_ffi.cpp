#include "zpaq_ffi.h"
#include "libzpaq.h"
#include <stdexcept>

namespace libzpaq {
void error(const char* msg) {
    throw std::runtime_error(msg ? msg : "libzpaq error");
}
}

namespace {
class FFIReader : public libzpaq::Reader {
    zpaq_read_cb read_fn;
    void* read_ctx;
public:
    FFIReader(zpaq_read_cb fn, void* ctx) : read_fn(fn), read_ctx(ctx) {}

    int get() override {
        char c;
        int n = read(&c, 1);
        if (n <= 0) return -1;
        return static_cast<unsigned char>(c);
    }

    int read(char* buf, int n) override {
        if (!read_fn || n <= 0) return 0;
        int r = read_fn(read_ctx, buf, n);
        if (r < 0) {
            libzpaq::error("FFI read callback error");
        }
        return r;
    }
};

class FFIWriter : public libzpaq::Writer {
    zpaq_write_cb write_fn;
    void* write_ctx;
public:
    FFIWriter(zpaq_write_cb fn, void* ctx) : write_fn(fn), write_ctx(ctx) {}

    void put(int c) override {
        char ch = static_cast<char>(c);
        write(&ch, 1);
    }

    void write(const char* buf, int n) override {
        if (!write_fn || n <= 0) return;
        int w = write_fn(write_ctx, buf, n);
        if (w < n) {
            libzpaq::error("FFI write callback error");
        }
    }
};
} // namespace

extern "C" {

int zpaq_compress_stream(
    zpaq_read_cb read_fn,
    void* read_ctx,
    zpaq_write_cb write_fn,
    void* write_ctx,
    const char* method
) {
    if (!read_fn || !write_fn) return -1;
    if (!method) method = "1";
    try {
        FFIReader reader(read_fn, read_ctx);
        FFIWriter writer(write_fn, write_ctx);
        libzpaq::compress(&reader, &writer, method);
        return 0;
    } catch (...) {
        return -1;
    }
}

int zpaq_decompress_stream(
    zpaq_read_cb read_fn,
    void* read_ctx,
    zpaq_write_cb write_fn,
    void* write_ctx
) {
    if (!read_fn || !write_fn) return -1;
    try {
        FFIReader reader(read_fn, read_ctx);
        FFIWriter writer(write_fn, write_ctx);
        libzpaq::decompress(&reader, &writer);
        return 0;
    } catch (...) {
        return -1;
    }
}

} // extern "C"
