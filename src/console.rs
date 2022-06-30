use core::fmt::{Result, Write};
use core::ptr::{read_volatile, write_volatile};
// use crate::lock::Mutex;

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

pub fn run() {
    print!("Type something> ");
}
