use std::fs::{self, File};
use std::io::prelude::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

const NUM_REPEATS: usize = 10;

fn baseline_read(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("test_file");
    let data = b"hello world";
    let mut fd = File::create(&path).unwrap();
    fd.write_all(data).unwrap();
    drop(fd);
    c.bench_function("baseline_read_sync", move |b| {
        b.iter(|| fs::read(&path).unwrap())
    });
}

fn baseline_read_many(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let paths: Vec<_> = (0..)
        .take(NUM_REPEATS)
        .map(|i| tmp.path().join(format!("test_file_{}", i)))
        .collect();
    let data = b"hello world";
    for path in paths.iter() {
        let mut fd = File::create(path).unwrap();
        fd.write_all(data).unwrap();
        drop(fd);
    }
    c.bench_function("baseline_read_many_sync", move |b| {
        b.iter(|| {
            for path in paths.iter() {
                fs::read(black_box(&path)).unwrap();
            }
        })
    });
}

fn read_hash(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    let sri = cacache_sync::write(&cache, "hello", data).unwrap();
    c.bench_function("get::data_hash_sync", move |b| {
        b.iter(|| cacache_sync::read_hash(black_box(&cache), black_box(&sri)).unwrap())
    });
}

fn read_hash_many(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data: Vec<_> = (0..)
        .take(NUM_REPEATS)
        .map(|i| format!("test_file_{}", i))
        .collect();
    let sris: Vec<_> = data
        .iter()
        .map(|datum| cacache_sync::write(&cache, "hello", datum).unwrap())
        .collect();
    c.bench_function("get::data_hash_many_sync", move |b| {
        b.iter(|| {
            for sri in sris.iter() {
                cacache_sync::read_hash(black_box(&cache), black_box(sri)).unwrap();
            }
        })
    });
}

fn read(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = b"hello world".to_vec();
    cacache_sync::write(&cache, "hello", data).unwrap();
    c.bench_function("get::data_sync", move |b| {
        b.iter(|| cacache_sync::read(black_box(&cache), black_box(String::from("hello"))).unwrap())
    });
}

fn read_hash_big_data(c: &mut Criterion) {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().to_owned();
    let data = vec![1; 1024 * 1024 * 5];
    let sri = cacache_sync::write(&cache, "hello", data).unwrap();
    c.bench_function("get_hash_big_data", move |b| {
        b.iter(|| cacache_sync::read_hash(black_box(&cache), black_box(&sri)).unwrap())
    });
}

criterion_group!(
    benches,
    baseline_read,
    baseline_read_many,
    read_hash,
    read_hash_many,
    read,
    read_hash_big_data
);
criterion_main!(benches);
