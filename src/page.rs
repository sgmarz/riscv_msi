use core::{mem::size_of, ptr::null_mut};


pub const PAGE_SIZE: usize = 0x1000; // 4,096 bytes
static mut PAGES_START: *mut u8 = null_mut();
static mut PAGES_END: *mut u8 = null_mut();

/// # Overview
/// Align a value down to the next page size.
/// # Arguments
/// `bytes` - the number of bytes to round
/// # Returns
/// usize - the parameter rounded down.
pub const fn align_down(bytes: usize) -> usize {
    bytes & !(PAGE_SIZE - 1)
}

/// # Overview
/// Align a value up to the next page size.
/// # Arguments
/// `bytes` - the number of bytes to round
/// # Returns
/// usize - the parameter rounded up.
pub const fn align_up(bytes: usize) -> usize {
    align_down(bytes + PAGE_SIZE - 1)
}

/// # Overview
/// Allocate a new structure as a mutable reference
/// This function will allocate in multiples of pages
/// so this could be very wasteful!
/// # Returns
/// `Option<&mut T>` - A Some containing the reference to the data type or None if it could not be allocated.
pub fn alloc<'a, T>() -> Option<&'a mut T> {
    let num_pages = align_up(size_of::<T>()) / PAGE_SIZE;
    unsafe {
        alloc_page(num_pages).map(|ptr| (ptr as *mut T).as_mut().unwrap())
    }
}

/// # Overview
/// Allocate a number of consecutive pages
/// # Arguments
/// `num` - the number of pages to allocate
/// # Returns
/// `Some(*mut u8)` - a pointer to the top of the page
/// 
/// `None` - if the number of pages could not be allocated consecutively
pub fn alloc_page(num: usize) -> Option<*mut u8> {
    if pages_remaining() < num {
        None
    }
    else {
        let ret;
        unsafe {
            ret = PAGES_START;
            PAGES_START = PAGES_START.add(PAGE_SIZE * num);
        }
        Some(ret)
    }
}

/// # Overview
/// Calculate the number of pages remaining on the heap.
/// # Returns
/// `usize` - the number of raw pages remaining on the heap.
pub fn pages_remaining() -> usize {
    unsafe {
        if PAGES_END.is_null() || PAGES_START.is_null() {
            0
        }
        else {
            (PAGES_END as usize - PAGES_START as usize) / PAGE_SIZE
        }
    }
}

pub fn page_init() {
    extern "C" {
        // These come from lds/virt.lds
        static _heap_start: usize;
        static _heap_end: usize;
    }
    unsafe {
        // Basically convert the symbols into pointers. The thing about
        // this is that the address of the symbols is the address we want
        // not the value of the symbol.
        PAGES_START = (&_heap_start) as *const usize as *mut u8;
        PAGES_END = (&_heap_end) as *const usize as *mut u8;
    }
}

