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

PtrInfo bcsv_to_csv(const char*, const unsigned char*, size_t, unsigned char);
PtrInfo csv_to_bcsv(const char*, unsigned char, unsigned int);
void bcsv_to_xlsx(const char*, const unsigned char*, const char*, size_t, unsigned char);

#ifdef __cplusplus
}
#endif