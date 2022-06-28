#[no_mangle]
pub fn rust_trap() {
    let mepc = csr_read!("mepc");
    let mcause = csr_read!("mcause");
    let interrupt = mcause >> 31 & 1 == 1;

    if interrupt {
        match mcause & 0xFF {
            11 => crate::imsic::imsic_handle(),
            _ => println!("Interrupt {}", mcause),
        }
    } else {
        println!("Exception {} @ 0x{:08x}: 0x{:08x}", mcause, mepc, csr_read!("mtval"));
        crate::abort();
    }
}
