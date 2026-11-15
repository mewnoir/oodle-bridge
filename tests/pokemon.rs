
use std::path::PathBuf;

fn verify_decompression(filename: &str) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pokemon-test-data");

    let compressed_path = root
        .join("compressed")
        .join(filename);

    let expected_path = root
        .join("decompressed")
        .join(filename);

    let compressed = std::fs::read(&compressed_path)
        .unwrap_or_else(|error| {
            panic!("Failed to read compressed file {filename}: {error}");
        });

    let expected = std::fs::read(&expected_path)
        .unwrap_or_else(|error| {
            panic!("Failed to read reference file {filename}: {error}");
        });

    let mut output = vec![0u8; expected.len()];

    let size = oodle_bridge::decompress(
        &compressed,
        &mut output,
        false,
        0,
    )
    .unwrap_or_else(|error| {
        panic!("Decompression failed for {filename}: {error:?}");
    });

    assert_eq!(size, expected.len());
    assert_eq!(output, expected);
}

#[test]
#[ignore = "Requires local Pokemon test data"]
fn decompress_tranm_sleep01_start() {
    verify_decompression("pm0025_00_00_00280_sleep01_start.tranm");
}

#[test]
#[ignore = "Requires local Pokemon test data"]
fn decompress_tranm_down01_end() {
    verify_decompression("pm0025_00_00_00522_down01_end.tranm");
}

#[test]
#[ignore = "Requires local Pokemon test data"]
fn decompress_bntx_body_normal() {
    verify_decompression("pm0025_00_00_body_b_nrm.bntx");
}
