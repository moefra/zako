use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};

use ahash::HashMap;
use parking_lot::RwLock;

use crate::chunk::{Chunk, IndexedChunk};

#[derive(Debug)]
pub struct Pool {
    chunks: RwLock<Vec<Arc<IndexedChunk>>>,
    active_chunk: RwLock<Arc<IndexedChunk>>,
    next_chunk_size: AtomicUsize,
}

#[derive(Debug, Clone)]
pub struct PoolOptions {
    initial_chunk_size: usize,
}

impl Pool {
    pub fn new(options: PoolOptions) -> Self {
        let initial_chunk = Chunk::new_memory(options.initial_chunk_size);

        let next_chunk_size = AtomicUsize::new(
            usize::checked_mul(options.initial_chunk_size, 2).unwrap_or(options.initial_chunk_size),
        );

        let initial_chunk = IndexedChunk {
            chunk: initial_chunk,
            index: 0,
        };

        let initial_chunk = Arc::new(initial_chunk);

        Self {
            chunks: RwLock::new(Vec::new()),
            active_chunk: RwLock::new(initial_chunk),
            next_chunk_size,
        }
    }

    /// 核心分配逻辑
    pub fn alloc(&self, minimum_size: usize) -> (Arc<IndexedChunk>, usize) {
        // fast path
        {
            let chunk_lock = self.active_chunk.read();
            let chunk = &chunk_lock.chunk;

            if let Some(offset) = chunk.try_alloc(minimum_size) {
                return (chunk_lock.clone(), offset);
            }
        }

        let mut active_guard = self.active_chunk.write();

        let new_index = active_guard.index + 1;

        let next_chunk_size = self.next_chunk_size.load(Ordering::Acquire);

        self.next_chunk_size.store(
            usize::max(
                usize::checked_mul(next_chunk_size, 2).unwrap_or(next_chunk_size),
                minimum_size,
            ),
            Ordering::Release,
        );

        let new_chunk = Arc::new(Chunk::new_memory(next_chunk_size).with_index(new_index));

        *active_guard = new_chunk.clone();

        let offset = new_chunk
            .chunk
            .try_alloc(minimum_size)
            .expect("The data should be allocated in the new chunk");

        (new_chunk, offset)
    }
}
