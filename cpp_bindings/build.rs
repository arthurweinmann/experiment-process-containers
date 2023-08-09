 
// extern crate cc;

fn main() {
    cc::Build::new()
        .include("/usr/include/libnl3")
        .file("src/nsjail_export.cpp")
        .cpp(true)
        .compile("nsjail_export.a");
}