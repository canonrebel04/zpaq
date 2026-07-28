#ifndef ZPAQ_FFI_H
#define ZPAQ_FFI_H

#ifdef __cplusplus
#include <cstddef>
#include <cstdint>
extern "C" {
#else
#include <stddef.h>
#include <stdint.h>
#endif

// Callback for reading input data.
// Should read up to `size` bytes into `buf`.
// Return the number of bytes actually read (0 on EOF, < 0 on error).
typedef int (*zpaq_read_cb)(void* user_data, char* buf, int size);

// Callback for writing output data.
// Should write `size` bytes from `buf`.
// Return the number of bytes written (or size on success, < 0 on error).
typedef int (*zpaq_write_cb)(void* user_data, const char* buf, int size);

/**
 * Compress stream using libzpaq.
 * @param read_fn Callback to read input data.
 * @param read_ctx Opaque pointer passed to read_fn.
 * @param write_fn Callback to write output data.
 * @param write_ctx Opaque pointer passed to write_fn.
 * @param method Compression method string (e.g., "1", "2", "14,128,0", etc.). If NULL, defaults to "1".
 * @return 0 on success, non-zero on error.
 */
int zpaq_compress_stream(
    zpaq_read_cb read_fn,
    void* read_ctx,
    zpaq_write_cb write_fn,
    void* write_ctx,
    const char* method
);

/**
 * Decompress stream using libzpaq.
 * @param read_fn Callback to read input data.
 * @param read_ctx Opaque pointer passed to read_fn.
 * @param write_fn Callback to write output data.
 * @param write_ctx Opaque pointer passed to write_fn.
 * @return 0 on success, non-zero on error.
 */
int zpaq_decompress_stream(
    zpaq_read_cb read_fn,
    void* read_ctx,
    zpaq_write_cb write_fn,
    void* write_ctx
);

#ifdef __cplusplus
}
#endif

#endif // ZPAQ_FFI_H
