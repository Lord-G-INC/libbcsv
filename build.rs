fn main() {
    #[cfg(feature = "cxx")] {
        println!("cargo::rerun-if-changed=src\\cxx_exports.rs");
        cxx_build::bridge("src\\cxx_exports.rs")
        .compile("libbcsv");
    }
}