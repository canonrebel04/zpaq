use std::os::raw::{c_char, c_int, c_void};

pub type ZpaqReadCb = Option<
    unsafe extern "C" fn(user_data: *mut c_void, buf: *mut c_char, size: c_int) -> c_int,
>;

pub type ZpaqWriteCb = Option<
    unsafe extern "C" fn(user_data: *mut c_void, buf: *const c_char, size: c_int) -> c_int,
>;

extern "C" {
    pub fn zpaq_compress_stream(
        read_fn: ZpaqReadCb,
        read_ctx: *mut c_void,
        write_fn: ZpaqWriteCb,
        write_ctx: *mut c_void,
        method: *const c_char,
    ) -> c_int;

    pub fn zpaq_decompress_stream(
        read_fn: ZpaqReadCb,
        read_ctx: *mut c_void,
        write_fn: ZpaqWriteCb,
        write_ctx: *mut c_void,
    ) -> c_int;
}
