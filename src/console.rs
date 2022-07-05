use core::fmt::{Result, Write};
use core::ptr::{read_volatile, write_volatile};
use crate::page::pages_remaining;
use crate::ringbuffer::{RingBuffer, RING_BUFFER_SIZE};

const UART_BASE: usize = 0x1000_0000;
const UART_THR: usize = 0;
const UART_RBR: usize = 0;
const UART_ICR: usize = 1;
const UART_FCR: usize = 2;
const UART_LCR: usize = 3;
const UART_LSR: usize = 5;

fn uart_write(reg: usize, val: u8) {
    unsafe {
        write_volatile((UART_BASE + reg) as *mut u8, val);
    }
}

fn uart_read(reg: usize) -> u8 {
    unsafe { read_volatile((UART_BASE + reg) as *const u8) }
}

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
        }
        else {
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

fn strequals(left: &[u8], right: &[u8]) -> bool {
    if left.len() < right.len() {
        return false;
    }
    for i in 0..right.len() {
        if left[i] != right[i] {
            return false;
        }
        else if left[i] == 0 {
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
    }
    else if strequals(buffer, b"pages") {
        println!("There are {} pages remaining.", pages_remaining());
    }
    else if strequals(buffer, b"help") {
        println!("Commands: ");
        println!("  pages   - How many pages are remaining?");
        println!("  quit    - Quit");
    }
    else {
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
            }
            else if c == 127 {
                // Backspace, make sure we don't go past the prompt
                if typed > 0 {
                    // 0x08 is the backspace key, and a BS/SP/BS will
                    // clear whatever was at that point. The backspace alone
                    // doesn't actually delete the character that was there.
                    print!("\x08 \x08");
                    typed -= 1;
                }
            }
            else if c < 20 {
                // These are *unknown* characters, so instead print out
                // its character number instead of trying to translate it.
                print!(" '{}' ", c);
            }
            else if typed + 1 < buffer.len() {
                buffer[typed] = c;
                typed += 1;
                print!("{}", c_as_char)
            }
        }
        else {
            // There was nothing to grab, wait for an interrupt
            unsafe {
                core::arch::asm!("wfi");
            }
        }
    }
}


