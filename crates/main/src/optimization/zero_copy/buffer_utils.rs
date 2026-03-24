// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
    #[must_use]
    pub const fn new(buffer_size: usize, max_buffers: usize) -> Self {
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
    #[must_use]
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
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        let len = data.len();
        Self {
            data: Arc::new(data),
            start: 0,
            len,
        }
    }

    /// Get buffer length
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get slice of data
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data[self.start..self.start + self.len]
    }

    /// Create a slice of this buffer
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    // BufferPool tests
    #[test]
    fn test_buffer_pool_new() {
        let pool = BufferPool::new(1024, 10);
        assert_eq!(pool.pool_size(), 0);
    }

    #[test]
    fn test_buffer_pool_get_buffer_creates_new() {
        let mut pool = BufferPool::new(1024, 10);
        let buf = pool.get_buffer();
        assert_eq!(buf.capacity(), 1024);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_buffer_pool_return_and_reuse() {
        let mut pool = BufferPool::new(1024, 10);
        let buf = pool.get_buffer();
        assert_eq!(pool.pool_size(), 0);

        pool.return_buffer(buf);
        assert_eq!(pool.pool_size(), 1);

        let buf2 = pool.get_buffer();
        assert_eq!(pool.pool_size(), 0);
        assert!(buf2.is_empty()); // Buffer should be cleared
    }

    #[test]
    fn test_buffer_pool_max_buffers() {
        let mut pool = BufferPool::new(64, 2);

        // Return 3 buffers, but max is 2
        pool.return_buffer(vec![1, 2, 3]);
        pool.return_buffer(vec![4, 5, 6]);
        pool.return_buffer(vec![7, 8, 9]); // This should be dropped

        assert_eq!(pool.pool_size(), 2);
    }

    #[test]
    fn test_buffer_pool_returned_buffer_cleared() {
        let mut pool = BufferPool::new(1024, 10);
        let mut buf = vec![0u8; 100];
        buf.fill(42);
        pool.return_buffer(buf);

        let reused = pool.get_buffer();
        assert!(reused.is_empty()); // Should be cleared
    }

    // SharedBuffer tests
    #[test]
    fn test_shared_buffer_new() {
        let buf = SharedBuffer::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(buf.len(), 5);
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_shared_buffer_empty() {
        let buf = SharedBuffer::new(vec![]);
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_shared_buffer_as_slice() {
        let buf = SharedBuffer::new(vec![10, 20, 30, 40, 50]);
        assert_eq!(buf.as_slice(), &[10, 20, 30, 40, 50]);
    }

    #[test]
    fn test_shared_buffer_slice_valid() {
        let buf = SharedBuffer::new(vec![1, 2, 3, 4, 5]);
        let sliced = buf.slice(1, 3);
        assert!(sliced.is_some());
        let sliced = sliced.expect("should succeed");
        assert_eq!(sliced.len(), 3);
        assert_eq!(sliced.as_slice(), &[2, 3, 4]);
    }

    #[test]
    fn test_shared_buffer_slice_out_of_bounds() {
        let buf = SharedBuffer::new(vec![1, 2, 3]);
        assert!(buf.slice(2, 5).is_none());
        assert!(buf.slice(4, 1).is_none());
    }

    #[test]
    fn test_shared_buffer_slice_zero_length() {
        let buf = SharedBuffer::new(vec![1, 2, 3]);
        let sliced = buf.slice(1, 0);
        assert!(sliced.is_some());
        let sliced = sliced.expect("should succeed");
        assert!(sliced.is_empty());
    }

    #[test]
    fn test_shared_buffer_clone_shares_data() {
        let buf = SharedBuffer::new(vec![1, 2, 3]);
        let clone = buf.clone();
        assert_eq!(buf.as_slice(), clone.as_slice());
        // Both should point to the same underlying data via Arc
        assert_eq!(Arc::strong_count(&buf.data), 2);
    }

    #[test]
    fn test_shared_buffer_nested_slice() {
        let buf = SharedBuffer::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let slice1 = buf.slice(2, 4).expect("should succeed"); // [3, 4, 5, 6]
        let slice2 = slice1.slice(1, 2).expect("should succeed"); // [4, 5]
        assert_eq!(slice2.as_slice(), &[4, 5]);
    }
}
