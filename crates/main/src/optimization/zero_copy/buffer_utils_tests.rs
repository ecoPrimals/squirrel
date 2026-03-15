// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for zero-copy buffer utilities

#[cfg(test)]
mod tests {
    use super::super::buffer_utils::*;

    #[test]
    fn test_buffer_pool_new() {
        let pool = BufferPool::new(1024, 10);
        assert_eq!(pool.pool_size(), 0);
    }

    #[test]
    fn test_buffer_pool_get_buffer_creates_new() {
        let mut pool = BufferPool::new(1024, 10);
        let buffer = pool.get_buffer();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.capacity() >= 1024);
        assert_eq!(pool.pool_size(), 0);
    }

    #[test]
    fn test_buffer_pool_return_and_reuse() {
        let mut pool = BufferPool::new(1024, 10);

        let mut buffer = pool.get_buffer();
        buffer.extend_from_slice(b"test data");
        pool.return_buffer(buffer);

        assert_eq!(pool.pool_size(), 1);

        let reused = pool.get_buffer();
        assert_eq!(reused.len(), 0); // Should be cleared
        assert!(reused.capacity() >= 1024);
        assert_eq!(pool.pool_size(), 0);
    }

    #[test]
    fn test_buffer_pool_max_capacity() {
        let mut pool = BufferPool::new(512, 3);

        // Add 5 buffers, but only 3 should be retained
        for _ in 0..5 {
            pool.return_buffer(vec![0u8; 512]);
        }

        assert_eq!(pool.pool_size(), 3);
    }

    #[test]
    fn test_buffer_pool_multiple_buffers() {
        let mut pool = BufferPool::new(256, 5);

        // Return multiple buffers
        pool.return_buffer(vec![1, 2, 3]);
        pool.return_buffer(vec![4, 5, 6]);
        pool.return_buffer(vec![7, 8, 9]);

        assert_eq!(pool.pool_size(), 3);

        // Get them back
        let b1 = pool.get_buffer();
        let b2 = pool.get_buffer();
        let b3 = pool.get_buffer();

        assert_eq!(pool.pool_size(), 0);
        assert_eq!(b1.len(), 0);
        assert_eq!(b2.len(), 0);
        assert_eq!(b3.len(), 0);
    }

    #[test]
    fn test_shared_buffer_new() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer = SharedBuffer::new(data);

        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_shared_buffer_empty() {
        let buffer = SharedBuffer::new(vec![]);

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        let empty: &[u8] = &[];
        assert_eq!(buffer.as_slice(), empty);
    }

    #[test]
    fn test_shared_buffer_clone_shares_data() {
        let buffer1 = SharedBuffer::new(vec![1, 2, 3, 4, 5]);
        let buffer2 = buffer1.clone();

        // Both should have same data
        assert_eq!(buffer1.as_slice(), buffer2.as_slice());
        assert_eq!(buffer1.len(), buffer2.len());
    }

    #[test]
    fn test_shared_buffer_slice_valid() {
        let buffer = SharedBuffer::new(vec![1, 2, 3, 4, 5]);
        let slice = buffer.slice(1, 3).unwrap();

        assert_eq!(slice.len(), 3);
        assert_eq!(slice.as_slice(), &[2, 3, 4]);
    }

    #[test]
    fn test_shared_buffer_slice_full() {
        let buffer = SharedBuffer::new(vec![1, 2, 3, 4, 5]);
        let slice = buffer.slice(0, 5).unwrap();

        assert_eq!(slice.len(), 5);
        assert_eq!(slice.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_shared_buffer_slice_empty() {
        let buffer = SharedBuffer::new(vec![1, 2, 3, 4, 5]);
        let slice = buffer.slice(2, 0).unwrap();

        assert_eq!(slice.len(), 0);
        assert!(slice.is_empty());
        let empty: &[u8] = &[];
        assert_eq!(slice.as_slice(), empty);
    }

    #[test]
    fn test_shared_buffer_slice_out_of_bounds() {
        let buffer = SharedBuffer::new(vec![1, 2, 3, 4, 5]);

        assert!(buffer.slice(3, 5).is_none()); // Would exceed length
        assert!(buffer.slice(10, 1).is_none()); // Start beyond end
    }

    #[test]
    fn test_shared_buffer_nested_slicing() {
        let buffer = SharedBuffer::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let slice1 = buffer.slice(2, 5).unwrap(); // [3,4,5,6,7]
        let slice2 = slice1.slice(1, 3).unwrap(); // [4,5,6]

        assert_eq!(slice2.as_slice(), &[4, 5, 6]);
    }

    #[test]
    fn test_shared_buffer_large_data() {
        let large_data = vec![42u8; 10000];
        let buffer = SharedBuffer::new(large_data);

        assert_eq!(buffer.len(), 10000);
        assert_eq!(buffer.as_slice()[0], 42);
        assert_eq!(buffer.as_slice()[9999], 42);
    }

    #[test]
    fn test_buffer_pool_stress() {
        let mut pool = BufferPool::new(128, 5);

        // Simulate heavy usage
        for _ in 0..100 {
            let mut buf = pool.get_buffer();
            buf.extend_from_slice(b"data");
            pool.return_buffer(buf);
        }

        // Should stabilize at max capacity
        assert!(pool.pool_size() <= 5);
    }

    #[test]
    fn test_shared_buffer_zero_copy_benefit() {
        let data = vec![1u8; 1000];
        let buffer1 = SharedBuffer::new(data);

        // Create multiple clones - they should all share the same underlying data
        let buffer2 = buffer1.clone();
        let buffer3 = buffer1.clone();
        let buffer4 = buffer1.clone();

        // All should have same content
        assert_eq!(buffer1.as_slice(), buffer2.as_slice());
        assert_eq!(buffer1.as_slice(), buffer3.as_slice());
        assert_eq!(buffer1.as_slice(), buffer4.as_slice());
    }
}
