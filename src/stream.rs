//! In-memory compression/decompression streams

use std::error;
use std::fmt;
use std::io;
use std::mem;
use std::slice;

use brotli_sys;
use libc::{c_int, c_void, size_t};

/// In-memory state for decompressing brotli-encoded data.
///
/// This stream is at the heart of the I/O streams and is used to decompress an
/// incoming brotli stream.
pub struct Decompress {
    state: *mut brotli_sys::BrotliState,
}

unsafe impl Send for Decompress {}
unsafe impl Sync for Decompress {}

/// In-memory state for compressing/encoding data with brotli
///
/// This stream is at the heart of the I/O encoders and is used to compress
/// data.
pub struct Compress {
    state: *mut brotli_sys::RustBrotliCompressor,
}

unsafe impl Send for Compress {}
unsafe impl Sync for Compress {}

/// Parameters passed to various compression routines.
pub struct CompressParams {
    params: *mut brotli_sys::RustBrotliParams,
}

unsafe impl Send for CompressParams {}
unsafe impl Sync for CompressParams {}

/// Possible choices for modes of compression
pub enum CompressMode {
    /// Default compression mode, the compressor does not know anything in
    /// advance about the properties of the input.
    Generic = brotli_sys::RUST_MODE_GENERIC as isize,
    /// Compression mode for utf-8 formatted text input.
    Text = brotli_sys::RUST_MODE_TEXT as isize,
    /// Compression mode in WOFF 2.0.
    Font = brotli_sys::RUST_MODE_FONT as isize,
}

/// Error that can happen from decompressing or compressing a brotli stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Error(());

/// Possible status results returned from compressing or decompressing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    /// Decompression was successful and has finished
    Finished,
    /// More input is needed to continue
    NeedInput,
    /// More output is needed to continue
    NeedOutput,
}

impl Decompress {
    /// Creates a new brotli decompression/decoding stream ready to receive
    /// data.
    pub fn new() -> Decompress {
        unsafe {
            let state = brotli_sys::BrotliCreateState(None, None, 0 as *mut _);
            assert!(!state.is_null());
            Decompress { state: state }
        }
    }

    /// Decompress a block of input data into a block of output data.
    ///
    /// This function will decompress the data in `input` and place the output
    /// in `output`, returning the result. Possible statuses that can be
    /// returned are that the stream is finished, more input is needed, or more
    /// output space is needed.
    ///
    /// The `input` slice is updated to point to the remaining data that was not
    /// consumed, and the `output` slice is updated to point to the portion of
    /// the output slice that still needs to be filled in.
    ///
    /// # Errors
    ///
    /// If the input stream is not a valid brotli stream, then an error is
    /// returned.
    pub fn decompress(&mut self,
                      input: &mut &[u8],
                      output: &mut &mut [u8]) -> Result<Status, Error> {
        let mut available_in = input.len();
        let mut next_in = input.as_ptr();
        let mut available_out = output.len();
        let mut next_out = output.as_mut_ptr();
        let mut total_out = 0;
        let r = unsafe {
            brotli_sys::BrotliDecompressStream(&mut available_in,
                                               &mut next_in,
                                               &mut available_out,
                                               &mut next_out,
                                               &mut total_out,
                                               self.state)
        };
        *input = &input[input.len() - available_in..];
        let out_len = output.len();
        *output = &mut mem::replace(output, &mut [])[out_len - available_out..];
        Decompress::rc(r)
    }

    /// Decompress a block of input data into the remaining capacity of a
    /// vector.
    ///
    /// This function is the same as `decompress` except that it will fill up
    /// the remaining capacity in a destination vector and update the length as
    /// necessary.
    pub fn decompress_vec(&mut self,
                          input: &mut &[u8],
                          output: &mut Vec<u8>) -> Result<Status, Error> {
        let cap = output.capacity();
        let len = output.len();

        unsafe {
            let (ret, remaining) = {
                let ptr = output.as_mut_ptr().offset(len as isize);
                let mut out = slice::from_raw_parts_mut(ptr, cap - len);
                let r = self.decompress(input, &mut out);
                (r, out.len())
            };
            output.set_len(cap - remaining);
            return ret
        }
    }

    fn rc(rc: brotli_sys::BrotliResult) -> Result<Status, Error> {
        match rc {
            brotli_sys::BROTLI_RESULT_ERROR => Err(Error(())),
            brotli_sys::BROTLI_RESULT_SUCCESS => Ok(Status::Finished),
            brotli_sys::BROTLI_RESULT_NEEDS_MORE_INPUT => Ok(Status::NeedInput),
            brotli_sys::BROTLI_RESULT_NEEDS_MORE_OUTPUT => Ok(Status::NeedOutput),
            n => panic!("unknown return code: {}", n)
        }
    }
}

impl Drop for Decompress {
    fn drop(&mut self) {
        unsafe {
            brotli_sys::BrotliDestroyState(self.state);
        }
    }
}

/// Returns the decompressed size of the given encoded stream.
///
/// This function only works if the encoded buffer has a single meta block,
/// or if it has two meta-blocks, where the first is uncompressed and the
/// second is empty.
pub fn decompressed_size(data: &[u8]) -> Result<usize, Error> {
    let mut size = 0;
    let ret = unsafe {
        brotli_sys::BrotliDecompressedSize(data.len(),
                                           data.as_ptr(),
                                           &mut size)
    };
    if ret == 0 {
        Err(Error(()))
    } else {
        Ok(size)
    }
}

/// Decompress data in one go in memory.
///
/// Decompresses the data in `input` into the `output` buffer. The `output`
/// buffer is updated to point to the actual output slice if successful
pub fn decompress_buf(input: &[u8],
                      output: &mut &mut [u8]) -> Result<Status, Error> {
    let mut size = output.len();
    let r = unsafe {
        brotli_sys::BrotliDecompressBuffer(input.len(),
                                           input.as_ptr(),
                                           &mut size,
                                           output.as_mut_ptr())
    };
    *output = &mut mem::replace(output, &mut [])[..size];
    Decompress::rc(r)
}

impl Compress {
    /// Creates a new compressor ready to encode data into brotli
    pub fn new(params: &CompressParams) -> Compress {
        unsafe {
            let state = brotli_sys::RustBrotliCompressorCreate(params.params);
            assert!(!state.is_null());
            Compress { state: state }
        }
    }

    /// Returns the maximum amount of data that can be internally buffered to
    /// get processed at once.
    ///
    /// Data is fed into this compressor via the `copy_input` function, and then
    /// it's later compressed via the `write_brotli_data` function.
    pub fn input_block_size(&self) -> usize {
        unsafe { brotli_sys::RustBrotliCompressorInputBlockSize(self.state) }
    }

    // Apparently this is just a shim around CopyInputToRingBuffer,
    // WriteBrotliData, and then finally a memcpy?
    //
    // #[allow(dead_code)]
    // fn write_metablock(&mut self,
    //                    input: &[u8],
    //                    last: bool,
    //                    encoded: &mut [u8]) -> Result<usize, Error> {
    //     let mut size = encoded.len();
    //     let r = unsafe {
    //         brotli_sys::RustBrotliCompressorWriteMetaBlock(self.state,
    //                                                        input.len(),
    //                                                        input.as_ptr(),
    //                                                        last as c_int,
    //                                                        &mut size,
    //                                                        encoded.as_mut_ptr())
    //     };
    //     if r == 0 {
    //         Err(Error(()))
    //     } else {
    //         Ok(size)
    //     }
    // }

    // Maybe someone will eventually come up with a use for this?
    //
    // #[allow(dead_code)]
    // fn write_metadata(&mut self,
    //                   input: &[u8],
    //                   last: bool,
    //                   encoded: &mut [u8]) -> Result<usize, Error> {
    //     let mut size = encoded.len();
    //     let r = unsafe {
    //         brotli_sys::RustBrotliCompressorWriteMetadata(self.state,
    //                                                       input.len(),
    //                                                       input.as_ptr(),
    //                                                       last as c_int,
    //                                                       &mut size,
    //                                                       encoded.as_mut_ptr())
    //     };
    //     if r == 0 {
    //         Err(Error(()))
    //     } else {
    //         Ok(size)
    //     }
    // }

    // This is just a shim around WriteMetaBlock, which is in turn just a shim
    // around WriteBrotliData, so let's just delegate there I guess?
    //
    // #[allow(dead_code)]
    // fn finish_stream(&mut self, output: &mut [u8]) -> Result<usize, Error> {
    //     let mut size = output.len();
    //     let r = unsafe {
    //         brotli_sys::RustBrotliCompressorFinishStream(self.state,
    //                                                      &mut size,
    //                                                      output.as_mut_ptr())
    //     };
    //     if r == 0 {
    //         Err(Error(()))
    //     } else {
    //         Ok(size)
    //     }
    // }

    /// Feeds data into this compressor.
    ///
    /// This compressor can store up to `self.input_block_size()` bytes
    /// internally after which the `compress` call must be made to generate all
    /// output of the compressor.
    ///
    /// If too much data is copied in then the next call to `compress` will
    /// generate an error.
    pub fn copy_input(&mut self, input: &[u8]) {
        unsafe {
            brotli_sys::RustBrotliCompressorCopyInputToRingBuffer(self.state,
                                                                  input.len(),
                                                                  input.as_ptr())
        }
    }

    /// Compresses the internal data in this compressor, returning the output
    /// buffer of the compressed data.
    ///
    /// After data has been fed to this compressor via the `copy_input` method,
    /// the data is then compressed by calling this method. The `last` flag
    /// indicates whether this is the last block of the input (it should only
    /// get passed on EOF), and the `force_flush` flag indicates whether a new
    /// meta-block should be created to flush the internal data.
    ///
    /// Returns an error, if any, and otherwise the internal buffer which
    /// contains the output data of compressed information.
    pub fn compress(&mut self, last: bool, force_flush: bool)
                    -> Result<&[u8], Error> {
        let mut size = 0;
        let mut ptr = 0 as *mut _;
        unsafe {
            let (last, flush) = (last as c_int, force_flush as c_int);
            let r = brotli_sys::RustBrotliCompressorWriteBrotliData(self.state,
                                                                    last,
                                                                    flush,
                                                                    &mut size,
                                                                    &mut ptr);
            if r == 0 {
                Err(Error(()))
            } else if size == 0 {
                Ok(slice::from_raw_parts_mut(1 as *mut _, size))
            } else {
                Ok(slice::from_raw_parts_mut(ptr, size))
            }
        }
    }
}

impl Drop for Compress {
    fn drop(&mut self) {
        unsafe {
            brotli_sys::RustBrotliCompressorDestroy(self.state);
        }
    }
}

/// Compresses the data in `input` into `output`.
///
/// The `output` buffer is updated to point to the exact slice which contains
/// the output data.
///
/// If successful, the amount of compressed bytes are returned (the size of the
/// `output` slice), and otherwise an error is returned.
pub fn compress_buf(params: &CompressParams,
                    input: &[u8],
                    output: &mut &mut [u8]) -> Result<usize, Error> {
    let mut size = output.len();
    let r = unsafe {
        brotli_sys::RustBrotliCompressBuffer(params.params,
                                             input.len(),
                                             input.as_ptr(),
                                             &mut size,
                                             output.as_mut_ptr())
    };
    *output = &mut mem::replace(output, &mut [])[..size];
    if r == 0 {
        Err(Error(()))
    } else {
        Ok(size)
    }
}

/// Compresses the data in `input` into `output`.
///
/// The `output` vector will be pushed onto and reallocated if necessary, but
/// the entirety of the compressed `input` will be present in `output` when
/// finished.
pub fn compress_vec(params: &CompressParams,
                    input: &[u8],
                    output: &mut Vec<u8>) -> Result<(), Error> {
    let r = unsafe {
        brotli_sys::RustBrotliCompressBufferVec(params.params,
                                                input.len(),
                                                input.as_ptr(),
                                                output as *mut _ as *mut c_void,
                                                callback)
    };
    return if r == 0 {
        Err(Error(()))
    } else {
        Ok(())
    };

    extern fn callback(output: *mut c_void,
                       buf: *const c_void,
                       size: size_t) -> c_int {
        unsafe {
            let output = &mut *(output as *mut Vec<u8>);
            let input = slice::from_raw_parts(buf as *mut u8, size);
            output.extend_from_slice(input);
            1 // "true" == all data written
        }
    }
}

impl CompressParams {
    /// Creates a new default set of compression parameters.
    pub fn new() -> CompressParams {
        unsafe {
            let params = brotli_sys::RustBrotliParamsCreate();
            assert!(!params.is_null());
            CompressParams { params: params }
        }
    }

    /// Set the mode of this compression.
    pub fn mode(&mut self, mode: CompressMode) -> &mut CompressParams {
        unsafe {
            brotli_sys::RustBrotliParamsSetMode(self.params,
                                                mode as brotli_sys::RustBrotliMode);
        }
        self
    }

    /// Controls the compression-speed vs compression-density tradeoffs.
    ///
    /// The higher the quality, the slower the compression. Currently the range
    /// for the quality is 0 to 11.
    pub fn quality(&mut self, quality: u32) -> &mut CompressParams {
        unsafe {
            brotli_sys::RustBrotliParamsSetQuality(self.params,
                                                   quality as c_int);
        }
        self
    }

    /// Sets the base 2 logarithm of the sliding window size.
    ///
    /// Currently the range is 10 to 24.
    pub fn lgwin(&mut self, lgwin: u32) -> &mut CompressParams {
        unsafe {
            brotli_sys::RustBrotliParamsSetLgwin(self.params,
                                                 lgwin as c_int);
        }
        self
    }

    /// Sets the base 2 logarithm of the maximum input block size.
    ///
    /// Currently the range is 16 to 24, and if set to 0 the value will be set
    /// based on the quality.
    pub fn lgblock(&mut self, lgblock: u32) -> &mut CompressParams {
        unsafe {
            brotli_sys::RustBrotliParamsSetLgblock(self.params,
                                                   lgblock as c_int);
        }
        self
    }
}

impl Drop for CompressParams {
    fn drop(&mut self) {
        unsafe {
            brotli_sys::RustBrotliParamsDestroy(self.params);
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        error::Error::description(self).fmt(f)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "brotli error"
    }
}

impl From<Error> for io::Error {
    fn from(_err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, "brotli error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompress_error() {
        let mut d = Decompress::new();
        d.decompress(&mut &[0; 1024][..], &mut &mut [0; 2048][..]).unwrap_err();
    }

    #[test]
    fn compress_vec_smoke() {
        let mut data = Vec::new();
        compress_vec(&CompressParams::new(), b"hello!", &mut data).unwrap();

        let mut dst = [0; 128];
        assert_eq!(decompressed_size(&data), Ok(6));
        {
            let mut dst = &mut dst[..];
            let n = decompress_buf(&data, &mut dst).unwrap();
            assert_eq!(n, Status::Finished);
            assert_eq!(dst.len(), 6);
        }
        assert_eq!(&dst[..6], b"hello!");
    }

    #[test]
    fn compress_buf_smoke() {
        let mut data = [0; 128];
        let mut data = &mut data[..];
        compress_buf(&CompressParams::new(), b"hello!", &mut data).unwrap();
        assert_eq!(decompressed_size(data), Ok(6));

        let mut dst = [0; 128];
        {
            let mut dst = &mut dst[..];
            let n = decompress_buf(data, &mut dst).unwrap();
            assert_eq!(n, Status::Finished);
            assert_eq!(dst.len(), 6);
        }
        assert_eq!(&dst[..6], b"hello!");
    }

    #[test]
    fn decompressor_smoke() {
        let mut data = Vec::new();
        compress_vec(&CompressParams::new(), b"hello!", &mut data).unwrap();

        let mut d = Decompress::new();
        let mut dst = [0; 128];
        {
            let mut data = &data[..];
            let mut dst = &mut dst[..];
            assert_eq!(d.decompress(&mut data, &mut dst), Ok(Status::Finished));
        }
        assert_eq!(&dst[..6], b"hello!");

        let mut d = Decompress::new();
        let mut dst = Vec::with_capacity(10);
        assert_eq!(d.decompress_vec(&mut &data[..], &mut dst),
                   Ok(Status::Finished));
        assert_eq!(&dst, b"hello!");
    }

    #[test]
    fn compress_smoke() {
        let mut data = Vec::new();
        let mut c = Compress::new(&CompressParams::new());
        c.copy_input(b"hello!");
        data.extend_from_slice(c.compress(true, false).unwrap());

        let mut dst = [0; 128];
        decompress_buf(&data, &mut &mut dst[..]).unwrap();
        assert_eq!(&dst[..6], b"hello!");

        data.truncate(0);
        let mut c = Compress::new(&CompressParams::new());
        c.copy_input(b"hel");
        data.extend_from_slice(c.compress(false, true).unwrap());
        c.copy_input(b"lo!");
        data.extend_from_slice(c.compress(true, false).unwrap());

        let mut dst = [0; 128];
        decompress_buf(&data, &mut &mut dst[..]).unwrap();
        assert_eq!(&dst[..6], b"hello!");
    }
}
