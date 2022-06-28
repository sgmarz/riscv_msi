#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

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

const MAX_HARTS: usize = 4;
static mut TRAP_FRAMES: [[usize; 32]; MAX_HARTS] = [[0; 32]; MAX_HARTS];

#[no_mangle]
fn main(hart: usize) {
    if hart >= MAX_HARTS {
        return;
    }
    if hart == 0 {
        console::uart_init();
    }
    csr_write!("mscratch", &TRAP_FRAMES[hart]);
    println!("Booted on hart {}.", hart);
    imsic::imsic_init();
    println!("Done");
    unsafe {
        core::ptr::write_volatile(0x10_0000 as *mut u16, 0x5555);
    }
}

pub mod console;
pub mod imsic;
pub mod trap;
