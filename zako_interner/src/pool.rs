use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use ahash::HashMap;
use parking_lot::RwLock;
use redb::WriteTransaction;

use crate::chunk::{Chunk, IndexedChunk};

#[derive(Debug)]
pub struct Pool {
    chunks: RwLock<Vec<IndexedChunk>>,
    active_chunk: RwLock<IndexedChunk>,
    next_chunk_size: AtomicUsize,
    options: PoolOptions,
}

#[derive(Debug, Clone)]
pub struct PoolOptions {
    initial_chunk_size: usize,
    max_chunk_size: usize,
}

impl Pool {
    fn get_next_chunk_size(current_size: usize, maximum_size: usize) -> usize {
        usize::min(
            usize::checked_mul(current_size, 2).unwrap_or(current_size),
            maximum_size,
        )
    }

    pub fn new(options: PoolOptions) -> Self {
        let initial_chunk = Chunk::new_memory(options.initial_chunk_size);

        let next_chunk_size = AtomicUsize::new(Self::get_next_chunk_size(
            options.initial_chunk_size,
            options.max_chunk_size,
        ));

        let initial_chunk = initial_chunk.with_index(0);

        Self {
            chunks: RwLock::new(Vec::new()),
            active_chunk: RwLock::new(initial_chunk),
            next_chunk_size,
            options,
        }
    }

    pub fn access(&self, chunk_idx: u16, offset: usize, length: usize) -> &[u8] {
        // Use unsafe to extend the lifetime, since chunk storage is stable for read lock
        let chunks_guard = self.chunks.read();
        let chunk_ref = &chunks_guard[chunk_idx as usize];
        // SAFETY: `chunks_guard` will live at least as long as the returned slice,
        // and chunks' storage in Vec is stable as long as Vec is not modified.
        unsafe { &*(chunk_ref.access(offset, length) as *const [u8]) }
    }

    pub fn access_mut(&self, chunk_idx: u16, offset: usize, length: usize) -> &mut [u8] {
        // Use unsafe to extend the lifetime, since chunk storage is stable for read lock
        let chunks_guard = self.chunks.read();
        let chunk_ref = &chunks_guard[chunk_idx as usize];
        // SAFETY: `chunks_guard` will live at least as long as the returned slice,
        // and chunks' storage in Vec is stable as long as Vec is not modified.
        unsafe { &mut *(chunk_ref.access_mut(offset, length) as *mut [u8]) }
    }

    pub fn save(&self, db: &mut WriteTransaction, bin_file: &Path) -> eyre::Result<()> {
        {
            // save chunks to bin_file
            let chunks = self.chunks.read();

            // we append to the file, keep old data
            let mut bin_file = OpenOptions::new()
                .read(true)
                .truncate(false)
                .write(true)
                .create(true)
                .append(true)
                .open(bin_file)?;

            for chunk in chunks.iter() {
                match &*chunk.chunk {
                    Chunk::Memory { data, .. } => {
                        bin_file.write_all(data)?;
                    }
                    Chunk::Mmap(mmap) => {
                        // mmap file is from old data, we don't need to save it
                        continue;
                    }
                }
            }
        }
        Ok(())
    }

    /// 核心分配逻辑
    pub fn alloc(&self, minimum_size: usize) -> eyre::Result<(IndexedChunk, usize)> {
        // fast path
        {
            let chunk_lock = self.active_chunk.read();
            let chunk = &chunk_lock.chunk;

            if let Some(offset) = chunk.try_alloc(minimum_size) {
                return Ok((chunk_lock.clone(), offset));
            }
        }

        let mut active_guard = self.active_chunk.write();

        let new_index = u16::checked_add(active_guard.index, 1).ok_or(eyre::eyre!(
            "Too many chunks. The index of chunk should is overflow"
        ))?;

        let next_chunk_size = self.next_chunk_size.load(Ordering::Acquire);

        self.next_chunk_size.store(
            Self::get_next_chunk_size(next_chunk_size, self.options.max_chunk_size),
            Ordering::Release,
        );

        let new_chunk = Chunk::new_memory(next_chunk_size).with_index(new_index);

        *active_guard = new_chunk.clone();

        let offset = new_chunk.chunk.try_alloc(minimum_size).ok_or(eyre::eyre!(
            "The allocation of data at new chunk should be successful"
        ))?;

        Ok((new_chunk, offset))
    }
}
