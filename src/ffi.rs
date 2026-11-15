
use core::slice;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::compress::compress;
use crate::decompress::decompress;
use crate::model::{
    CompressOptions as RustCompressOptions,
    CompressionLevel,
    Compressor,
    Profile,
};
use crate::util::compression::get_compressed_buffer_size_hint;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OodleStatus {
    Ok = 0,
    InvalidArgument = 1,
    DecompressionFailed = 2,
    CompressionFailed = 3,
    InvalidCompressedData = 4,
    InvalidHeader = 5,
    InternalError = 255,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OodleProfile {
    Main = 0,
    Reduced = 1,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OodleCompressor {
    Kraken = 0,
    Leviathan = 1,
    Mermaid = 2,
    Selkie = 3,
    Hydra = 4,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OodleCompressionLevel {
    SuperFast = 0,
    VeryFast = 1,
    Fast = 2,
    Normal = 3,
    Optimal1 = 4,
    Optimal2 = 5,
    Optimal3 = 6,
    Optimal4 = 7,
    Optimal5 = 8,
    HyperFast1 = 9,
    HyperFast2 = 10,
    HyperFast3 = 11,
    HyperFast4 = 12,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OodleCompressOptions {
    pub min_match_len: i32,
    pub seek_chunk_reset: u8,
    pub seek_chunk_len: i32,
    pub profile: i32,
    pub dictionary_size: i32,
    pub space_speed_tradeoff_bytes: i32,
    pub send_quantum_crcs: u8,
    pub max_local_dictionary_size: i32,
    pub make_long_range_matcher: u8,
    pub match_table_size_log2: i32,
    pub far_match_min_len: i32,
    pub far_match_offset_log2: i32,
}

#[cfg(test)]
mod abi_tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn test_compress_options_layout() {
        assert_eq!(size_of::<OodleStatus>(), 4);
        assert_eq!(size_of::<OodleProfile>(), 4);

        assert_eq!(size_of::<OodleCompressOptions>(), 48);
        assert_eq!(align_of::<OodleCompressOptions>(), 4);

        assert_eq!(offset_of!(OodleCompressOptions, min_match_len), 0);
        assert_eq!(offset_of!(OodleCompressOptions, seek_chunk_reset), 4);
        assert_eq!(offset_of!(OodleCompressOptions, seek_chunk_len), 8);
        assert_eq!(offset_of!(OodleCompressOptions, profile), 12);
        assert_eq!(offset_of!(OodleCompressOptions, dictionary_size), 16);
        assert_eq!(offset_of!(OodleCompressOptions, space_speed_tradeoff_bytes), 20);
        assert_eq!(offset_of!(OodleCompressOptions, send_quantum_crcs), 24);
        assert_eq!(offset_of!(OodleCompressOptions, max_local_dictionary_size), 28);
        assert_eq!(offset_of!(OodleCompressOptions, make_long_range_matcher), 32);
        assert_eq!(offset_of!(OodleCompressOptions, match_table_size_log2), 36);
        assert_eq!(offset_of!(OodleCompressOptions, far_match_min_len), 40);
        assert_eq!(offset_of!(OodleCompressOptions, far_match_offset_log2), 44);
    }

    #[test]
    fn test_status_discriminants() {
        assert_eq!(OodleStatus::Ok as i32, 0);
        assert_eq!(OodleStatus::InvalidArgument as i32, 1);
        assert_eq!(OodleStatus::DecompressionFailed as i32, 2);
        assert_eq!(OodleStatus::CompressionFailed as i32, 3);
        assert_eq!(OodleStatus::InvalidCompressedData as i32, 4);
        assert_eq!(OodleStatus::InvalidHeader as i32, 5);
        assert_eq!(OodleStatus::InternalError as i32, 255);
    }

    #[test]
    fn test_profile_discriminants() {
        assert_eq!(OodleProfile::Main as i32, 0);
        assert_eq!(OodleProfile::Reduced as i32, 1);
    }
}

/// Initializes compression options with their default values.
///
/// # Safety
///
/// - `options` must point to a writable `OodleCompressOptions`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oodle_compress_options_init(
    options: *mut OodleCompressOptions,
) -> OodleStatus {
    if options.is_null() {
        return OodleStatus::InvalidArgument;
    }

    let defaults = RustCompressOptions::new();

    let options_value = OodleCompressOptions {
        min_match_len: defaults.min_match_len(),
        seek_chunk_reset: u8::from(defaults.seek_chunk_reset()),
        seek_chunk_len: defaults.seek_chunk_len(),
        profile: match defaults.profile() {
            Profile::Main => OodleProfile::Main as i32,
            Profile::Reduced => OodleProfile::Reduced as i32,
        },
        dictionary_size: defaults.dictionary_size(),
        space_speed_tradeoff_bytes: defaults.space_speed_tradeoff_bytes(),
        send_quantum_crcs: u8::from(defaults.send_quantum_crcs()),
        max_local_dictionary_size: defaults.max_local_dictionary_size(),
        make_long_range_matcher: u8::from(defaults.make_long_range_matcher()),
        match_table_size_log2: defaults.match_table_size_log2(),
        far_match_min_len: defaults.far_match_min_len(),
        far_match_offset_log2: defaults.far_match_offset_log2(),
    };

    // SAFETY: The caller guarantees that the pointer is writable.
    unsafe {
        *options = options_value;
    }

    OodleStatus::Ok
}

/// Decompresses an Oodle-compressed buffer.
///
/// # Safety
///
/// - `compressed` must point to `compressed_size` readable bytes.
/// - `decompressed` must point to `decompressed_capacity` writable bytes.
/// - `decompressed_size` must point to a writable `usize`.
/// - The input and output buffers must not overlap.
/// - `decompressed_size` must not overlap any other memory passed to this function.
/// - All pointers must satisfy Rust's alignment and validity requirements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oodle_decompress(
    compressed: *const u8,
    compressed_size: usize,
    decompressed: *mut u8,
    decompressed_capacity: usize,
    decompressed_size: *mut usize,
    dictionary_len: usize,
) -> OodleStatus {
    if compressed.is_null()
        || decompressed.is_null()
        || decompressed_size.is_null()
        || compressed_size == 0
        || decompressed_capacity == 0
        || compressed_size > isize::MAX as usize
        || decompressed_capacity > isize::MAX as usize
        || dictionary_len >= decompressed_capacity
        || dictionary_len % crate::BLOCK_LEN != 0
    {
        return OodleStatus::InvalidArgument;
    }

    // SAFETY: The caller guarantees that this pointer is writable.
    unsafe {
        *decompressed_size = 0;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Pointer validity and buffer sizes are guaranteed by the caller.
        let compressed = unsafe {
            slice::from_raw_parts(compressed, compressed_size)
        };

        // SAFETY: Pointer validity and buffer sizes are guaranteed by the caller.
        let decompressed = unsafe {
            slice::from_raw_parts_mut(decompressed, decompressed_capacity)
        };

        decompress(compressed, decompressed, false, dictionary_len)
    }));

    match result {
        Ok(Ok(size)) => {
            // SAFETY: The caller guarantees that this pointer is writable.
            unsafe {
                *decompressed_size = size;
            }

            OodleStatus::Ok
        }
        Ok(Err(crate::error::Error::InvalidCompressedData(_))) => {
            OodleStatus::InvalidCompressedData
        }
        Ok(Err(crate::error::Error::InvalidHeader)) => {
            OodleStatus::InvalidHeader
        }
        Ok(Err(_)) => OodleStatus::DecompressionFailed,
        Err(_) => OodleStatus::InternalError,
    }
}

/// Returns the recommended compressed buffer capacity.
///
/// # Arguments
///
/// * `decompressed_size` - The size of the data to compress.
///
/// # Returns
///
/// The recommended buffer capacity in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn oodle_compress_bound(
    decompressed_size: usize,
) -> usize {
    get_compressed_buffer_size_hint(decompressed_size)
}

fn convert_compressor(value: i32) -> Option<Compressor> {
    match value {
        0 => Some(Compressor::Kraken),
        1 => Some(Compressor::Leviathan),
        2 => Some(Compressor::Mermaid),
        3 => Some(Compressor::Selkie),
        4 => Some(Compressor::Hydra),
        _ => None,
    }
}

fn convert_compression_level(value: i32) -> Option<CompressionLevel> {
    match value {
        0 => Some(CompressionLevel::SuperFast),
        1 => Some(CompressionLevel::VeryFast),
        2 => Some(CompressionLevel::Fast),
        3 => Some(CompressionLevel::Normal),
        4 => Some(CompressionLevel::Optimal1),
        5 => Some(CompressionLevel::Optimal2),
        6 => Some(CompressionLevel::Optimal3),
        7 => Some(CompressionLevel::Optimal4),
        8 => Some(CompressionLevel::Optimal5),
        9 => Some(CompressionLevel::HyperFast1),
        10 => Some(CompressionLevel::HyperFast2),
        11 => Some(CompressionLevel::HyperFast3),
        12 => Some(CompressionLevel::HyperFast4),
        _ => None,
    }
}

fn convert_profile(value: i32) -> Option<Profile> {
    match value {
        0 => Some(Profile::Main),
        1 => Some(Profile::Reduced),
        _ => None,
    }
}

fn convert_compress_options(
    options: OodleCompressOptions,
) -> Option<RustCompressOptions> {
    let seek_chunk_reset = match options.seek_chunk_reset {
        0 => false,
        1 => true,
        _ => return None,
    };

    let send_quantum_crcs = match options.send_quantum_crcs {
        0 => false,
        1 => true,
        _ => return None,
    };

    let make_long_range_matcher = match options.make_long_range_matcher {
        0 => false,
        1 => true,
        _ => return None,
    };

    let profile = convert_profile(options.profile)?;

    if options.seek_chunk_len < (1 << 18)
        || !(options.seek_chunk_len as u32).is_power_of_two()
    {
        return None;
    }

    if options.max_local_dictionary_size <= 0
        || options.max_local_dictionary_size > (1 << 30)
        || !(options.max_local_dictionary_size as u32).is_power_of_two()
    {
        return None;
    }

    Some(
        RustCompressOptions::builder()
            .min_match_len(options.min_match_len)
            .seek_chunk_reset(seek_chunk_reset)
            .seek_chunk_len(options.seek_chunk_len)
            .profile(profile)
            .dictionary_size(options.dictionary_size)
            .space_speed_tradeoff_bytes(options.space_speed_tradeoff_bytes)
            .send_quantum_crcs(send_quantum_crcs)
            .max_local_dictionary_size(options.max_local_dictionary_size)
            .make_long_range_matcher(make_long_range_matcher)
            .match_table_size_log2(options.match_table_size_log2)
            .far_match_min_len(options.far_match_min_len)
            .far_match_offset_log2(options.far_match_offset_log2)
            .build(),
    )
}

/// Compresses an Oodle buffer.
///
/// # Safety
///
/// - `decompressed` must point to `decompressed_size` readable bytes.
/// - `compressed` must point to `compressed_capacity` writable bytes.
/// - `compressed_size` must point to a writable `usize`.
/// - `options` must point to a readable `OodleCompressOptions` when non-null.
/// - The input and output buffers must not overlap.
/// - `compressed_size` must not overlap any other memory passed to this function.
/// - All pointers must satisfy Rust's alignment and validity requirements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oodle_compress(
    decompressed: *const u8,
    decompressed_size: usize,
    compressed: *mut u8,
    compressed_capacity: usize,
    compressed_size: *mut usize,
    compressor: i32,
    level: i32,
    options: *const OodleCompressOptions,
    dictionary_len: usize,
) -> OodleStatus {
    if compressed_size.is_null() {
        return OodleStatus::InvalidArgument;
    }

    // SAFETY: The caller guarantees that this pointer is writable.
    unsafe {
        *compressed_size = 0;
    }

    if decompressed.is_null()
        || compressed.is_null()
        || decompressed_size == 0
        || compressed_capacity == 0
        || decompressed_size > isize::MAX as usize
        || compressed_capacity > isize::MAX as usize
        || dictionary_len >= decompressed_size
        || dictionary_len % crate::BLOCK_LEN != 0
    {
        return OodleStatus::InvalidArgument;
    }

    let Some(compressor) = convert_compressor(compressor) else {
        return OodleStatus::InvalidArgument;
    };

    let Some(level) = convert_compression_level(level) else {
        return OodleStatus::InvalidArgument;
    };

    let compress_options = if options.is_null() {
        RustCompressOptions::new()
    } else {
        // SAFETY: The caller guarantees that the options pointer is readable.
        let options = unsafe { *options };

        let Some(options) = convert_compress_options(options) else {
            return OodleStatus::InvalidArgument;
        };

        options
    };

    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Pointer validity and buffer sizes are guaranteed by the caller.
        let decompressed = unsafe {
            slice::from_raw_parts(decompressed, decompressed_size)
        };

        // SAFETY: Pointer validity and buffer sizes are guaranteed by the caller.
        let compressed = unsafe {
            slice::from_raw_parts_mut(compressed, compressed_capacity)
        };

        compress(
            decompressed,
            compressed,
            compressor,
            level,
            compress_options,
            dictionary_len,
        )
    }));

    match result {
        Ok(Ok(size)) => {
            // SAFETY: The caller guarantees that this pointer is writable.
            unsafe {
                *compressed_size = size;
            }

            OodleStatus::Ok
        }
        Ok(Err(_)) => OodleStatus::CompressionFailed,
        Err(_) => OodleStatus::InternalError,
    }
}
