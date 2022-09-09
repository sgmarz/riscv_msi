use crate::{
    page::pages_remaining,
    pci::pci_init,
    ringbuffer::{RingBuffer, RING_BUFFER_SIZE},
    nvme
};
use core::{
    arch::asm,
    fmt::{Result, Write},
    ptr::{read_volatile, write_volatile},
};

// Registers for the NS16550A. This is connected to 0x1000_0000
// via virt.c in QEMU.
const UART_BASE: usize = 0x1000_0000;
// THR is used if STORE
const UART_THR: usize = 0;
// RBR is used if LOAD
const UART_RBR: usize = 0;
const UART_ICR: usize = 1;
const UART_FCR: usize = 2;
const UART_LCR: usize = 3;
const UART_LSR: usize = 5;

/// Write to a UART register. There are no safety checks! So,
/// make sure you only use the UART_XXYYZZ registers for reg.
fn uart_write(reg: usize, val: u8) {
    unsafe {
        write_volatile((UART_BASE + reg) as *mut u8, val);
    }
}

/// Read from a UART register. There are no safety checks!
fn uart_read(reg: usize) -> u8 {
    unsafe { read_volatile((UART_BASE + reg) as *const u8) }
}

/// Initialize the UART system. For virt, this is probably not necessary.
/// However, LCR = 3 sets word size to 8 bits, FCR = 1 enables the FIFO
/// and ICR = 1 enables interrupts to be triggered when the RBR receives
/// data.
pub fn uart_init() {
    uart_write(UART_LCR, 3);
    uart_write(UART_FCR, 1);
    uart_write(UART_ICR, 1);
}

pub struct Uart;
impl Uart {
    pub fn read_char(&mut self) -> Option<u8> {
        if uart_read(UART_LSR) & 1 == 1 {
            Some(uart_read(UART_RBR))
        } else {
            None
        }
    }
}
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.bytes() {
            self.write_char(c as char)?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result {
        while uart_read(UART_LSR) & (1 << 6) == 0 {}
        uart_write(UART_THR, c as u8);
        Ok(())
    }
}

static mut CONSOLE_BUFFER: RingBuffer = RingBuffer::new();

/// This will be called when the IRQ #10 (hard coded in virt.c)
/// is triggered. This function first determines if the RBR has
/// data via the line status register (LSR) before pushing the
/// received data to the console ring buffer (CONSOLE_BUFFER).
pub fn console_irq() {
    if uart_read(UART_LSR) & 1 == 1 {
        unsafe {
            CONSOLE_BUFFER.push(uart_read(UART_RBR));
        }
    }
}

fn prompt() {
    print!("\n> ");
}

/// Very dump strcmp-like function. The left is expected
/// to be a buffer and the right is expected to be the
/// hard-coded "comparison". Usually, the right is a
/// converted string (in rust, using b"xxyyzz").
fn strequals(left: &[u8], right: &[u8]) -> bool {
    if left.len() < right.len() {
        return false;
    }
    for i in 0..right.len() {
        if left[i] != right[i] {
            return false;
        } else if left[i] == 0 {
            return true;
        }
    }
    true
}

fn runcmd(buffer: &[u8]) {
    if strequals(buffer, b"quit") {
        println!("Quitting...");
        unsafe {
            write_volatile(0x10_0000 as *mut u16, 0x5555);
        }
    } else if strequals(buffer, b"pages") {
        println!("There are {} pages remaining.", pages_remaining());
    } else if strequals(buffer, b"help") {
        println!("Commands: ");
        println!("  pages    - How many pages are remaining?");
        println!("  pci      - Start PCI");
        println!("  quit     - Quit");
    } else if strequals(buffer, b"pci") {
        pci_init();
    } else if strequals(buffer, b"nvme") {
        nvme::init();
    } else {
        println!("Command not found.");
    }
}

pub fn run() {
    let mut typed: usize = 0;
    let mut buffer: [u8; RING_BUFFER_SIZE] = [0; RING_BUFFER_SIZE];
    prompt();
    loop {
        if let Some(c) = unsafe { CONSOLE_BUFFER.pop() } {
            let c_as_char = c as char;
            if c == 10 || c == 13 {
                // Usually for a "terminal" connection, we get
                // a \r (13) instead of a \n (10) depending on the terminal
                // emulator. Check for either, and consider both a enter.
                buffer[typed] = 0;
                println!();
                if typed > 0 {
                    runcmd(&buffer);
                }
                prompt();
                typed = 0;
            } else if c == 127 {
                // Backspace, make sure we don't go past the prompt
                if typed > 0 {
                    // 0x08 is the backspace key, and a BS/SP/BS will
                    // clear whatever was at that point. The backspace alone
                    // doesn't actually delete the character that was there.
                    print!("\x08 \x08");
                    typed -= 1;
                }
            } else if c == 0x1B {
                // Escape sequence
                let esc1 = unsafe { CONSOLE_BUFFER.pop().unwrap_or(0x5B) };
                let esc2 = unsafe { CONSOLE_BUFFER.pop().unwrap_or(0x40) };
                if esc1 == 0x5B {
                    match esc2 {
                        0x41 => println!("UP"),
                        0x42 => println!("DOWN"),
                        0x43 => println!("RIGHT"),
                        0x44 => println!("LEFT"),
                        _ => {}
                    }
                }
            } else if c < 20 {
                // These are *unknown* characters, so instead print out
                // its character number instead of trying to translate it.
                print!(" '{}' ", c);
            } else if typed + 1 < buffer.len() {
                buffer[typed] = c;
                typed += 1;
                print!("{}", c_as_char)
            }
        } else {
            // There was nothing to grab, wait for an interrupt
            unsafe {
                asm!("wfi");
            }
        }
    }
}
