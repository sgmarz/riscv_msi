#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

// Include both assembly files and parse them as
// assembly.
global_asm!(include_str!("start.S"));
global_asm!(include_str!("trap.S"));

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(crate::console::Uart, $($args)+);
    });
}
#[macro_export]
macro_rules! println
{
    () => ({
           print!("\r\n")
           });
    ($fmt:expr) => ({
            print!(concat!($fmt, "\r\n"))
            });
    ($fmt:expr, $($args:tt)+) => ({
            print!(concat!($fmt, "\r\n"), $($args)+)
            });
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("[ABORT]: ");
    if let Some(p) = info.location() {
        println!("line {}, file {}", p.line(), p.file());
    } else {
        println!("no information available.");
    }
    abort();
}

fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

// Control and Status Register macros to read/write CSRs
#[macro_export]
macro_rules! csr_write {
    ($csr: expr, $val: expr) => ( unsafe {
        core::arch::asm!(concat!("csrw ", $csr, ", {value}"), value = in(reg) $val);
    })
}

#[macro_export]
macro_rules! csr_read {
    ($csr: expr) => ( unsafe {
        let ret: usize;
        core::arch::asm!(concat!("csrr {ret}, ", $csr), ret = out(reg) ret);
        ret
    })
}

// MAX_HARTS determines how many harts can run on this OS. If a HART is not permitted to
// run, it will be sent to park and never be able to leave, hence turning it off.
const MAX_HARTS: usize = 1;
// Trap frames are used to store the 32 general purpose registers when a hart enters a
// trap.
static mut TRAP_FRAMES: [[usize; 32]; MAX_HARTS] = [[0; 32]; MAX_HARTS];

// Entry point from start.S
#[no_mangle]
fn main(hart: usize) {
    // Make sure we have space for this HART
    if hart >= MAX_HARTS {
        // We don't, send it to park
        return;
    }
    // Set the trap frame for this hart into the scratch register.
    csr_write!("mscratch", &TRAP_FRAMES[hart]);
    // Let hart 0 be the bootstrap hart and set up UART
    if hart == 0 {
        console::uart_init();
        // Setup the IMSIC and see what happens!
        println!("Booted on hart {}.", hart);
        imsic::imsic_init();
        aplic::aplic_init();
        page::page_init();
        pci::pci_init();
        console::run();
    }
}

pub mod aplic;
pub mod console;
pub mod imsic;
pub mod page;
pub mod pci;
pub mod ringbuffer;
pub mod trap;
