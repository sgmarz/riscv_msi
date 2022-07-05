use core::ptr::null_mut;

pub const PAGE_SIZE: usize = 0x1000; // 4,096 bytes
static mut PAGES: *mut u8 = null_mut();
static mut PAGES_END: *mut u8 = null_mut();

pub fn alloc<'a, T>() -> Option<&'a mut T> {
    unsafe {
        alloc_page().map(|ptr| (ptr as *mut T).as_mut().unwrap())
    }
}

pub fn alloc_page() -> Option<*mut u8> {
    let ret;
    unsafe {
        if PAGES >= PAGES_END {
            return None;
        }
        ret = PAGES;
        PAGES = PAGES.add(PAGE_SIZE);
    }
    Some(ret)
}

pub fn pages_remaining() -> usize {
    unsafe {
        let addr_diff = PAGES_END as usize - PAGES as usize;
        addr_diff / PAGE_SIZE
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
        PAGES = (&_heap_start) as *const usize as *mut u8;
        PAGES_END = (&_heap_end) as *const usize as *mut u8;
    }
}

