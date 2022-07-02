pub const RING_BUFFER_SIZE: usize = 32;

#[derive(Default)]
pub struct RingBuffer {
    m_buffer: [u8; RING_BUFFER_SIZE],
    m_start: usize,
    m_size: usize
}


impl RingBuffer {
    pub const fn new() -> Self {
        Self {
            m_buffer: [0; RING_BUFFER_SIZE], 
            m_start: 0, 
            m_size: 0 
        }
    }
    pub fn max_size(&self) -> usize {
        self.m_buffer.len()
    }

    pub fn push(&mut self, c: u8) -> bool {
        if self.m_size + 1 >= self.max_size() {
            false
        }
        else {
            self.m_buffer[(self.m_start + self.m_size) % self.max_size()] = c;
            self.m_size += 1;
            true
        }
    }
    pub fn pop(&mut self) -> Option<u8> {
        if self.m_size == 0 {
            None
        }
        else {
            let c = self.m_buffer[self.m_start];
            self.m_size -= 1;
            self.m_start = (self.m_start + 1) % self.max_size();
            Some(c)
        }
    }
}
