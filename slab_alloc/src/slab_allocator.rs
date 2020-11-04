use std::alloc::{GlobalAlloc, Layout};
use std::ptr::null_mut;
use libc::{mmap, PROT_READ, PROT_WRITE, MAP_PRIVATE, MAP_ANONYMOUS};
use std::mem::size_of;
use std::cmp::max;

const PAGE_SIZE: usize = 16384;
const TOTAL_PAGES: usize = 100000;

static mut DEBUG: bool = false;
static mut BYPASS: bool = false;
static mut MEMORY_BEGIN: Option<*mut u8> = None;

static mut PAGE_TYPE_ARR: Option<*mut u8> = None;
static mut NEXT_PAGE_ARR: Option<*mut u8> = None;
static mut FREE_BLOCK_POINTER_ARR: Option<*mut u8> = None;
static mut FIRST_BLOCK_PAGE_ARR: Option<*mut u8> = None;
static mut TOTAL_BLOCKS_ALLOCATED: Option<*mut u8> = None;

pub struct SlabAllocator {
}

impl SlabAllocator {

    pub const fn new() -> Self {
        SlabAllocator {}
    }
}

unsafe impl GlobalAlloc for SlabAllocator {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // for debugging purposes
        if BYPASS {
            return allocate_working_memory(layout.size());
        }

        let target_size = align(layout.size());
        if target_size <= PAGE_SIZE / 2 {
            block_alloc(target_size)
        } else {
            multipage_alloc(pages_ceil(target_size))
        };

        return allocate_working_memory(layout.size());
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let page = page_by_ptr(ptr);
        if page > TOTAL_PAGES {
            return;
        }

        let page_type = get_page_type(page);

        if page_type == PageType::Blocks {
            blocks_dealloc(ptr);
        } else if page_type == PageType::Multipage {
            multipage_dealloc(ptr);
        }
    }
}

// multipage dealloc
unsafe fn multipage_dealloc(ptr: *mut u8) {
    let page = page_by_ptr(ptr);
    dealloc_pages(page);
}

unsafe fn dealloc_pages(page: usize) {
    set_page_type(page, PageType::Free);
    let next_page = get_next_page(page);
    if next_page != 0 && next_page < TOTAL_PAGES {
        dealloc_pages(next_page);
    }
}

// blocks dealloc
unsafe fn blocks_dealloc(ptr: *mut u8) {
    let page = page_by_ptr(ptr);

    let page_index = page;
    let block_size = block_size_by_page(page);

    let next_empty_block = get_free_block_offset(page);
    ptr.write_word(next_empty_block);
    set_free_block_offset(page, page_start(page).offset_from(ptr) as usize);

    let total_blocks = get_total_blocks_allocated(page_index);
    set_total_blocks_allocated(page_index,  total_blocks - 1);

    if total_blocks == 0 {
        let next_page = get_next_page(page_index);
        let mut next_page_found = 0;
        for i in 0..TOTAL_PAGES {
            if get_next_page(i) == page_index {
                set_next_page(page_index, next_page);
                next_page_found = i;
            }
        }

        for i in 0..log2_ceil(PAGE_SIZE / 2) {
            if get_first_block_page_index(i) == page_index {
                set_first_block_page_index(i, next_page);
            }
        }

        set_page_type(page_index, PageType::Free);
    }
}

// multipage alloc
unsafe fn multipage_alloc(pages: usize) -> *mut u8 {
    let mut first_free_page: Option<usize> = None;

    for i in 0..TOTAL_PAGES {
        if get_page_type(i) == PageType::Free {
            if first_free_page.is_none() {
                first_free_page = Some(i);
            } else if i - first_free_page.unwrap() >= pages {
                break
            }
        } else {
            first_free_page = None;
        }
    }

    let first_page = first_free_page.expect("out of memory for multipage alloc");
    for i in first_page..first_page + pages {
        set_page_type(i, PageType::Multipage);
        if i != first_page + pages - 1 {
            set_next_page(i, i + 1);
        } else {
            set_next_page(i, usize::MAX);
        }
    }

    page_start(first_page)
}

// block alloc
unsafe fn block_alloc(block_size: usize) -> *mut u8 {
    let block_size = max((2 as usize).pow(log2_ceil(block_size) as u32), word_size());
    BYPASS = true;
    //println!("using block size: {}", block_size);
    BYPASS = false;

    debug("block alloc start");
    let page_index = get_page_for_block_alloc(block_size);

    BYPASS = true;
    //println!("block_alloc: get_free_block_offset({})", page_index);
    BYPASS = false;

    debug("block alloc");
    let free_block = get_free_block_offset(page_index);
    debug("block_alloc: get free block");

    BYPASS = true;
    //println!("page index is {}, block offset is {}, block size is {}", page_index, free_block, block_size);
    BYPASS = false;

    let block_ptr = page_start(page_index).add(free_block);

    debug("block_alloc: read next free block");
    let next_free_block = block_ptr.read_word();
    debug("block_alloc: set free block offset");
    set_free_block_offset(page_index, next_free_block);
    debug("block alloc done");

    set_total_blocks_allocated(page_index, get_total_blocks_allocated(page_index) + 1);

    block_ptr
}

unsafe fn get_page_for_block_alloc(block_size: usize) -> usize {
    BYPASS = true;
    //println!("get_page_for_block_alloc {}", block_size);
    BYPASS = false;

    let first_page = get_first_block_page_index(log2_ceil(block_size));
    if first_page == 0 {
        let requested_page = request_block_page(block_size);
        BYPASS = true;
        //println!("setting first page for {} as {}", log2_ceil(block_size), requested_page);
        BYPASS = false;

        set_first_block_page_index(log2_ceil(block_size), requested_page);
        requested_page
    } else {
        BYPASS = true;
        //println!("page_with_free_blocks, first_page = {}", first_page);
        BYPASS = false;
        page_with_free_blocks(first_page, block_size)
    }
}

unsafe fn page_with_free_blocks(page: usize, block_size: usize) -> usize {
    BYPASS = true;
    //println!("page_with_free_blocks: {} {}", page, block_size);
    BYPASS = false;

    if has_free_blocks(page) {
        page
    } else {
        let next_page = get_next_page(page);
        BYPASS = true;
        //println!("page {} is full, next page is {}", page, next_page);
        BYPASS = false;
        if next_page == 0 {
            let requested_page = request_block_page(block_size);
            set_next_page(page, requested_page);
            requested_page
        } else {
            page_with_free_blocks(next_page, block_size)
        }
    }
}

unsafe fn has_free_blocks(page: usize) -> bool {
    BYPASS = true;
    //println!("has_free_blocks: {}", page);
    BYPASS = false;
    get_free_block_offset(page) != usize::MAX
}

unsafe fn request_block_page(block_size: usize) -> usize {
    let requested_page = request_page(PageType::Blocks);
    let mut ptr = page_start(requested_page);

    let total_blocks = PAGE_SIZE / block_size;

    // initially link blocks
    // debug("initially link blocks start");
    BYPASS = true;
    //println!("total blocks on this page: {} with block size {}", total_blocks, block_size);
    BYPASS = false;

    for i in 0..total_blocks {
        let value = if i == total_blocks - 1 {
            usize::MAX
        } else {
            (i + 1) * block_size
        };

        BYPASS = true;
        if i < 5 {
            //println!("for block {} next is {} {:?}", i * block_size, value, ptr);
        }
        BYPASS = false;

        ptr.write_word(value);
        ptr = ptr.add(block_size);
    }
    //  debug("initially link blocks done");

    requested_page
}

unsafe fn request_page(page_type: PageType) -> usize {
    for i in 0..TOTAL_PAGES {
        if get_page_type(i) == PageType::Free {
            set_page_type(i, page_type);
            return i;
        }
    }

    panic!("Out of memory pages");
}

// metadata access
unsafe fn get_total_blocks_allocated(page: usize) -> usize {
    total_blocks_allocated().add(page * word_size()).read_word()
}

unsafe fn set_total_blocks_allocated(page: usize, total_blocks: usize) {
    total_blocks_allocated().add(page * word_size()).write_word(total_blocks)
}

unsafe fn total_blocks_allocated() -> *mut u8 {
    match TOTAL_BLOCKS_ALLOCATED {
        Some(v) => v,
        None => {
            let arr = allocate_working_memory(TOTAL_PAGES * word_size());
            fill_with(arr, TOTAL_PAGES, 0);
            TOTAL_BLOCKS_ALLOCATED = Some(arr);

            arr
        }
    }
}

unsafe fn block_size_by_page(page: usize) -> usize {
    debug("block_size_by_page: 1");
    for i in 0..TOTAL_PAGES {
        if next_page_arr().add(i * word_size()).read_word() == page && i != page {
            return block_size_by_page(i);
        }
    }
    debug("block_size_by_page: 2");

    for i in 0..PAGE_SIZE / 2 {
        if first_block_page_arr().add(i * word_size()).read_word() == page {
            return page
        }
    }
    debug("block_size_by_page: 3");

    panic!("cannot get block size by page")
}

unsafe fn set_first_block_page_index(block_size: usize, page_index: usize) {
    //debug("set first block page index start");
    first_block_page_arr().add(block_size * word_size()).write_word(page_index);
    //debug("set first block page index done");
}

unsafe fn get_first_block_page_index(block_size: usize) -> usize {
    //debug("get first block page index start");
    let res = first_block_page_arr().add(block_size * word_size()).read_word();
    //debug("get first block page index done");

    res
}

unsafe fn first_block_page_arr() -> *mut u8 {
    match FIRST_BLOCK_PAGE_ARR {
        Some(v) => v,
        None => {
            debug("first block page arr alloc");
            let arr_size = log2_ceil(PAGE_SIZE);
            let arr = allocate_working_memory(arr_size * word_size());
            fill_with(arr, arr_size, 0);
            FIRST_BLOCK_PAGE_ARR = Some(arr);
            debug("first block page arr alloc done");

            arr
        }
    }
}

unsafe fn set_free_block_offset(page_index: usize, free_block_offset: usize) {
    //debug("set free block offset start");
    BYPASS = true;
    //println!("set free block offset {} {} {}", page_index, free_block_offset, free_block_offset == usize::MAX);
    BYPASS = false;

    free_block_arr().add(page_index * word_size()).write_word(free_block_offset);
    //debug("set free block offset done");
}

unsafe fn get_free_block_offset(page_index: usize) -> usize {
    //debug("get free block offset start");
    BYPASS = true;
    //println!("get free block offset {}", page_index);
    BYPASS = false;

    let res = free_block_arr().add(page_index * word_size()).read_word();
    //debug("get free block offset done");

    res
}

unsafe fn free_block_arr() -> *mut u8 {
    match FREE_BLOCK_POINTER_ARR {
        Some(v) => v,
        None => {
            debug("free block arr alloc start");
            let arr = allocate_working_memory(TOTAL_PAGES * word_size());
            fill_with(arr, TOTAL_PAGES, 0);
            FREE_BLOCK_POINTER_ARR = Some(arr);
            debug("free block arr alloc done");

            arr
        }
    }
}

unsafe fn get_next_page(page_index: usize) -> usize {
    debug("get next page read start");
    let res = next_page_arr().add(page_index * word_size()).read_word();
    debug("get next page read done");
    res
}

unsafe fn set_next_page(page_index: usize, next_page_index: usize) {
    BYPASS = true;
    //println!("set_next_page: {} {}", page_index, next_page_index);
    BYPASS = false;

    if page_index == next_page_index {
        panic!("page cannot be next page for itself");
    }

    //debug("set next page write start");
    next_page_arr().add(page_index * word_size()).write_word(next_page_index);
    //debug("set next page write done");
}

unsafe fn next_page_arr() -> *mut u8 {
    match NEXT_PAGE_ARR {
        Some(v) => v,
        None => {
            debug("next page alloc start");
            let arr = allocate_working_memory(TOTAL_PAGES * word_size());
            fill_with(arr, TOTAL_PAGES, 0);
            NEXT_PAGE_ARR = Some(arr);
            debug("next page alloc done");

            return arr
        }
    }
}

unsafe fn set_page_type(page_index: usize, page_type: PageType) {
    page_type_arr().add(page_index * word_size()).write_word(match page_type {
        PageType::Free => 0,
        PageType::Blocks => 1,
        PageType::Multipage => 2
    });
}

unsafe fn get_page_type(page_index: usize) -> PageType {
    match page_type_arr().add(page_index * word_size()).read_word() {
        1 => PageType::Blocks,
        2 => PageType::Multipage,
        _ => PageType::Free
    }
}

unsafe fn page_type_arr() -> *mut u8 {
    match PAGE_TYPE_ARR {
        Some(v) => v,
        None => {
            debug("page type alloc start");
            let arr = allocate_working_memory(TOTAL_PAGES * word_size());
            fill_with(arr, TOTAL_PAGES, 0);
            PAGE_TYPE_ARR = Some(arr);
            debug("page type alloc done");

            arr
        }
    }
}

unsafe fn memory_begin() -> *mut u8 {
    match MEMORY_BEGIN {
        Some(v) => v,
        None => {
            let arr = allocate_working_memory(TOTAL_PAGES * PAGE_SIZE);
            MEMORY_BEGIN = Some(arr);

            arr
        }
    }
}

// utils
unsafe fn page_by_ptr(ptr: *mut u8) -> usize {
    let offset = MEMORY_BEGIN.unwrap().offset_from(ptr) as usize;
    offset / PAGE_SIZE
}

unsafe fn page_start(page_index: usize) -> *mut u8 {
    memory_begin().add(page_index * PAGE_SIZE)
}

unsafe fn block_ptr(page_index: usize, block_offset: usize, block_size: usize) -> *mut u8 {
    BYPASS = true;
    //println!("using address: {:?}", page_start(page_index).add(block_offset * block_size));
    BYPASS = false;
    page_start(page_index).add(block_offset * block_size)
}

unsafe fn fill_with(arr: *mut u8, arr_size: usize, value: usize) {
    debug("fill start");
    let mut ptr = arr;

    for _ in 0..arr_size {
        ptr.write_word(value);
        ptr = ptr.add(size_of::<usize>());
    }

    debug("fill done");
}

const fn word_size() -> usize {
    size_of::<usize>()
}

fn pages_ceil(bytes: usize) -> usize {
    bytes/PAGE_SIZE + if bytes % PAGE_SIZE != 0 { 1 } else { 0 }
}

fn align(v: usize) -> usize {
    if v % size_of::<usize>() != 0 {
        v + size_of::<usize>() - (v % size_of::<usize>())
    } else {
        v
    }
}

fn log2_ceil(number: usize) -> usize {
    let mut log2 = 0;
    let mut number = number - 1;

    while number != 0 {
        number = number >> 1;
        log2 += 1;
    }

    log2
}

unsafe fn allocate_working_memory(memory_size: usize) -> *mut u8 {
    mmap(null_mut(), memory_size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) as *mut u8
}

pub trait WriteAndReadWords {

    unsafe fn write_word(&self, word: usize);
    unsafe fn read_word(&self) -> usize;
}

impl WriteAndReadWords for *mut u8 {

    unsafe fn write_word(&self, word: usize) {
        unsafe fn write_word_iter(ptr: &*mut u8, bytes_remaining: usize, word: usize) {
            if bytes_remaining == 0 {
                return;
            }

            ptr.write((word & 0xFF) as u8);

            write_word_iter(&ptr.offset(1), bytes_remaining - 1, word >> 8);
        }

        write_word_iter(&self, size_of::<usize>(), word)
    }

    unsafe fn read_word(&self) -> usize {
        unsafe fn read_word_iter(ptr: &* mut u8, bytes_remaining: usize, read: usize) -> usize {
            if bytes_remaining == 0 {
                return read;
            }

            (read_word_iter(&ptr.offset(1), bytes_remaining - 1, read) << 8) | (ptr.read() as usize)
        }

        read_word_iter(&self, size_of::<usize>(), 0)
    }
}

// util types
#[derive(PartialEq)]
enum PageType {
    Free,
    Blocks,
    Multipage
}

unsafe fn debug(s: &str) {
    if DEBUG {
        BYPASS = true;
        println!("{}", s);
        BYPASS = false;
    }
}