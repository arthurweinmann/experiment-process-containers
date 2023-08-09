/*
Benchmarking is not yet stable in RUST so we have to use a crate if we want to keep the stable cargo release
See https://crates.io/crates/criterion
to run:
    cd scheduler
    cargo bench
*/

#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;

use crossbeam_queue::{ArrayQueue, PushError};

use crossbeam_channel::bounded;
// use scheduler::thread_pool::ThreadPool;
// use scheduler::deque_thread_pool;
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use sys_util::num_cpu;

use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn array_queue(q: &ArrayQueue<i32>) {
    q.push(458);
    if q.pop().unwrap() != 458 {
        panic!("not good");
    }
}

// fn thread_pool_basic(tp: &ThreadPool, i: Arc<AtomicUsize>) {
// fn thread_pool_basic(tp: &ThreadPool) {
//     let (s, r) = bounded(1);
//     tp.execute(move || {
//         // i.fetch_add(1, Ordering::SeqCst);
//         s.send(1);
//     });
//     if r.recv().unwrap() != 1 {
//         panic!("not 1");
//     }
// }

// fn deque_thread_pool_basic(tp: &deque_thread_pool::ThreadPool) {
//     let (s, r) = bounded(1);
//     tp.execute(move || {
//         // i.fetch_add(1, Ordering::SeqCst);
//         s.send(1);
//     });
//     if r.recv().unwrap() != 1 {
//         panic!("not 1");
//     }
// }

fn format_than_cstring(s1: &str, s2: &str, s3: &str) {
    let _ = CString::new(format!("test {} {} {}", s1, s2, s3)).unwrap();
}

fn cstring_from_raw_unchecked(s1: &[u8]) {
    let _ = unsafe { CString::from_vec_unchecked(s1.to_vec()) };
}

fn concat_u8_slice(u1: &[u8], u2: &[u8], u3: &[u8]) {
    let _ = &[u1, u2, u2].concat();
}

fn clone_cstring(c1: &CString) {
    let _ = c1.clone();
}

fn parse_uint32(u1: &str) {
    let _ = u1.parse::<i32>().unwrap();
}

fn parse_uint32_cstr(u1: &CStr) {
    let _ = (unsafe { std::str::from_utf8_unchecked(u1.to_bytes()) })
        .parse::<i32>()
        .unwrap();
}

// fn thread_pool_basic_raw_comparison(i: Arc<AtomicUsize>) {
fn thread_pool_basic_raw_comparison() {
    let (s, r) = bounded(1);
    s.send(1);
    if r.recv().unwrap() != 1 {
        panic!("not 1");
    }
    // i.fetch_add(1, Ordering::SeqCst);
}

fn array_queue_benchmark(c: &mut Criterion) {
    let q = ArrayQueue::new(2);
    c.bench_function("CrossBeam ArrayQueue single thread push pop", |b| {
        b.iter(|| array_queue(&q))
    });
}

// fn thread_pool_basic_benchmark(c: &mut Criterion) {
//     let tp = ThreadPool::new(num_cpu::get());
//     // let i = Arc::new(AtomicUsize::new(1));
//     c.bench_function("thread pool basic", |b| {
//         b.iter(|| thread_pool_basic(&tp))
//     });
// }

// fn deque_thread_pool_basic_benchmark(c: &mut Criterion) {
//     let tp = deque_thread_pool::ThreadPool::new(num_cpu::get());
//     // let i = Arc::new(AtomicUsize::new(1));
//     c.bench_function("deque thread pool basic", |b| {
//         b.iter(|| deque_thread_pool_basic(&tp))
//     });
// }

// fn thread_pool_basic_raw_comparison_benchmark(c: &mut Criterion) {
//     // let i = Arc::new(AtomicUsize::new(1));
//     c.bench_function("thread pool basic raw comparison", |b| {
//         b.iter(|| thread_pool_basic_raw_comparison())
//     });
// }

fn parse_uint32_benchmark(c: &mut Criterion) {
    let u1 = "2678618";
    c.bench_function("parse uint32", |b| b.iter(|| parse_uint32(&u1)));
}

fn parse_uint32_cstr_benchmark(c: &mut Criterion) {
    let u1 = CString::new("2678618").unwrap();
    let u1_cstr = u1.as_c_str();
    c.bench_function("parse uint32 from cstr", |b| {
        b.iter(|| parse_uint32_cstr(u1_cstr))
    });
}

fn format_plus_cstring_benchmark(c: &mut Criterion) {
    let s1 = "bob";
    let s2 = "alice";
    let s3 = "wihner";
    c.bench_function("format than cstring new", |b| {
        b.iter(|| format_than_cstring(s1, s2, s3))
    });
}

fn cstring_from_raw_unchecked_benchmark(c: &mut Criterion) {
    let s1 = "eiofjzeoi jreiojf reoijf eriojf oirejf \000".as_bytes();
    c.bench_function("cstring_from_raw_unchecked", |b| {
        b.iter(|| cstring_from_raw_unchecked(&s1))
    });
}

fn clone_cstring_benchmark(c: &mut Criterion) {
    let s1 = CString::new("eiofjzeoi jreiojf reoijf eriojf oirejf").unwrap();
    c.bench_function("clone_cstring", |b| b.iter(|| clone_cstring(&s1)));
}

fn concat_u8_ref(c: &mut Criterion) {
    let u1 = "eoifjoejf".as_bytes();
    let u2 = "eoifefef joejf".as_bytes();
    let u3 = "eoifjoejf zeg \000".as_bytes();

    c.bench_function("concat_u8_slice", |b| {
        b.iter(|| concat_u8_slice(&u1, &u2, &u3))
    });
}

pub fn timestamp_second(delay_sec: u64) -> u64 {
    if delay_sec == 0 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    } else {
        SystemTime::now()
            .add(Duration::from_secs(delay_sec))
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}

fn timestamp_benchmark(c: &mut Criterion) {
    c.bench_function("timestamp benchmark", |b| b.iter(|| timestamp_second(0)));
}

criterion_group!(
    benches,
    timestamp_benchmark,
    // array_queue_benchmark,
    // parse_uint32_benchmark,
    // parse_uint32_cstr_benchmark,
    // format_plus_cstring_benchmark,
    // cstring_from_raw_unchecked_benchmark,
    // clone_cstring_benchmark,
    // concat_u8_ref,
    // thread_pool_basic_benchmark,
    // deque_thread_pool_basic_benchmark,
    // thread_pool_basic_raw_comparison_benchmark
); // https://docs.rs/criterion/0.3.0/criterion/macro.criterion_group.html
criterion_main!(benches); // https://docs.rs/criterion/0.3.0/criterion/macro.criterion_main.html