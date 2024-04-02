/// EXPORTS FOR FEATURE c_exports, THIS IS NOT FOR THE cxx FEATURE!!!
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdio.h>
#include <stdint.h>

struct PtrInfo;

void free_PtrInfo(PtrInfo);

struct PtrInfo {
    unsigned char* ptr;
    size_t len;
    ~PtrInfo() { free_PtrInfo(*this); }
};

PtrInfo bcsv_to_csv(const char*, const uint8_t*, size_t, uint8_t);
PtrInfo csv_to_bcsv(const char*, uint8_t);
void bcsv_to_xlsx(const char*, const char*, const uint8_t*, size_t, uint8_t);

#ifdef __cplusplus
}
#endif