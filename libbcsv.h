/// EXPORTS FOR FEATURE c_exports, THIS IS NOT FOR THE cxx FEATURE!!!
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdio.h>
#include <stdint.h>

struct PtrInfo {
    unsigned char* ptr;
    size_t len;
    ~PtrInfo() { free_PtrInfo(*this); }
};

const char* bcsv_to_csv(const char*, unsigned char*, size_t, unsigned char);
PtrInfo csv_to_bcsv(const char*, unsigned char, unsigned int mask = UINT32_MAX);
void free_PtrInfo(PtrInfo);
void bcsv_to_xlsx(const char*, unsigned char*, const char*, size_t, unsigned char);

#ifdef __cplusplus
}
#endif