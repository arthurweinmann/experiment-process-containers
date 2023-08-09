// build.rs

fn main() {
    cc::Build::new()
        .file("src/custom_ls.c")
        .file("src/error.c")
        .compile("custom_ls.a");
}