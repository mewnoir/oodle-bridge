
use std::hint::black_box;
use std::time::Duration;

use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};

use oodle_bridge::decompress;

unsafe extern "C" {
    #[link_name = "oodle_decompress"]
    fn ffi_oodle_decompress(
        compressed: *const u8,
        compressed_size: usize,
        decompressed: *mut u8,
        decompressed_capacity: usize,
        decompressed_size: *mut usize,
        dictionary_len: usize,
    ) -> i32;
}

fn rust_decompress(
    compressed: &[u8],
    decompressed: &mut [u8],
) -> usize {
    let size = decompress(
        compressed,
        decompressed,
        false,
        0,
    )
    .expect("Rust decompression failed");

    assert_eq!(size, decompressed.len());

    size
}

fn ffi_decompress(
    compressed: &[u8],
    decompressed: &mut [u8],
) -> usize {
    let mut decompressed_size = 0;

    let status = unsafe {
        ffi_oodle_decompress(
            compressed.as_ptr(),
            compressed.len(),
            decompressed.as_mut_ptr(),
            decompressed.len(),
            &mut decompressed_size,
            0,
        )
    };

    assert_eq!(status, 0, "Oodle FFI decompression failed");
    assert_eq!(decompressed_size, decompressed.len());

    decompressed_size
}

fn benchmark(c: &mut Criterion) {
    let compressed_data = std::fs::read("test-data/kraken/xml.kraken")
        .expect("Failed to read benchmark data");

    let (header_size, decompressed_len) =
        if compressed_data.get(4) == Some(&0x8C) {
            let size_bytes: [u8; 4] = compressed_data
                .get(..4)
                .expect("Invalid compressed file header")
                .try_into()
                .expect("Invalid compressed file header");

            (
                4,
                u32::from_le_bytes(size_bytes) as usize,
            )
        } else {
            let size_bytes: [u8; 8] = compressed_data
                .get(..8)
                .expect("Invalid compressed file header")
                .try_into()
                .expect("Invalid compressed file header");

            (
                8,
                usize::try_from(u64::from_le_bytes(size_bytes))
                    .expect("Decompressed size exceeds the platform limit"),
            )
        };

    let compressed = compressed_data
        .get(header_size..)
        .expect("Invalid compressed data");

    assert!(!compressed.is_empty(), "Compressed data is empty");
    assert!(decompressed_len > 0, "Decompressed size is zero");

    let mut decompressed = vec![0; decompressed_len];

    let mut group = c.benchmark_group("oodle");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("rust", |b| {
        b.iter(|| {
            black_box(rust_decompress(
                black_box(compressed),
                black_box(&mut decompressed),
            ))
        })
    });

    group.bench_function("ffi", |b| {
        b.iter(|| {
            black_box(ffi_decompress(
                black_box(compressed),
                black_box(&mut decompressed),
            ))
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
