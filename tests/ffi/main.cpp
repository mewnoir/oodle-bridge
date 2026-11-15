
#include "oodle_bridge.h"

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <vector>

static_assert(sizeof(OodleStatus) == 4);
static_assert(sizeof(OodleCompressor) == 4);
static_assert(sizeof(OodleCompressionLevel) == 4);
static_assert(sizeof(OodleProfile) == 4);

static_assert(sizeof(OodleCompressOptions) == 48);
static_assert(alignof(OodleCompressOptions) == 4);

static_assert(offsetof(OodleCompressOptions, min_match_len) == 0);
static_assert(offsetof(OodleCompressOptions, seek_chunk_reset) == 4);
static_assert(offsetof(OodleCompressOptions, seek_chunk_len) == 8);
static_assert(offsetof(OodleCompressOptions, profile) == 12);
static_assert(offsetof(OodleCompressOptions, dictionary_size) == 16);
static_assert(offsetof(OodleCompressOptions, space_speed_tradeoff_bytes) == 20);
static_assert(offsetof(OodleCompressOptions, send_quantum_crcs) == 24);
static_assert(offsetof(OodleCompressOptions, max_local_dictionary_size) == 28);
static_assert(offsetof(OodleCompressOptions, make_long_range_matcher) == 32);
static_assert(offsetof(OodleCompressOptions, match_table_size_log2) == 36);
static_assert(offsetof(OodleCompressOptions, far_match_min_len) == 40);
static_assert(offsetof(OodleCompressOptions, far_match_offset_log2) == 44);

static bool ReadFile(
    const char* path,
    std::vector<uint8_t>& data
)
{
    std::ifstream file(path, std::ios::binary);

    if (!file)
    {
        std::cerr << "Failed to open file: " << path << '\n';
        return false;
    }

    data.assign(
        std::istreambuf_iterator<char>(file),
        std::istreambuf_iterator<char>()
    );

    return true;
}

int main(int argc, char* argv[])
{
    if (argc != 3)
    {
        std::cerr
            << "Usage: oodle_ffi_test <compressed_file> <expected_file>\n";
        return 1;
    }

    std::vector<uint8_t> compressed;
    std::vector<uint8_t> expected;

    if (!ReadFile(argv[1], compressed) ||
        !ReadFile(argv[2], expected))
    {
        return 1;
    }

    if (compressed.empty() || expected.empty())
    {
        std::cerr << "Input files must not be empty.\n";
        return 1;
    }

    // Verify the original compressed file.
    std::vector<uint8_t> decompressed(expected.size());
    size_t decompressed_size = 0;

    const OodleStatus decompress_status = oodle_decompress(
        compressed.data(),
        compressed.size(),
        decompressed.data(),
        decompressed.size(),
        &decompressed_size,
        0
    );

    if (decompress_status != OODLE_OK ||
        decompressed_size != expected.size() ||
        decompressed != expected)
    {
        std::cerr << "Original decompression test failed.\n";
        return 1;
    }

    OodleCompressOptions options{};

    if (oodle_compress_options_init(&options) != OODLE_OK)
    {
        std::cerr << "Failed to initialize compression options.\n";
        return 1;
    }

    const OodleCompressor compressors[] = {
        OODLE_COMPRESSOR_KRAKEN,
        OODLE_COMPRESSOR_LEVIATHAN,
        OODLE_COMPRESSOR_MERMAID,
        OODLE_COMPRESSOR_SELKIE,
        OODLE_COMPRESSOR_HYDRA
    };

    const OodleCompressionLevel levels[] = {
        OODLE_LEVEL_SUPER_FAST,
        OODLE_LEVEL_VERY_FAST,
        OODLE_LEVEL_FAST,
        OODLE_LEVEL_NORMAL,
        OODLE_LEVEL_OPTIMAL1,
        OODLE_LEVEL_OPTIMAL2,
        OODLE_LEVEL_OPTIMAL3,
        OODLE_LEVEL_OPTIMAL4,
        OODLE_LEVEL_OPTIMAL5,
        OODLE_LEVEL_HYPER_FAST1,
        OODLE_LEVEL_HYPER_FAST2,
        OODLE_LEVEL_HYPER_FAST3,
        OODLE_LEVEL_HYPER_FAST4
    };

    const size_t compressed_capacity = oodle_compress_bound(expected.size());

    if (compressed_capacity == 0)
    {
        std::cerr << "Invalid compression buffer size.\n";
        return 1;
    }

    size_t successful_combinations = 0;

    for (const OodleCompressor compressor : compressors)
    {
        for (const OodleCompressionLevel level : levels)
        {
            std::vector<uint8_t> compressed_output(compressed_capacity);
            size_t compressed_output_size = 0;

            const OodleStatus compress_status = oodle_compress(
                expected.data(),
                expected.size(),
                compressed_output.data(),
                compressed_output.size(),
                &compressed_output_size,
                compressor,
                level,
                &options,
                0
            );

            if (compress_status != OODLE_OK)
            {
                std::cerr
                    << "Compression failed. Compressor: "
                    << static_cast<int>(compressor)
                    << ", level: "
                    << static_cast<int>(level)
                    << ", status: "
                    << static_cast<int>(compress_status)
                    << '\n';
                return 1;
            }

            if (compressed_output_size == 0 ||
                compressed_output_size > compressed_output.size())
            {
                std::cerr << "Invalid compressed output size.\n";
                return 1;
            }

            compressed_output.resize(compressed_output_size);

            std::vector<uint8_t> roundtrip(expected.size());
            size_t roundtrip_size = 0;

            const OodleStatus roundtrip_status = oodle_decompress(
                compressed_output.data(),
                compressed_output.size(),
                roundtrip.data(),
                roundtrip.size(),
                &roundtrip_size,
                0
            );

            if (roundtrip_status != OODLE_OK ||
                roundtrip_size != expected.size() ||
                roundtrip != expected)
            {
                std::cerr
                    << "Round-trip failed. Compressor: "
                    << static_cast<int>(compressor)
                    << ", level: "
                    << static_cast<int>(level)
                    << ", status: "
                    << static_cast<int>(roundtrip_status)
                    << '\n';
                return 1;
            }

            ++successful_combinations;
        }
    }

    // Test custom compression options.
    auto test_options = [&](const char* name, const OodleCompressOptions& test_options) -> bool
    {
        std::vector<uint8_t> compressed_output(compressed_capacity);
        size_t compressed_output_size = 0;

        const OodleStatus compress_status = oodle_compress(
            expected.data(),
            expected.size(),
            compressed_output.data(),
            compressed_output.size(),
            &compressed_output_size,
            OODLE_COMPRESSOR_KRAKEN,
            OODLE_LEVEL_NORMAL,
            &test_options,
            0
        );

        if (compress_status != OODLE_OK)
        {
            std::cerr << "Custom compression failed: " << name
                      << ", status: " << static_cast<int>(compress_status)
                      << '\n';
            return false;
        }

        if (compressed_output_size == 0 ||
            compressed_output_size > compressed_output.size())
        {
            std::cerr << "Invalid compressed size: " << name << '\n';
            return false;
        }

        compressed_output.resize(compressed_output_size);

        std::vector<uint8_t> roundtrip(expected.size());
        size_t roundtrip_size = 0;

        const OodleStatus decompress_status = oodle_decompress(
            compressed_output.data(),
            compressed_output.size(),
            roundtrip.data(),
            roundtrip.size(),
            &roundtrip_size,
            0
        );

        if (decompress_status != OODLE_OK ||
            roundtrip_size != expected.size() ||
            roundtrip != expected)
        {
            std::cerr << "Custom round-trip failed: " << name
                      << ", status: " << static_cast<int>(decompress_status)
                      << '\n';
            return false;
        }

        std::cout << "Custom options passed: " << name << '\n';
        return true;
    };

    OodleCompressOptions custom_options = options;

    custom_options.min_match_len = 6;
    if (!test_options("min_match_len", custom_options))
        return 1;

    custom_options = options;
    custom_options.dictionary_size = 1 << 20;
    if (!test_options("dictionary_size", custom_options))
        return 1;

    custom_options = options;
    custom_options.space_speed_tradeoff_bytes = 512;
    if (!test_options("space_speed_tradeoff_bytes", custom_options))
        return 1;

    custom_options = options;
    custom_options.profile = OODLE_PROFILE_REDUCED;
    if (!test_options("reduced_profile", custom_options))
        return 1;

    custom_options = options;
    custom_options.seek_chunk_reset = 1;
    custom_options.seek_chunk_len = 1 << 18;
    if (!test_options("seek_chunk_reset", custom_options))
        return 1;

    custom_options = options;
    custom_options.send_quantum_crcs = 1;
    if (!test_options("send_quantum_crcs", custom_options))
        return 1;

    custom_options = options;
    custom_options.max_local_dictionary_size = 1 << 20;
    custom_options.make_long_range_matcher = 0;
    if (!test_options("local_dictionary", custom_options))
        return 1;

    custom_options = options;
    custom_options.match_table_size_log2 = 20;
    if (!test_options("match_table_size_log2", custom_options))
        return 1;

    custom_options = options;
    custom_options.far_match_min_len = 6;
    custom_options.far_match_offset_log2 = 20;
    if (!test_options("far_match", custom_options))
        return 1;

    // Test dictionary compression and decompression.
    constexpr size_t dictionary_size = 1 << 18;
    constexpr size_t payload_size = 1 << 20;
    constexpr size_t input_size = dictionary_size + payload_size;

    std::vector<uint8_t> dictionary(dictionary_size);
    std::vector<uint8_t> payload(payload_size);
    std::vector<uint8_t> input(input_size);

    for (size_t i = 0; i < dictionary_size; ++i)
        dictionary[i] = static_cast<uint8_t>((i * 31 + 17) & 0xFF);

    for (size_t i = 0; i < payload_size; ++i)
        payload[i] = dictionary[i % dictionary_size];

    std::copy(dictionary.begin(), dictionary.end(), input.begin());
    std::copy(payload.begin(), payload.end(), input.begin() + dictionary_size);

    OodleCompressOptions dictionary_options = options;
    dictionary_options.dictionary_size = static_cast<int32_t>(dictionary_size);

    const size_t dictionary_compressed_capacity =
        oodle_compress_bound(payload_size);

    std::vector<uint8_t> dictionary_compressed(dictionary_compressed_capacity);
    size_t dictionary_compressed_size = 0;

    const OodleStatus dictionary_compress_status = oodle_compress(
        input.data(),
        input.size(),
        dictionary_compressed.data(),
        dictionary_compressed.size(),
        &dictionary_compressed_size,
        OODLE_COMPRESSOR_KRAKEN,
        OODLE_LEVEL_NORMAL,
        &dictionary_options,
        dictionary_size
    );

    if (dictionary_compress_status != OODLE_OK ||
        dictionary_compressed_size == 0 ||
        dictionary_compressed_size > dictionary_compressed.size())
    {
        std::cerr << "Dictionary compression failed.\n";
        return 1;
    }

    dictionary_compressed.resize(dictionary_compressed_size);

    std::vector<uint8_t> dictionary_roundtrip(input_size);
    std::copy(dictionary.begin(), dictionary.end(), dictionary_roundtrip.begin());

    size_t dictionary_roundtrip_size = 0;

    const OodleStatus dictionary_decompress_status = oodle_decompress(
        dictionary_compressed.data(),
        dictionary_compressed.size(),
        dictionary_roundtrip.data(),
        dictionary_roundtrip.size(),
        &dictionary_roundtrip_size,
        dictionary_size
    );

    if (dictionary_decompress_status != OODLE_OK ||
        dictionary_roundtrip_size != input_size ||
        !std::equal(
            payload.begin(),
            payload.end(),
            dictionary_roundtrip.begin() + dictionary_size))
    {
        std::cerr << "Dictionary round-trip failed. Status: "
                  << static_cast<int>(dictionary_decompress_status)
                  << ", output size: " << dictionary_roundtrip_size
                  << '\n';
        return 1;
    }

    std::cout << "Dictionary round-trip passed.\n";

    // Test invalid compression options.
    auto test_invalid_options = [&](const char* name, const OodleCompressOptions& test_options) -> bool
    {
        std::vector<uint8_t> output(compressed_capacity);
        size_t output_size = 123;

        const OodleStatus status = oodle_compress(
            expected.data(),
            expected.size(),
            output.data(),
            output.size(),
            &output_size,
            OODLE_COMPRESSOR_KRAKEN,
            OODLE_LEVEL_NORMAL,
            &test_options,
            0
        );

        if (status != OODLE_INVALID_ARGUMENT || output_size != 0)
        {
            std::cerr << "Invalid options test failed: " << name
                      << ", status: " << static_cast<int>(status)
                      << ", output size: " << output_size << '\n';
            return false;
        }

        std::cout << "Invalid options rejected: " << name << '\n';
        return true;
    };

    custom_options = options;
    custom_options.profile = 99;
    if (!test_invalid_options("invalid_profile", custom_options))
        return 1;

    custom_options = options;
    custom_options.seek_chunk_reset = 2;
    if (!test_invalid_options("invalid_seek_chunk_reset", custom_options))
        return 1;

    custom_options = options;
    custom_options.send_quantum_crcs = 2;
    if (!test_invalid_options("invalid_send_quantum_crcs", custom_options))
        return 1;

    custom_options = options;
    custom_options.make_long_range_matcher = 2;
    if (!test_invalid_options("invalid_make_long_range_matcher", custom_options))
        return 1;

    custom_options = options;
    custom_options.seek_chunk_len = 1 << 17;
    if (!test_invalid_options("invalid_seek_chunk_len", custom_options))
        return 1;

    custom_options = options;
    custom_options.seek_chunk_len = 300000;
    if (!test_invalid_options("non_power_of_two_seek_chunk_len", custom_options))
        return 1;

    custom_options = options;
    custom_options.max_local_dictionary_size = 3;
    if (!test_invalid_options("invalid_local_dictionary_size", custom_options))
        return 1;

    custom_options = options;
    custom_options.max_local_dictionary_size = 1 << 31;
    if (!test_invalid_options("oversized_local_dictionary", custom_options))
        return 1;

    std::cout << "Original decompression successful.\n";
    std::cout << "Compression combinations passed: "
              << successful_combinations << '\n';
    std::cout << "All round-trip tests passed.\n";

    return 0;
}
