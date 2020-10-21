# Simple Memory Allocator

This allocator uses a single continious block of memory allocated with `mmap` call. This one large block of memory is split into smaller blocks, which are allocated on demand of the application and freed when no longer needed.

Each block has the following structure (note that the following numbers are for 64 bit platforms. If you compile this for 32 bit then header will take 8 bytes instead of 16):

0..7 bytes   -- block size and is_filled bit (header).

8..n bytes   -- n-8 bytes for block data.

n..n+7 bytes -- block size (needed for coalescing).

This allocator implements `GlobalAlloc` trait. The following functions of this trait are implemented:

`unsafe fn alloc(&self, layout: Layout) -> *mut u8`

This function creates new block in allocator memory area of size `layout.size`. Note that 16 additional bytes are used for headers.

`unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout)`

This function marks block as free. That means that on one of the next alloc calls this memory space will be reused.

As for `realloc`, default implementation is used.

# Running this allocator

Because this allocator implements `GlobalAlloc` trait, you can actually use it instead of default one.

Notice the following code in `main.rs`:

```
#[global_allocator]
static A: SimpleAllocator = SimpleAllocator::new();
```

That means that our custom allocator will be used. Try running it:

`cargo run --release`

Now, if you want to switch back to default allocator comment out those two lines and run again.

As you may see, our custom allocator is completely usable (though it is not thread safe at all). Notice that it is around 15% slower than the default one, according to the test provided in `main.rs`.