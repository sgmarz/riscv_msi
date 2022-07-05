use core::ptr::null_mut;

static mut PAGES: *mut u8 = null_mut();
static mut PAGES_END: *mut u8 = null_mut();

pub fn alloc<T>() -> Option<*mut T> {
    alloc_page().map(|ptr| ptr as *mut T)
}

pub fn alloc_page() -> Option<*mut u8> {
    let ret;
    unsafe {
        if PAGES >= PAGES_END {
            return None;
        }
        ret = PAGES;
        PAGES = PAGES.add(0x1000);
    }
    Some(ret)
}

pub fn page_init() {
    extern "C" {
        static _heap_start: usize;
        static _heap_end: usize;
    }
    unsafe {
        PAGES = (&_heap_start) as *const usize as *mut u8;
        PAGES_END = (&_heap_end) as *const usize as *mut u8;
    }
}

