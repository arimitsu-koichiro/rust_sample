fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");
    // https://github.com/rust-lang/rust/issues/76021
}
