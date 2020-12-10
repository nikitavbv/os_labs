use std::thread::sleep;
use std::time::Duration;

fn main() {
    func2(51, 110);
}

fn func1(a: i32, b: i32) -> i32 {
    let mut res = 0;

    for i in 0..10 {
        //sleep(Duration::from_millis(1000));
        if i > 8 {
            res = result_of_sum(a, b);
        }
        if res > 0 {
            return res;
        }
    }

    res
}

fn func2(a: i32, b: i32) -> i32 {
    let res = 0;
    for _ in 0..10 {
        let res = func1(a, b);
        if res > 0 {
            return res;
        }
    }

    res
}

fn result_of_sum(a: i32, b: i32) -> i32 {
    a + b
}