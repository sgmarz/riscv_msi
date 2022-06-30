use crate::imsic::PrivMode;

#[no_mangle]
pub fn rust_trap() {
    let mcause = csr_read!("mcause");
    let interrupt = mcause >> 31 & 1 == 1;

    if interrupt {
        // Interrupt (asynchronous)
        match mcause & 0xFF {
            9 => crate::imsic::imsic_handle(PrivMode::Supervisor),
            11 => crate::imsic::imsic_handle(PrivMode::Machine),
            _ => println!("Interrupt {}", mcause),
        }
    } else {
        // Exception (synchronous)
        panic!("Exception {} @ 0x{:08x}: 0x{:08x}", mcause, csr_read!("mepc"), csr_read!("mtval"));
    }
}
