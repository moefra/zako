use std::{
    fs::File,
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
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

    pub fn with_index(self, index: u16) -> IndexedChunk {
        IndexedChunk {
            chunk: Arc::new(self),
            index,
        }
    }

    pub fn load(mmap: memmap2::Mmap) {
        let mut index = 0;
        while index != mmap.len() {
            let length = &mmap.as_ref()[index..index + 8];
            let length = usize::from_le_bytes(length.try_into()?);
            index += 8;
            let data = &mmap.as_ref()[index..index + length];
            index += Self::get_aligned_size(length);
        }
        Ok(())
    }

    pub fn save(&self, bin_file: &mut File) -> eyre::Result<()> {
        match self {
            Chunk::Memory { data, cursor } => {
                let cursor = cursor.load(Ordering::Acquire);
                if cursor == 0 {
                    return Ok(());
                }

                let mut index = 0;

                while index != cursor {
                    // read 8 bytes from chunk as length
                    let length = &data.as_ref()[index..index + 8];
                    bin_file.write_all(&length)?;
                    let length = usize::from_le_bytes(length.try_into()?);
                    index += 8;
                    // read data from chunk
                    let data = &data.as_ref()[index..index + length];
                    bin_file.write_all(data)?;
                    index += Self::get_aligned_size(length);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn get_aligned_size(size: usize) -> usize {
        let align = 8; //std::mem::align_of::<usize>();
        let aligned_size = (size + align - 1) & !(align - 1);
        aligned_size
    }

    pub fn try_alloc(&self, minimum_size: usize) -> Option<usize> {
        match self {
            Chunk::Memory { data, cursor } => {
                let aligned_size = Self::get_aligned_size(minimum_size);

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
                                Ordering::Acquire,
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

#[derive(Debug, Clone)]
pub struct IndexedChunk {
    pub chunk: Arc<Chunk>,
    pub index: u16,
}

impl IndexedChunk {
    pub fn access(&self, offset: usize, length: usize) -> &[u8] {
        let mem = match &*self.chunk {
            Chunk::Memory { data, cursor: _ } => &data,
            Chunk::Mmap(mmap) => mmap.as_ref(),
        };

        &mem[offset..offset + length]
    }

    pub fn access_mut(&self, offset: usize, length: usize) -> &mut [u8] {
        match &*self.chunk {
            Chunk::Memory { data, .. } => unsafe {
                // SAFETY: The caller must ensure no aliasing mutable accesses are created.
                std::mem::transmute::<*const [u8], &mut [u8]>(&data[offset..offset + length])
            },
            Chunk::Mmap(_) => panic!("MMap chunk cannot be accessed mutably"),
        }
    }
}
