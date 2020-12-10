# Memory Access Optimization

This demonstrates improving performance of a code snippet leveraging a sequential locality.

Consider the following code:

```rust
let mut arr = [[0u8; 100]; 100];

for i in 0..100 {
    for j in 0..100 {
        arr[j][i] += 1;
    }
}
```

Multi dimensional arrays are still flat in memory. That means that:

```
arr[j][i]
```

is actually

```
arr[j * DIMENSION_SIZE + i]
```
(`DIMENSION_SIZE` is 100 in our example).

In the original code sample, the first dimension index is updated more frequently in inner loop than second dimension
index. That means that instead of accessing memory sequentially, it is accessed randomly instead (jumping 100 bytes at
a time) making CPU cache line useless.

To fix that, we can swap dimension indexes:

```rust
let mut arr = [[0u8; 100]; 100];

for i in 0..100 {
    for j in 0..100 {
        arr[i][j] += 1;
    }
}
```

To make sure that there is a difference, let's run a `cargo bench` (Intel Core i7-8550U - Kaby Lake):

```
test tests::bench_before ... bench:       2,804 ns/iter (+/- 1,328)
test tests::bench_after  ... bench:         511 ns/iter (+/- 18)
```

Swapping i and j allowed 5x speedup.