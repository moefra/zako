use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub enum Chunk {
    /// In-memory writeable chunk.
    ///
    /// This will append to the data file when save.
    Memory {
        data: Box<[u8]>,
        cursor: AtomicUsize,
    },

    /// Read-only memory mapped file.
    ///
    /// This will be dropped when the pool is saved to filesystem but it will be mapped again soon when the save is done.
    ///
    /// This is because write the data file cause the mmap to be invalid.
    Mmap(memmap2::Mmap),
}

impl Chunk {
    pub fn new_memory(size: usize) -> Self {
        Self::Memory {
            data: vec![0; size].into_boxed_slice(),
            cursor: AtomicUsize::new(0),
        }
    }

    pub fn with_index(self, index: usize) -> IndexedChunk {
        IndexedChunk { chunk: self, index }
    }

    pub fn try_alloc(&self, minimum_size: usize) -> Option<usize> {
        match self {
            Chunk::Memory { data, cursor } => {
                let align = std::mem::align_of::<usize>();
                let aligned_size = (minimum_size + align - 1) & !(align - 1);

                loop {
                    let current_cursor = cursor.load(Ordering::Acquire);

                    let final_cursor = usize::checked_add(current_cursor, aligned_size)?;

                    if final_cursor > data.len() {
                        return None;
                    } else {
                        if cursor
                            .compare_exchange_weak(
                                current_cursor,
                                final_cursor,
                                Ordering::Release,
                                Ordering::Relaxed,
                            )
                            .is_err()
                        {
                            continue;
                        }
                        // alloc success
                        return Some(current_cursor);
                    }
                }
            }
            _ => return None,
        }
    }
}

#[derive(Debug)]
pub struct IndexedChunk {
    pub chunk: Chunk,
    pub index: usize,
}
