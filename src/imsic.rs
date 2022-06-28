#![allow(dead_code)]

use core::arch::asm;
use crate::console::Uart;

// There are two IMSICs per HART
//   one for machine mode (M)
//   one for supervisor mode (S)
const IMSIC_M: usize = 0x2400_0000;
const IMSIC_S: usize = 0x2800_0000;

// Helper functions for determining MMIO address
// for the messages. Each HART has an M and S mode
// IMSIC. Each HART has its own IMSIC in its own page.
const fn imsic_m(hart: usize) -> usize {
    IMSIC_M + 0x1000 * hart
}

const fn imsic_s(hart: usize) -> usize {
    IMSIC_S + 0x1000 * hart
}

// We only use XLEN for the EIE and EIP
// since there are multiple registers based on the
// interrupt number to enable or to set pending.
const XLEN: usize = usize::BITS as usize;

// The following are used as parameters to a match statement.
// However, I chose to use the same number as their CSRs so
// that if you need to cross-reference it, you have multiple
// places to look.

// M-mode IMSIC CSRs
const MISELECT: usize = 0x350;
const MIREG: usize = 0x351;
const MTOPI: usize = 0xFB0;
const MTOPEI: usize = 0x35C;

// S-Mode IMSIC CSRs
const SISELECT: usize = 0x150;
const SIREG: usize = 0x151;
const STOPI: usize = 0xDB0;
const STOPEI: usize = 0x15C;

// Constants for MISELECT/MIREG
// Pass one of these into MISELECT
// Then the MIREG will reflect that register
const EIDELIVERY: usize = 0x70;
const EITHRESHOLD: usize = 0x72;

// For 32-bit, we have 0x80 for messages 0..31
//                     0x81 for messages 32..63
// and so forth.

// For 64-bit, 0x80 covers 0x81 for messages 0..63
// Referencing 0x81 or any other odd-numbered CSR will cause
// an illegal instruction.

// Same goes for EIP and EIE
const EIP: usize = 0x80;
const EIE: usize = 0xC0;

enum PrivMode {
    Machine = 0,
    Supervisor = 1,
}

// Currently, the CSRs for the IMSICs are not recognized by my
// assembler. Luckily, we can specify any value for the CSR. If it
// doesn't exist, we will get a trap #2 (illegal instruction).

// Write to an IMSIC CSR
fn imsic_write(reg: usize, val: usize) {
    unsafe {
        match reg {
            MISELECT => asm!("csrw 0x350, {val}", val = in(reg) val),
            SISELECT => asm!("csrw 0x150, {val}", val = in(reg) val),

            MIREG => asm!("csrw 0x351, {val}", val = in(reg) val),
            SIREG => asm!("csrw 0x151, {val}", val = in(reg) val),

            MTOPI => asm!("csrw 0xFB0, {val}", val = in(reg) val),
            STOPI => asm!("csrw 0xDB0, {val}", val = in(reg) val),

            MTOPEI => asm!("csrw 0x35C, {val}", val = in(reg) val),
            STOPEI => asm!("csrw 0x15C, {val}", val = in(reg) val),

            _ => panic!("Unknown CSR {}", reg)
        }
    }
}

// Read from an IMSIC CSR
fn imsic_read(reg: usize) -> usize {
    let ret: usize;
    unsafe {
        match reg {
            MISELECT => asm!("csrr {val}, 0x350", val = out(reg) ret),
            SISELECT => asm!("csrr {val}, 0x150", val = out(reg) ret),

            MIREG => asm!("csrr {val}, 0x351", val = out(reg) ret),
            SIREG => asm!("csrr {val}, 0x151", val = out(reg) ret),

            MTOPI => asm!("csrr {val}, 0xFB0", val = out(reg) ret),
            STOPI => asm!("csrr {val}, 0xDB0", val = out(reg) ret),

            MTOPEI => asm!("csrr {val}, 0x35C", val = out(reg) ret),
            STOPEI => asm!("csrr {val}, 0x15C", val = out(reg) ret),

            _ => panic!("Unknown CSR {}", reg)
        }
    }
    ret
}

// Enable a message number
fn imsic_enable(mode: PrivMode, which: usize) {
    let eiebyte = EIE + which / XLEN;
    let bit = which % XLEN;

    match mode {
        PrivMode::Machine => {
            imsic_write(MISELECT, eiebyte);
            let reg = imsic_read(MIREG);
            imsic_write(MIREG, reg | 1 << bit);
        }
        PrivMode::Supervisor => {
            imsic_write(SISELECT, eiebyte);
            let reg = imsic_read(SIREG);
            imsic_write(SIREG, reg | 1 << bit);
        }
    };
}

fn imsic_disable(mode: PrivMode, which: usize) {
    let iebyte = EIE + which / XLEN;
    let bit = which % XLEN;

    match mode {
        PrivMode::Machine => {
            imsic_write(MISELECT, iebyte);
            let reg = imsic_read(MIREG);
            imsic_write(MIREG, reg & !(1 << bit));
        }
        PrivMode::Supervisor => {
            imsic_write(SISELECT, iebyte);
            let reg = imsic_read(SIREG);
            imsic_write(SIREG, reg & !(1 << bit));
        }
    };
}

fn imsic_trigger(mode: PrivMode, which: usize) {
    let iebyte = EIP + which / XLEN;
    let bit = which % XLEN;

    match mode {
        PrivMode::Machine => {
            imsic_write(MISELECT, iebyte);
            let reg = imsic_read(MIREG);
            imsic_write(MIREG, reg | 1 << bit);
        }
        PrivMode::Supervisor => {
            imsic_write(SISELECT, iebyte);
            let reg = imsic_read(SIREG);
            imsic_write(SIREG, reg | 1 << bit);
        }
    };
}

fn imsic_clear(mode: PrivMode, which: usize) {
    let iebyte = EIP + which / XLEN;
    let bit = which % XLEN;

    match mode {
        PrivMode::Machine => {
            imsic_write(MISELECT, iebyte);
            let reg = imsic_read(MIREG);
            imsic_write(MIREG, reg & !(1 << bit));
        }
        PrivMode::Supervisor => {
            imsic_write(SISELECT, iebyte);
            let reg = imsic_read(SIREG);
            imsic_write(SIREG, reg & !(1 << bit));
        }
    };
}

pub fn imsic_init() {
    let hartid = csr_read!("mhartid");
    // First, enable the interrupt file
    // 0 = disabled
    // 1 = enabled
    // 0x4000_0000 = use PLIC instead
    imsic_write(MISELECT, EIDELIVERY);
    imsic_write(MIREG, 1);

    // Set the interrupt threshold.
    // 0 = enable all interrupts
    // P = enable < P only
    // Priorities come from the interrupt number directly
    imsic_write(MISELECT, EITHRESHOLD);
    // Only hear 0, 1, 2, 3, and 4
    imsic_write(MIREG, 5);

    // Enable message #10.
    imsic_enable(PrivMode::Machine, 2);
    imsic_enable(PrivMode::Machine, 4);

    // Trigger interrupt #2
    // SETEIPNUM no longer works
    // This can be done via SETEIPNUM CSR or via MMIO
    // imsic_write!(csr::s::SETEIPNUM, 2);
    unsafe {
        // We are required to write only 32 bits.
        core::ptr::write_volatile(imsic_m(hartid) as *mut u32, 2)
    }
    imsic_trigger(PrivMode::Machine, 4);
}

fn imsic_pop(pr: PrivMode) -> i32 {
    let ret: i32;
    unsafe {
        match pr {
            // MTOPEI
            PrivMode::Machine => asm!("csrrw {ret}, 0x35C, zero", ret = out(reg) ret),
            // STOPEI
            PrivMode::Supervisor => asm!("csrrw {ret}, 0x15C, zero", ret = out(reg) ret),
        }
    }
    // Lower 11 bits are the priority which is the same as the identity
    ret & 0x7FF
}

pub fn imsic_handle() {
    let v = imsic_pop(PrivMode::Machine);
    let mut u = Uart;
    match v {
        2 => println!("First test triggered by MMIO write successful!"),
        4 => println!("Second test triggered by EIP successful!"),
        10 => {
            if let Some(c) = u.read_char() {
                print!("{}", c as char);
            }
        }
        _ => println!("Unknown msi #{}", v),
    }
}
