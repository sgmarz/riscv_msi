use core::fmt::{Result, Write};
use core::ptr::{read_volatile, write_volatile};
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
    return true;
}

fn runcmd(buffer: &[u8]) {
    if strequals(buffer, b"quit") {
        println!("Quitting...");
        unsafe {
            write_volatile(0x10_0000 as *mut u16, 0x5555);
        }
    }
    else if strequals(buffer, b"help") {
        println!("Commands: ");
        println!("  quit - Quit");
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
                buffer[typed] = 0;
                println!();
                if typed > 0 {
                    runcmd(&buffer);
                }
                prompt();
                typed = 0;
            }
            else if c == 127 {
                if typed > 0 {
                    print!("\x08 \x08");
                    typed -= 1;
                }
            }
            else if c < 20 {
                print!("{}", c);
            }
            else if typed + 1 < buffer.len() {
                buffer[typed] = c;
                typed += 1;
                print!("{}", c_as_char)
            }
        }
        else {
            unsafe {
                core::arch::asm!("wfi");
            }
        }
    }
}


