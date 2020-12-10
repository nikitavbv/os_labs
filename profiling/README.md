# Profiling

There are multiple possible ways to profile Rust code on Linux.

After building an executable with `cargo build` you can run

```bash
perf record --call-graph=dwarf --freq=997 target/debug/profiling
```

After running the binary you can view a report with:

```bash
perf report --hierarchy -M intel
```

CLion also uses `perf` profiler and builds flamegraph from its output.

Here is what we get after running profiler for example code:

[profiler output](assets/1.png)

You may notice that `std::thread:sleep` is taking around 60% of all time.

Here is what we get after removing it:

[profiler output after optimization](assets/2.png)
