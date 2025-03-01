/// EXPORTS FOR FEATURE c_exports, THIS IS NOT FOR THE cxx FEATURE!!!
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdio.h>
#include <stdint.h>

struct ManagedBuffer {
    uint8_t *buffer;
    uintptr_t len;
};

void free_managed_buffer(const ManagedBuffer *buffer);

const ManagedBuffer* bcsv_to_csv(const uint8_t* data, uintptr_t len, const int8_t* hash_path,
    bool is_signed, uint8_t endian, uint8_t delim);

void bcsv_to_xlsx(const int8_t* hash_path, const int8_t* output_path, const uint8_t* data, uintptr_t len,
    bool is_signed, uint8_t endian);

const ManagedBuffer* csv_to_bcsv(const int8_t* path, uint8_t endian, uint8_t delim);

#ifdef __cplusplus
}
#endif