/*
Benchmarking is not yet stable in RUST so we have to use a crate if we want to keep the stable cargo release
See https://crates.io/crates/criterion
to run:
    cd jail
    cargo bench
*/

#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;

use jail::config::JailConf;

use std::ffi::CString;

fn clonejconf(jconf: &JailConf) {
    let _ = jconf.clone();
}

fn allocjconf() {
    let _ = JailConf::default();
}

fn parseint() {
    let _ = "123654".parse::<i32>().unwrap();
}

fn ctsring_new() {
    let _ = CString::new("").unwrap().as_ptr();
}

fn empty_vec() {
    let _: Vec<CString> = vec![];
}

fn i32_to_string(k: i32) {
    let _ = k.to_string();
}

fn bash() {
    cmd::exec::BashCommand::new_bash("").run_utf8();
}

fn clone_benchmark(c: &mut Criterion) {
    let jconf = JailConf::default();
    c.bench_function("clone jconf", |b| b.iter(|| clonejconf(&jconf)));
}

fn alloc_benchmark(c: &mut Criterion) {
    c.bench_function("allocate jconf", |b| b.iter(|| allocjconf()));
}

fn parseint_benchmark(c: &mut Criterion) {
    c.bench_function("parseint", |b| b.iter(|| parseint()));
}

fn cstring_benchmark(c: &mut Criterion) {
    c.bench_function("cstring new", |b| b.iter(|| ctsring_new()));
}

fn empty_vec_benchmark(c: &mut Criterion) {
    c.bench_function("new empty vec", |b| b.iter(|| empty_vec()));
}

fn i32_to_string_benchmark(c: &mut Criterion) {
    let k: i32 = 26478469;
    c.bench_function("i32 to string", |b| b.iter(|| i32_to_string(k)));
}

fn bash_benchmark(c: &mut Criterion) {
    c.bench_function("bash empty command", |b| b.iter(|| bash()));
}

criterion_group!(
    benches,
    empty_vec_benchmark,
    cstring_benchmark,
    parseint_benchmark,
    i32_to_string_benchmark,
    alloc_benchmark,
    clone_benchmark,
    bash_benchmark
); // https://docs.rs/criterion/0.3.0/criterion/macro.criterion_group.html
criterion_main!(benches); // https://docs.rs/criterion/0.3.0/criterion/macro.criterion_main.html
