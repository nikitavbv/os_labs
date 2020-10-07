use std::alloc::{GlobalAlloc, Layout};

use std::ptr::null_mut;
use std::mem::size_of;

use libc::{mmap, PROT_READ, PROT_WRITE, MAP_PRIVATE, MAP_ANONYMOUS};

use crate::utils::{ToBit, FromBit};

static mut BYPASS: bool = false;

static mut MEMORY_BEGIN: Option<*mut u8> = None;

/* Each block has the following structure:
--- header
0..7   -- block size and is_filled bit.
8..n   -- n-8 bytes for block data.
n..n+7 -- block size (needed for coalescing).

Block size = block data + 16
*/

pub struct SimpleAllocator {
    working_memory_size: usize,
}

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // for debugging purposes
        if BYPASS {
            return Self::allocate_working_memory(layout.size());
        }

        let mut target_size = layout.size();
        if target_size % size_of::<usize>() != 0 {
            target_size += size_of::<usize>() - (target_size % size_of::<usize>());
        }

        let memory_begin = &self.memory_begin();

        let (first_empty_block, header) = &self.find_first_empty_block_after(memory_begin, target_size);
        // check if it makes sense to split the block (we check that new block will have non-zero size)
        if target_size + 3 * size_of::<usize>() < header.0 {
            &self.split_block(first_empty_block, header.0, target_size);
        } else {
            Self::write_block_headers(first_empty_block, header.0, true);
        }

        first_empty_block.add(size_of::<usize>())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let mut ptr = ptr.sub(size_of::<usize>());
        let (mut block_size, _) = Self::unpack_header(ptr.read_word());

        let has_previous_block = ptr != self.memory_begin();
        if has_previous_block {
            let previous_block_size = ptr.sub(size_of::<usize>()).read_word();
            let previous_block_ptr = ptr.sub(size_of::<usize>() * 2 + previous_block_size);
            let (_, previous_block_empty) = Self::unpack_header(previous_block_ptr.read_word());

            if previous_block_empty {
                ptr = previous_block_ptr;
                block_size = previous_block_size + size_of::<usize>() * 2 + block_size;
            }
        }

        let has_next_block = ptr.add(size_of::<usize>() * 2 + block_size) != self.memory_end();
        if has_next_block {
            let next_block_ptr = ptr.add(size_of::<usize>() * 2 + block_size);
            let (next_block_size, next_block_empty) = Self::unpack_header(next_block_ptr.read_word());

            if next_block_empty {
                block_size = block_size + size_of::<usize>() * 2 + next_block_size;
            }
        }

        Self::write_block_headers(&ptr, block_size, false);
    }
}

impl SimpleAllocator {

    pub const fn new() -> Self {
        SimpleAllocator {
            working_memory_size: 20 * 1024 * 1024 * 1024,
        }
    }

    unsafe fn split_block(&self, ptr: &*mut u8, prev_size: usize, target_size: usize) {
        let next_block_addr = Self::next_block_addr(&ptr, target_size);

        Self::write_block_headers(&ptr, target_size, true);
        Self::write_block_headers(&next_block_addr, prev_size - target_size - size_of::<usize>() * 2, false);
    }

    unsafe fn next_block_addr(ptr: &*mut u8, block_size: usize) -> *mut u8 {
        ptr.add(block_size + size_of::<usize>() * 2)
    }

    unsafe fn find_first_empty_block_after(&self, ptr: &*mut u8, target_size: usize) -> (*mut u8, (usize, bool)) {
        let header = Self::unpack_header(ptr.read_word());
        if header.1 || header.0 < target_size {
            self.find_first_empty_block_after(&Self::next_block_addr(&ptr, header.0), target_size)
        } else {
            (ptr.clone(), header)
        }
    }

    unsafe fn write_block_headers(ptr: &*mut u8, block_size: usize, is_filled: bool) {
        ptr.write_word(Self::pack_header(block_size, is_filled));
        ptr.add(&block_size + size_of::<usize>()).write_word(block_size);
    }

    unsafe fn unpack_header(header: usize) -> (usize, bool) {
        let block_size = (&header >> 1) << 1;
        let is_filled = bool::from_bit(header & 1);

        (block_size, is_filled)
    }

    unsafe fn pack_header(block_size: usize, is_filled: bool) -> usize {
        let block_size = if (&block_size & 1) == 1 {
            block_size + 1
        } else {
            block_size
        };

        block_size | is_filled.to_bit()
    }

    unsafe fn memory_begin(&self) -> *mut u8 {
        match MEMORY_BEGIN {
            Some(v) => v,
            None => {
                let memory_begin = Self::allocate_working_memory(self.working_memory_size);
                MEMORY_BEGIN = Some(memory_begin);

                Self::write_block_headers(&memory_begin, &self.working_memory_size - size_of::<usize>() * 2, false);
                memory_begin
            }
        }
    }

    unsafe fn memory_end(&self) -> *mut u8 {
        self.memory_begin().add(self.working_memory_size)
    }

    unsafe fn allocate_working_memory(memory_size: usize) -> *mut u8 {
        mmap(null_mut(), memory_size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) as *mut u8
    }
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
