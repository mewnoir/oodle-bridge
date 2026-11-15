# oodle-bridge

A cross-platform Rust library and C-compatible interface for Oodle data compression and decompression.

`oodle-bridge` provides a Rust API and a C ABI for working with Oodle compression algorithms, backed by the native Oodle source code.

## Features

- Rust API for compression and decompression.
- C-compatible API for integration with C and C++ projects.
- Native compilation for supported platforms.
- Support for Kraken, Leviathan, Mermaid, and Selkie compression.
- Configurable compression options and compression levels.

## Requirements

Building the library requires:

- Rust toolchain (Edition 2024).
- A supported C++ compiler with C++20 support.
- The Oodle SDK source code.
- `libclang` for generating Rust bindings with `bindgen`.

The Oodle SDK source code is not included in this repository. See [Development](#development) for details.

## Building

Once the required Oodle source code is available in the expected `oodle/src` directory, build the library with:

```bash
cargo build --release
```

To run the test suite:

```bash
cargo test --lib --tests
```

To compile the benchmark:

```bash
cargo bench --bench decompress --no-run
```

## Development

### Oodle source code

The Oodle source code is not distributed with this project. You must obtain it separately through an authorized source and place it in the expected `oodle/src` directory before building.

This repository includes a Nix development environment that can be used to prepare the required source tree. The downloaded Oodle files are local build dependencies and are not part of the project's source distribution.

### Test data

The integration tests use the separate [oodle-test-data](https://github.com/sehnryr/oodle-test-data) repository.

Since the test data repository can be relatively large, a partial clone with sparse checkout can reduce the amount of data downloaded.

After entering the `test-data` submodule directory, run:

<details>
<summary>Bash</summary>

```bash
git clone \
    --depth 1 \
    --filter blob:none \
    --sparse \
    https://github.com/sehnryr/oodle-test-data.git .

git sparse-checkout add raw kraken leviathan mermaid selkie
```

</details>

<details>
<summary>Nushell</summary>

```nu
(git clone
    --depth 1
    --filter blob:none
    --sparse
    https://github.com/sehnryr/oodle-test-data.git .)

["raw", "kraken", "leviathan", "mermaid", "selkie"] | each { |name|
    git sparse-checkout add $name
}
```

</details>

The test data is only required for the integration tests that use those files. It is not required to build the library.

## Licensing

### oodle-bridge

The original code in this repository is licensed under either:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

You may choose either license when using the project's original code.

### Oodle SDK

Oodle is proprietary software developed by RAD Game Tools, now part of Epic Games. Its source code, headers, libraries, and other SDK components are **not covered by the licenses of this project**.

This repository does not grant any rights to use, modify, or redistribute Oodle. Users are responsible for obtaining the SDK through an authorized source and complying with its applicable license terms.

In particular:

- Do not assume that publicly accessible Oodle source code is freely redistributable.
- Do not redistribute Oodle source files, headers, or compiled binaries as part of this project without explicit authorization.
- Verify that your intended use, including commercial distribution, is permitted by your Oodle license.

For licensing questions or commercial use, contact [RAD Game Tools](https://www.radgametools.com/sales.htm).

**The MIT and Apache-2.0 licenses apply only to the original code in `oodle-bridge`, not to Oodle or any third-party components.**

## Disclaimer

This project is an independent integration and is not affiliated with, endorsed by, or sponsored by RAD Game Tools or Epic Games.
