
#ifndef OODLE_BRIDGE_H
#define OODLE_BRIDGE_H

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32)
    #if defined(OODLE_BRIDGE_BUILD)
        #define OODLE_API __declspec(dllexport)
    #else
        #define OODLE_API __declspec(dllimport)
    #endif
#else
    #define OODLE_API __attribute__((visibility("default")))
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef enum OodleStatus {
    OODLE_OK = 0,
    OODLE_INVALID_ARGUMENT = 1,
    OODLE_DECOMPRESSION_FAILED = 2,
    OODLE_COMPRESSION_FAILED = 3,
    OODLE_INVALID_COMPRESSED_DATA = 4,
    OODLE_INVALID_HEADER = 5,
    OODLE_INTERNAL_ERROR = 255
} OodleStatus;

typedef enum OodleCompressor {
    OODLE_COMPRESSOR_KRAKEN = 0,
    OODLE_COMPRESSOR_LEVIATHAN = 1,
    OODLE_COMPRESSOR_MERMAID = 2,
    OODLE_COMPRESSOR_SELKIE = 3,
    OODLE_COMPRESSOR_HYDRA = 4
} OodleCompressor;

typedef enum OodleCompressionLevel {
    OODLE_LEVEL_SUPER_FAST = 0,
    OODLE_LEVEL_VERY_FAST = 1,
    OODLE_LEVEL_FAST = 2,
    OODLE_LEVEL_NORMAL = 3,
    OODLE_LEVEL_OPTIMAL1 = 4,
    OODLE_LEVEL_OPTIMAL2 = 5,
    OODLE_LEVEL_OPTIMAL3 = 6,
    OODLE_LEVEL_OPTIMAL4 = 7,
    OODLE_LEVEL_OPTIMAL5 = 8,
    OODLE_LEVEL_HYPER_FAST1 = 9,
    OODLE_LEVEL_HYPER_FAST2 = 10,
    OODLE_LEVEL_HYPER_FAST3 = 11,
    OODLE_LEVEL_HYPER_FAST4 = 12
} OodleCompressionLevel;

typedef enum OodleProfile {
    OODLE_PROFILE_MAIN = 0,
    OODLE_PROFILE_REDUCED = 1
} OodleProfile;

typedef struct OodleCompressOptions {
    int32_t min_match_len;
    uint8_t seek_chunk_reset;
    int32_t seek_chunk_len;
    int32_t profile;
    int32_t dictionary_size;
    int32_t space_speed_tradeoff_bytes;
    uint8_t send_quantum_crcs;
    int32_t max_local_dictionary_size;
    uint8_t make_long_range_matcher;
    int32_t match_table_size_log2;
    int32_t far_match_min_len;
    int32_t far_match_offset_log2;
} OodleCompressOptions;

OODLE_API size_t oodle_compress_bound(size_t decompressed_size);

OODLE_API OodleStatus oodle_compress_options_init(
    OodleCompressOptions* options
);

OODLE_API OodleStatus oodle_compress(
    const uint8_t* decompressed,
    size_t decompressed_size,
    uint8_t* compressed,
    size_t compressed_capacity,
    size_t* compressed_size,
    int32_t compressor,
    int32_t level,
    const OodleCompressOptions* options,
    size_t dictionary_len
);

OODLE_API OodleStatus oodle_decompress(
    const uint8_t* compressed,
    size_t compressed_size,
    uint8_t* decompressed,
    size_t decompressed_capacity,
    size_t* decompressed_size,
    size_t dictionary_len
);

#ifdef __cplusplus
}
#endif

#endif
