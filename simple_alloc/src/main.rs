#![feature(test)]

use std::time::Instant;
use crate::simple_allocator::SimpleAllocator;

mod simple_allocator;
mod utils;

#[global_allocator]
static A: SimpleAllocator = SimpleAllocator::new();

fn main() {
    let start = Instant::now();

    for _ in 0..500 {
        let mut v: Vec<usize> = Vec::new();

        for i in 0..10000 {
            v.push(i);
        }

        assert_eq!(10000, v.len());
    }

    let end = Instant::now();
    println!("done in {}ms", (end - start).as_millis());
}
