fn main() {
    #[cfg(feature = "cxx")] {
        cxx_build::bridge("src\\cxx_exports.rs")
        .compile("libbcsv");
    }
}