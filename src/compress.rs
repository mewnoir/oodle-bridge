
use core::ptr;

use crate::bindings::root::oo2::OodleLZ_Compress;
use crate::error::{Error, Result};
use crate::model::{CompressOptions, CompressionLevel, Compressor};

/// Compress some data from memory to memory synchronously.
///
/// # Arguments
///
/// * `decompressed` - The data to be compressed.
/// * `compressed` - The buffer to store the compressed data.
/// * `compressor` - The compressor to use.
/// * `level` - The compression level to use.
/// * `compress_options` - The compression options to use. See
///   [`CompressOptions`] for more details.
/// * `dictionary_len` - The length of the base dictionary.
///
/// # Returns
///
/// The size of the compressed data.
///
/// # Errors
///
/// Returns an error if the dictionary length is invalid or compression fails.
///
/// # Notes
///
/// A dictionary base can be provided within the `decompressed` buffer
/// at `[0..dictionary_len]`.
pub fn compress(
    decompressed: &[u8],
    compressed: &mut [u8],
    compressor: Compressor,
    level: CompressionLevel,
    compress_options: CompressOptions,
    dictionary_len: usize,
) -> Result<usize> {
    if dictionary_len >= decompressed.len() {
        return Err(Error::InvalidInput(
            "dictionary length must be smaller than the input size",
        ));
    }

    let dictionary_base = &decompressed[..dictionary_len];
    let decompressed = &decompressed[dictionary_len..];

    let n = unsafe {
        OodleLZ_Compress(
            compressor.into(),
            decompressed.as_ptr().cast(),
            decompressed.len().cast_signed(),
            compressed.as_mut_ptr().cast(),
            level.into(),
            &compress_options.into(),
            dictionary_base.as_ptr().cast(),
            ptr::null(),
            ptr::null_mut(),
            0,
        )
    };

    if n <= 0 {
        return Err(Error::CompressionFailed);
    }

    Ok(n.cast_unsigned())
}
