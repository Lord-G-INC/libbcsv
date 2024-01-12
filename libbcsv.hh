// EXPORTS FOR FEATURE cxx, THIS IS NOT FOR THE c_exports FEATURE!!!
#ifndef __cplusplus
#error libbcsv.hh must be included in a C++ file!
#endif
#pragma once
#include <cstdint>
#include <memory>
#include <string>
#include <vector>

/// DLL Exports
extern "C" {
std::vector<std::uint8_t> *libbcsv$cxxbridge1$bcsv_to_csv(std::string const &path, std::vector<std::uint8_t> const &data, std::uint8_t endian) noexcept;

std::vector<std::uint8_t> *libbcsv$cxxbridge1$csv_to_bcsv(std::string const &path, std::uint8_t endian, std::uint32_t mask) noexcept;

void libbcsv$cxxbridge1$bcsv_to_xlsx(std::string const &path, std::vector<std::uint8_t> const &data, std::string const &output, std::uint8_t endian) noexcept;
} // extern "C"

namespace libbcsv {
std::unique_ptr<std::vector<std::uint8_t>> bcsv_to_csv(std::string const &path, std::vector<std::uint8_t> const &data, std::uint8_t endian) noexcept;

std::unique_ptr<std::vector<std::uint8_t>> csv_to_bcsv(std::string const &path, std::uint8_t endian, std::uint32_t mask) noexcept;

void bcsv_to_xlsx(std::string const &path, std::vector<std::uint8_t> const &data, std::string const &output, std::uint8_t endian) noexcept;
} // namespace libbcsv

std::unique_ptr<std::vector<std::uint8_t>> 
libbcsv::bcsv_to_csv(const std::string& path, const std::vector<std::uint8_t>& data, std::uint8_t endian) 
noexcept {
    return std::unique_ptr<std::vector<std::uint8_t>>(libbcsv$cxxbridge1$bcsv_to_csv(path, data, endian));
}

std::unique_ptr<std::vector<std::uint8_t>>
libbcsv::csv_to_bcsv(const std::string& path, std::uint8_t endian, std::uint32_t mask) noexcept {
    return std::unique_ptr<std::vector<std::uint8_t>>(libbcsv$cxxbridge1$csv_to_bcsv(path, endian, mask));
}

void libbcsv::bcsv_to_xlsx(const std::string& path, const std::vector<std::uint8_t>& data, 
const std::string& output, std::uint8_t endian) noexcept {
    libbcsv$cxxbridge1$bcsv_to_xlsx(path, data, output, endian);
}