//! Zero-copy buffer management

use std::collections::VecDeque;
use std::sync::Arc;

/// Buffer pool for reusing byte buffers
pub struct BufferPool {
    buffers: VecDeque<Vec<u8>>,
    buffer_size: usize,
    max_buffers: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            buffers: VecDeque::new(),
            buffer_size,
            max_buffers,
        }
    }

    /// Get a buffer from the pool or create a new one
    pub fn get_buffer(&mut self) -> Vec<u8> {
        if let Some(mut buffer) = self.buffers.pop_front() {
            buffer.clear();
            buffer.reserve(self.buffer_size);
            buffer
        } else {
            Vec::with_capacity(self.buffer_size)
        }
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, buffer: Vec<u8>) {
        if self.buffers.len() < self.max_buffers {
            self.buffers.push_back(buffer);
        }
    }

    /// Get current pool size
    pub fn pool_size(&self) -> usize {
        self.buffers.len()
    }
}

/// Shared buffer with reference counting
#[derive(Debug, Clone)]
pub struct SharedBuffer {
    data: Arc<Vec<u8>>,
    start: usize,
    len: usize,
}

impl SharedBuffer {
    /// Create a new shared buffer
    pub fn new(data: Vec<u8>) -> Self {
        let len = data.len();
        Self {
            data: Arc::new(data),
            start: 0,
            len,
        }
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get slice of data
    pub fn as_slice(&self) -> &[u8] {
        &self.data[self.start..self.start + self.len]
    }

    /// Create a slice of this buffer
    pub fn slice(&self, start: usize, len: usize) -> Option<Self> {
        if start + len <= self.len {
            Some(Self {
                data: self.data.clone(),
                start: self.start + start,
                len,
            })
        } else {
            None
        }
    }
} 