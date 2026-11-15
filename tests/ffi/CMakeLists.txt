
cmake_minimum_required(VERSION 3.20)

project(oodle_bridge_ffi_tests LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

get_filename_component(
    PROJECT_ROOT
    "${CMAKE_CURRENT_SOURCE_DIR}/../.."
    ABSOLUTE
)

if(NOT DEFINED OODLE_BUILD_DIR)
    set(OODLE_BUILD_DIR "${PROJECT_ROOT}/target/debug")
endif()

if(WIN32)
    set(OODLE_LIBRARY "${OODLE_BUILD_DIR}/oodle_bridge.dll")
    set(OODLE_IMPORT_LIBRARY "${OODLE_BUILD_DIR}/oodle_bridge.dll.lib")
elseif(APPLE)
    set(OODLE_LIBRARY "${OODLE_BUILD_DIR}/liboodle_bridge.dylib")
elseif(UNIX)
    set(OODLE_LIBRARY "${OODLE_BUILD_DIR}/liboodle_bridge.so")
else()
    message(FATAL_ERROR "Unsupported platform.")
endif()

if(NOT EXISTS "${OODLE_LIBRARY}")
    message(FATAL_ERROR "Missing Oodle library: ${OODLE_LIBRARY}")
endif()

add_library(oodle_bridge SHARED IMPORTED GLOBAL)

if(WIN32)
    if(NOT EXISTS "${OODLE_IMPORT_LIBRARY}")
        message(FATAL_ERROR "Missing Oodle import library: ${OODLE_IMPORT_LIBRARY}")
    endif()

    set_target_properties(oodle_bridge PROPERTIES
        IMPORTED_LOCATION "${OODLE_LIBRARY}"
        IMPORTED_IMPLIB "${OODLE_IMPORT_LIBRARY}"
        INTERFACE_INCLUDE_DIRECTORIES "${PROJECT_ROOT}/include"
    )
else()
    set_target_properties(oodle_bridge PROPERTIES
        IMPORTED_LOCATION "${OODLE_LIBRARY}"
        INTERFACE_INCLUDE_DIRECTORIES "${PROJECT_ROOT}/include"
    )
endif()

add_executable(oodle_ffi_test main.cpp)

if(MSVC)
    target_compile_options(oodle_ffi_test PRIVATE /EHsc)
endif()

target_link_libraries(oodle_ffi_test PRIVATE oodle_bridge)

add_custom_command(TARGET oodle_ffi_test POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${OODLE_LIBRARY}"
        "$<TARGET_FILE_DIR:oodle_ffi_test>"
)

enable_testing()

set(POKEMON_TEST_DATA "${PROJECT_ROOT}/pokemon-test-data")

set(POKEMON_TEST_FILES
    "pm0025_00_00_00280_sleep01_start.tranm"
    "pm0025_00_00_00522_down01_end.tranm"
    "pm0025_00_00_body_b_nrm.bntx"
)

set(POKEMON_TEST_NAMES
    "pokemon_sleep01_start"
    "pokemon_down01_end"
    "pokemon_body_b_nrm"
)

list(LENGTH POKEMON_TEST_FILES POKEMON_TEST_COUNT)
math(EXPR POKEMON_TEST_LAST "${POKEMON_TEST_COUNT} - 1")

foreach(INDEX RANGE ${POKEMON_TEST_LAST})
    list(GET POKEMON_TEST_FILES ${INDEX} TEST_FILE)
    list(GET POKEMON_TEST_NAMES ${INDEX} TEST_NAME)

    set(COMPRESSED_FILE "${POKEMON_TEST_DATA}/compressed/${TEST_FILE}")
    set(EXPECTED_FILE "${POKEMON_TEST_DATA}/decompressed/${TEST_FILE}")

    if(EXISTS "${COMPRESSED_FILE}" AND EXISTS "${EXPECTED_FILE}")
        add_test(
            NAME ${TEST_NAME}
            COMMAND oodle_ffi_test
                "${COMPRESSED_FILE}"
                "${EXPECTED_FILE}"
        )
    endif()
endforeach()
