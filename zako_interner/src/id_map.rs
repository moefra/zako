use std::sync::atomic::{AtomicPtr, AtomicU64, AtomicUsize, Ordering};

use redb::ReadableTable;
use rkyv::{Deserialize, Serialize};

const PAGE_SIZE: usize = 4096;
const PAGE_SIZE_BYTES: usize = 8 * PAGE_SIZE;
const MAX_PAGES: usize = 1024 * 1024;

#[derive(Debug)]
pub struct IdMap {
    /// This field is used to store the mapping of ID to chunk.
    ///
    /// It supports 42 billion IDs, occupying 8MB of memory.
    pages: Box<[AtomicPtr<MappingPage>; MAX_PAGES]>,
    allocated_pages: AtomicUsize,
}

#[repr(transparent)]
#[derive(Debug)]
struct MappingPage([AtomicU64; PAGE_SIZE]);

impl MappingPage {
    pub fn new() -> Self {
        Self([const { AtomicU64::new(0) }; PAGE_SIZE])
    }
}

impl IdMap {
    pub fn new() -> Self {
        Self {
            pages: Box::new([const { AtomicPtr::new(std::ptr::null_mut()) }; MAX_PAGES]),
            allocated_pages: AtomicUsize::new(0),
        }
    }

    pub unsafe fn new_uninit() -> Self {
        Self {
            pages: unsafe { Box::new_uninit().assume_init() },
            allocated_pages: AtomicUsize::new(0),
        }
    }

    pub fn load(db: &redb::ReadOnlyTable<u64, &[u8; PAGE_SIZE_BYTES]>) -> eyre::Result<Self> {
        let map = Self::new();

        for entry in db.iter()? {
            let (key, value) = entry?;
            let key = key.value();
            let value = value.value();

            let ptr = map.ensure_page_index(key as usize);

            unsafe {
                std::ptr::copy_nonoverlapping(value.as_ptr(), ptr as *mut u8, PAGE_SIZE_BYTES);
            }
        }
        Ok(map)
    }

    pub fn save(&self, db: &mut redb::Table<u64, &[u8; PAGE_SIZE_BYTES]>) -> eyre::Result<()> {
        for (page_idx, page) in self.pages.iter().enumerate() {
            let page = page.load(Ordering::Relaxed);
            if page.is_null() {
                continue;
            }

            let value = unsafe { (*page).0.as_ptr() as *const [u8; PAGE_SIZE_BYTES] };

            db.insert(page_idx as u64, unsafe { &*value })?;
        }

        Ok(())
    }

    fn ensure_page_index(&self, expect_page_idx: usize) -> *mut MappingPage {
        let mut allocated_pages = self.allocated_pages.load(Ordering::Acquire);

        if allocated_pages >= expect_page_idx {
            return self.pages[expect_page_idx].load(Ordering::Relaxed);
        }

        if allocated_pages >= MAX_PAGES || expect_page_idx > allocated_pages {
            panic!("Zako Interner: ID map out of bounds");
        }

        let mut allocated: Option<*mut MappingPage> = None;

        loop {
            allocated_pages = self.allocated_pages.load(Ordering::Acquire);

            if allocated_pages >= expect_page_idx {
                if let Some(allocated) = allocated {
                    drop(unsafe { Box::from_raw(allocated) });
                }
                return self.pages[expect_page_idx].load(Ordering::Relaxed);
            }

            let allocated = if let Some(allocated) = allocated {
                allocated
            } else {
                let raw = Box::into_raw(Box::new(MappingPage::new()));
                allocated = Some(raw);
                raw
            };

            _ = self.pages[allocated_pages].compare_exchange(
                std::ptr::null_mut(),
                allocated,
                Ordering::Release,
                Ordering::Acquire,
            );
        }
    }

    fn unwrap(value: u64) -> (u16, u64) {
        let chunk_idx = (value >> 48) as u16;
        let offset = value & 0x0000FFFFFFFFFFFF;
        (chunk_idx, offset)
    }

    fn wrap(chunk_idx: u16, offset: u64) -> u64 {
        (chunk_idx as u64) << 48 | offset
    }

    pub fn resolve(&self, id: u32) -> (u16, u64) {
        let page_idx = (id as usize) / PAGE_SIZE;
        let entry_idx = (id as usize) % PAGE_SIZE;

        let page_ptr = self.pages[page_idx].load(Ordering::Acquire);

        if page_ptr.is_null() {
            panic!("Zako Interner: ID out of bounds");
        }

        let packed = unsafe { (*page_ptr).0[entry_idx].load(Ordering::Relaxed) };

        Self::unwrap(packed)
    }

    pub fn register(&self, id: u32, chunk_idx: u16, offset: u64) {
        let page_idx = (id as usize) / PAGE_SIZE;
        let entry_idx = (id as usize) % PAGE_SIZE;

        let page_ptr = self.ensure_page_index(page_idx);

        let packed = Self::wrap(chunk_idx, offset);

        unsafe {
            (*page_ptr).0[entry_idx].store(packed, Ordering::Relaxed);
        }
    }
}
