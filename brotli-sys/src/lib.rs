#![allow(bad_style)]

extern crate libc;

use libc::{c_void, size_t, c_int};

#[cfg(target_env = "msvc")]
#[doc(hidden)]
pub type __enum_ty = libc::c_int;
#[cfg(not(target_env = "msvc"))]
#[doc(hidden)]
pub type __enum_ty = libc::c_uint;

pub type brotli_alloc_func = extern fn(*mut c_void, size_t) -> *mut c_void;
pub type brotli_free_func = extern fn(*mut c_void, *mut c_void);
pub type BrotliResult = __enum_ty;
pub type BrotliRunningState = __enum_ty;
pub type BrotliRunningMetablockHeaderState = __enum_ty;
pub type BrotliRunningUncompressedState = __enum_ty;
pub type BrotliRunningTreeGroupState = __enum_ty;
pub type BrotliRunningContextMapState = __enum_ty;
pub type BrotliRunningHuffmanState = __enum_ty;
pub type BrotliRunningDecodeUint8State = __enum_ty;
pub type BrotliRunningReadBlockLengthState = __enum_ty;

pub type BrotliState = BrotliStateStruct;

#[repr(C)]
pub struct BrotliStateStruct {
    pub state: BrotliRunningState,
    pub loop_counter: c_int,
    pub br: BrotliBitReader,
    pub alloc_func: Option<brotli_alloc_func>,
    pub free_func: Option<brotli_free_func>,
    pub memory_manager_opaque: *mut c_void,
    buffer: u64,
    pub buffer_length: u32,
    pub pos: c_int,
    pub max_backward_distance: c_int,
    pub max_backward_distance_minus_custom_dict_size: c_int,
    pub max_distance: c_int,
    pub ringbuffer_size: c_int,
    pub ringbuffer_mask: c_int,
    pub dist_rb_idx: c_int,
    pub dist_rb: [c_int; 4],
    pub ringbuffer: *mut u8,
    pub ringbuffer_end: *mut u8,
    pub htree_command: *mut HuffmanCode,
    pub context_lookup1: *const u8,
    pub context_lookup2: *const u8,
    pub context_map_slice: *mut u8,
    pub dist_context_map_slice: *mut u8,
    pub sub_loop_counter: u32,
    pub literal_hgroup: HuffmanTreeGroup,
    pub insert_copy_hgroup: HuffmanTreeGroup,
    pub distance_hgroup: HuffmanTreeGroup,
    pub block_type_trees: *mut HuffmanCode,
    pub block_len_trees: *mut HuffmanCode,
    pub trivial_literal_context: c_int,
    pub distance_context: c_int,
    pub meta_block_remaining_len: c_int,
    pub block_length_index: u32,
    pub block_length: [u32; 3],
    pub num_block_types: [u32; 3],
    pub block_type_rb: [u32; 6],
    pub distance_postfix_bits: u32,
    pub num_direct_distance_codes: u32,
    pub distance_postfix_mask: c_int,
    pub num_dist_htrees: u32,
    pub dist_context_map: *mut u8,
    pub literal_htree: *mut HuffmanCode,
    pub literal_htree_index: u8,
    pub dist_htree_index: u8,
    pub repeat_code_len: u32,
    pub prev_code_len: u32,
    pub copy_length: c_int,
    pub distance_code: c_int,
    pub rb_roundtrips: size_t,
    pub partial_pos_out: size_t,
    pub symbol: u32,
    pub repeat: u32,
    pub space: u32,
    pub table: [HuffmanCode; 32],
    pub symbol_lists: *mut u16,
    pub symbols_lists_array: [u16; 720],
    pub next_symbol: [c_int; 32],
    pub code_length_code_lengths: [u8; 18],
    pub code_length_histo: [u16; 16],
    pub htree_index: c_int,
    pub next: *mut HuffmanCode,
    pub context_index: u32,
    pub max_run_length_prefix: u32,
    pub code: u32,
    pub context_map_table: [HuffmanCode; 646],
    pub mtf_upper_bound: u32,
    pub mtf: [u8; 256],
    pub custom_dict: *const u8,
    pub custom_dict_size: c_int,
    pub substate_metablock_header: BrotliRunningMetablockHeaderState,
    pub substate_tree_group: BrotliRunningTreeGroupState,
    pub substate_context_map: BrotliRunningContextMapState,
    pub substate_uncompressed: BrotliRunningUncompressedState,
    pub substate_huffman: BrotliRunningHuffmanState,
    pub substate_decode_uint8: BrotliRunningDecodeUint8State,
    pub substate_read_block_length: BrotliRunningReadBlockLengthState,
    pub is_last_metablock: u8,
    pub is_uncompressed: u8,
    pub is_metadata: u8,
    pub size_nibbles: u8,
    pub window_bits: u32,
    pub num_literal_htrees: u32,
    pub context_map: *mut u8,
    pub context_modes: *mut u8,
}

#[cfg(target_pointer_width = "32")]
pub type reg_t = u32;
#[cfg(target_pointer_width = "64")]
pub type reg_t = u64;

#[repr(C)]
pub struct BrotliBitReader {
    pub val_: reg_t,
    pub bit_pos_: u32,
    pub next_in: *const u8,
    pub avail_in: size_t,
}

#[repr(C)]
pub struct HuffmanCode {
    pub bits: u8,
    pub value: u16,
}

#[repr(C)]
pub struct HuffmanTreeGroup {
    pub htrees: *mut *mut HuffmanCode,
    pub codes: *mut HuffmanCode,
    pub alphabet_size: u16,
    pub num_htrees: u16,
}

pub const BROTLI_RESULT_ERROR: BrotliResult = 0;
pub const BROTLI_RESULT_SUCCESS: BrotliResult = 1;
pub const BROTLI_RESULT_NEEDS_MORE_INPUT: BrotliResult = 2;
pub const BROTLI_RESULT_NEEDS_MORE_OUTPUT: BrotliResult = 3;

pub enum RustBrotliCompressor {}
pub enum RustBrotliParams {}
pub type RustBrotliMode = __enum_ty;

pub const RUST_MODE_GENERIC: RustBrotliMode = 0;
pub const RUST_MODE_TEXT: RustBrotliMode = 1;
pub const RUST_MODE_FONT: RustBrotliMode = 2;

extern {
    // BrotliState
    pub fn BrotliCreateState(alloc_func: Option<brotli_alloc_func>,
                             free_func: Option<brotli_free_func>,
                             opaque: *mut c_void) -> *mut BrotliState;
    pub fn BrotliDestroyState(state: *mut BrotliState);
    pub fn BrotliDecompressedSize(encoded_size: size_t,
                                  encoded_buff: *const u8,
                                  decoded_size: *mut size_t) -> c_int;
    pub fn BrotliDecompressBuffer(encoded_size: size_t,
                                  encoded_buffer: *const u8,
                                  decoded_size: *mut size_t,
                                  decoded_buffer: *mut u8) -> BrotliResult;
    pub fn BrotliDecompressStream(available_in: *mut size_t,
                                  next_in: *mut *const u8,
                                  available_out: *mut size_t,
                                  next_out: *mut *mut u8,
                                  total_out: *mut size_t,
                                  s: *mut BrotliState) -> BrotliResult;
    pub fn BrotliSetCustomDictionary(size: size_t,
                                     dict: *const u8,
                                     s: *mut BrotliState);

    // raw state
    pub fn BrotliStateInit(s: *mut BrotliState);
    pub fn BrotliStateInitWithCustomAllocators(s: *mut BrotliState,
                                               alloc_func: brotli_alloc_func,
                                               free_func: brotli_free_func,
                                               opaque: *mut c_void);
    pub fn BrotliStateCleanup(s: *mut BrotliState);
    pub fn BrotliStateMetablockBegin(s: *mut BrotliState);
    pub fn BrotliStateCleanupAfterMetablock(s: *mut BrotliState);
    pub fn BrotliHuffmanTreeGroupInit(s: *mut BrotliState,
                                      group: *mut HuffmanTreeGroup,
                                      alphabet_size: u32,
                                      ntrees: u32);
    pub fn BrotliHuffmanTreeGroupRelease(s: *mut BrotliState,
                                         group: *mut HuffmanTreeGroup);
    pub fn BrotliStateIsStreamStart(s: *const BrotliState) -> c_int;
    pub fn BrotliStateIsStreamEnd(s: *const BrotliState) -> c_int;

    // huffman
    pub fn BrotliBuildCodeLengthsHuffmanTable(root_table: *mut HuffmanCode,
                                              code_lengths: *const u8,
                                              count: *mut u16);
    pub fn BrotliBuildHuffmanTable(root_table: *mut HuffmanCode,
                                   root_bits: c_int,
                                   symbol_lists: *const u16,
                                   count_arg: *mut u16) -> u32;
    pub fn BrotliBuildSimpleHuffmanTable(table: *mut HuffmanCode,
                                         root_bits: c_int,
                                         symbols: *mut u16,
                                         num_symbols: u32) -> u32;

    // compress params
    pub fn RustBrotliParamsCreate() -> *mut RustBrotliParams;
    pub fn RustBrotliParamsDestroy(params: *mut RustBrotliParams);
    pub fn RustBrotliParamsSetMode(params: *mut RustBrotliParams,
                                   mode: RustBrotliMode);
    pub fn RustBrotliParamsSetQuality(params: *mut RustBrotliParams,
                                      quality: c_int);
    pub fn RustBrotliParamsSetLgwin(params: *mut RustBrotliParams,
                                    lgwin: c_int);
    pub fn RustBrotliParamsSetLgblock(params: *mut RustBrotliParams,
                                      lgblock: c_int);
    pub fn RustBrotliParamsSetEnableDictionary(params: *mut RustBrotliParams,
                                               enable: c_int);
    pub fn RustBrotliParamsSetEnableTransforms(params: *mut RustBrotliParams,
                                               enable: c_int);
    pub fn RustBrotliParamsSetGreedyBlockSplit(params: *mut RustBrotliParams,
                                               split: c_int);
    pub fn RustBrotliParamsSetEnableContextModeling(params: *mut RustBrotliParams,
                                                    enable: c_int);

    // compress in memory
    pub fn RustBrotliCompressBuffer(params: *const RustBrotliParams,
                                    input_size: size_t,
                                    input_buffer: *const u8,
                                    encoded_size: *mut size_t,
                                    encoded_buffer: *mut u8) -> c_int;
    pub fn RustBrotliCompressBufferVec(params: *const RustBrotliParams,
                                       input_size: size_t,
                                       input_buffer: *const u8,
                                       data: *mut c_void,
                                       callback: extern fn(*mut c_void,
                                                           *const c_void,
                                                           size_t) -> c_int)
                                       -> c_int;

    // compressor stream
    pub fn RustBrotliCompressorCreate(params: *const RustBrotliParams)
                                      -> *mut RustBrotliCompressor;
    pub fn RustBrotliCompressorDestroy(c: *mut RustBrotliCompressor);
    pub fn RustBrotliCompressorInputBlockSize(c: *const RustBrotliCompressor)
                                              -> size_t;
    pub fn RustBrotliCompressorWriteMetaBlock(c: *mut RustBrotliCompressor,
                                              input_size: size_t,
                                              input_buffer: *const u8,
                                              is_last: c_int,
                                              encoded_size: *mut size_t,
                                              encoded_buffer: *mut u8)
                                              -> c_int;
    pub fn RustBrotliCompressorWriteMetadata(c: *mut RustBrotliCompressor,
                                             input_size: size_t,
                                             input_buffer: *const u8,
                                             is_last: c_int,
                                             encoded_size: *mut size_t,
                                             encoded_buffer: *mut u8)
                                             -> c_int;
    pub fn RustBrotliCompressorFinishStream(c: *mut RustBrotliCompressor,
                                            encoded_size: *mut size_t,
                                            encoded_buffer: *mut u8) -> c_int;
    pub fn RustBrotliCompressorCopyInputToRingBuffer(c: *mut RustBrotliCompressor,
                                                     input_size: size_t,
                                                     input_buffer: *const u8);
    pub fn RustBrotliCompressorWriteBrotliData(c: *mut RustBrotliCompressor,
                                               is_last: c_int,
                                               force_flush: c_int,
                                               out_size: *mut size_t,
                                               output: *mut *mut u8) -> c_int;
}
