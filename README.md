# oodle-bridge

A C-compatible fork of [oodle-rs](https://github.com/ymelois/oodle-rs), bringing its Oodle compression and decompression functionality to C and C++ applications.

Originally developed by Youn Mélois, `oodle-rs` provides a Rust implementation of Oodle. This fork retains its Rust API while adding a C ABI for native integrations.

## Features

- Rust API for compression and decompression.
- C ABI for C and C++ integration.
- Kraken, Leviathan, Mermaid, and Selkie.
- Configurable compression levels and options.
- Cross-platform builds.

## Requirements

- Rust toolchain (Edition 2024).
- C++20 compatible compiler.
- Oodle SDK source code.
- `libclang` for `bindgen`.

The Oodle source code must be obtained separately and placed in the expected `oodle/src` directory. It is not included in this repository.

## Building

```bash
cargo build --release
```

Run the tests:

```bash
cargo test --lib --tests
```

Compile the benchmark:

```bash
cargo bench --bench decompress --no-run
```

## Usage

### Rust

```rust
use oodle_bridge::{
    CompressOptions, CompressionLevel, Compressor,
    compress, decompress, get_compressed_buffer_size_hint,
};

fn main() {
    let input = b"Hello from oodle-bridge!";
    let mut compressed = vec![0; get_compressed_buffer_size_hint(input.len())];

    let size = compress(
        input,
        &mut compressed,
        Compressor::Kraken,
        CompressionLevel::Normal,
        CompressOptions::default(),
        0,
    ).expect("Compression failed");

    compressed.truncate(size);

    let mut decompressed = vec![0; input.len()];

    let size = decompress(
        &compressed,
        &mut decompressed,
        false,
        0,
    ).expect("Decompression failed");

    decompressed.truncate(size);
    assert_eq!(&input[..], &decompressed[..]);
}
```

### C / C++

Include `include/oodle_bridge.h` and link against the compiled `oodle-bridge` library.

```cpp
#include "oodle_bridge.h"

#include <cstdint>
#include <cstring>
#include <vector>

int main() {
    constexpr char input[] = "Hello from oodle-bridge!";
    const auto* input_data =
        reinterpret_cast<const std::uint8_t*>(input);
    const std::size_t input_size = sizeof(input);

    OodleCompressOptions options;

    if (oodle_compress_options_init(&options) != OODLE_OK)
        return 1;

    std::vector<std::uint8_t> compressed(
        oodle_compress_bound(input_size)
    );

    std::size_t compressed_size = 0;

    if (oodle_compress(
        input_data,
        input_size,
        compressed.data(),
        compressed.size(),
        &compressed_size,
        OODLE_COMPRESSOR_KRAKEN,
        OODLE_LEVEL_NORMAL,
        &options,
        0
    ) != OODLE_OK)
        return 1;

    compressed.resize(compressed_size);

    std::vector<std::uint8_t> decompressed(input_size);
    std::size_t decompressed_size = 0;

    if (oodle_decompress(
        compressed.data(),
        compressed.size(),
        decompressed.data(),
        decompressed.size(),
        &decompressed_size,
        0
    ) != OODLE_OK)
        return 1;

    return decompressed_size == input_size &&
        std::memcmp(input_data, decompressed.data(), input_size) == 0
        ? 0 : 1;
}
```

Both examples use a dictionary length of `0`, meaning no dictionary is provided.

## Development

The repository includes a Nix development environment for preparing the required Oodle source tree.

Integration tests use the separate [oodle-test-data](https://github.com/sehnryr/oodle-test-data) repository. Its test data is optional for building the library but required for the corresponding integration tests.

## Acknowledgements

Based on the original Rust implementation by **Youn Mélois**:

- [oodle-rs](https://github.com/ymelois/oodle-rs)

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for third-party attribution and license notices.

## Licensing

### oodle-bridge

Original project code is licensed under either:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

Portions derived from `oodle-rs` remain subject to their applicable third-party license terms.

### Oodle SDK

Oodle is proprietary software developed by RAD Game Tools, now part of Epic Games. Its source code, headers, libraries, and binaries are **not covered by this project's licenses**.

This repository does not grant permission to use or redistribute Oodle. You must obtain the SDK through an authorized source and comply with its license terms.

In particular, do not redistribute Oodle source code, headers, or compiled binaries without explicit authorization. Verify that your intended use, including commercial distribution, is permitted.

For licensing questions, contact [RAD Game Tools](https://www.radgametools.com/sales.htm).

### Disclaimer

This project is an independent integration and is not affiliated with, endorsed by, or sponsored by RAD Game Tools or Epic Games.
