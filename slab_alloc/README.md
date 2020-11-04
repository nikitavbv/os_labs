# Slab Allocator

While this allocator may look similar to simple allocator here are some implementation differences:

- This allocator splits memory into pages after requesting large memory block from the OS.

- Pages are divided into two groups: block pages (for small blocks of memory) and multipage blocks (for large blocks). This in theory should allow better performance compared to simple allocator.

- Pages are linked to each other.

- Blocks inside page are linked as well.

Apart from main memory block we use the following helper structures:

- `PAGE_TYPE_ARR` - store which pages are free/block/multipage.

- `NEXT_PAGE_ARR` - used for linking pages. Page indexes, not references are used for linking - for simplicity.

- `FREE_BLOCK_POINTER_ARR` - for block pages we need to know the index of free block. Note that first word of each emppty block contains an index of next empty block or usize::MAX if no blocks are left. When block is freed, we put it back here, linking to the previous block (looks like a stack).

- `FIRST_BLOCK_PAGE_ARR` - structure to lookup pages by block size,

- `TOTAL_BLOCKS_ALLOCATED` - we also need to track how many blocks are free and how many are not to know when to free this page. We don't need this for multipage allocations.

Important functions:

- `request_page` - find first unused page

- `request_block_page` - find first unused page and create a layout for blocks

- `page_with_free_blocks` - find page with block size which has some free blocks remaining.

- `block_alloc` - allocate memory for block with size

- `multipage_alloc` - allocate memory for large blocks

- `blocks_dealloc`, `dealloc_pages` - free blocks (freeing page if necessary) and pages.
