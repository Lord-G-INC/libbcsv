#include "libbcsv.hh"

int main() {
    std::vector<uint8_t> buffer{};
    libbcsv::csv_to_bcsv("BlueCoinIDRangeTable.csv", 0, buffer);
    printf("%llu\n", buffer.size());
}