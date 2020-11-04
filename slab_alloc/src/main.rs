use std::time::Instant;
use slab_allocator::SlabAllocator;

mod slab_allocator;

#[global_allocator]
static A: SlabAllocator = SlabAllocator::new();

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

