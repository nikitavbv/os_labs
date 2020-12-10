#![feature(test)]
extern crate test;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_before(b: &mut Bencher) {
        b.iter(|| {
            let mut arr = [[0u8; 100]; 100];

            for i in 0..100 {
                for j in 0..100 {
                    arr[j][i] += 1;
                }
            }
        });
    }

    #[bench]
    fn bench_after(b: &mut Bencher) {
        b.iter(|| {
            let mut arr = [[0u8; 100]; 100];

            for i in 0..100 {
                for j in 0..100 {
                    arr[i][j] += 1;
                }
            }
        });
    }
}