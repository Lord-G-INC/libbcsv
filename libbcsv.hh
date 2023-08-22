// EXPORTS FOR FEATURE cxx, THIS IS NOT FOR THE c_exports FEATURE!!!
#pragma once
#ifndef __cplusplus
#error libbcsv.hh must be included in a C++ file!
#endif
#include <cstdint>
#include <string>
#include <vector>

// Dll Exports

extern "C" {
    void libbcsv$cxxbridge1$bcsv_to_csv(std::string &, std::vector<std::uint8_t> const &, 
    std::uint8_t) noexcept;
    void libbcsv$cxxbridge1$csv_to_bcsv(std::string const &, std::uint8_t, 
    std::vector<std::uint8_t> &, std::uint32_t) noexcept;
}

namespace libbcsv {
void bcsv_to_csv(std::string &path, std::vector<std::uint8_t> const &data, std::uint8_t endian) 
    noexcept {libbcsv$cxxbridge1$bcsv_to_csv(path, data, endian);}

void csv_to_bcsv(std::string const &path, std::uint8_t endian, std::vector<std::uint8_t> &buffer,
    std::uint32_t mask = UINT32_MAX) noexcept 
    {libbcsv$cxxbridge1$csv_to_bcsv(path, endian, buffer, mask);}
} // namespace libbcsv