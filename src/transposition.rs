// https://github.com/mvanthoor/rustic/blob/4.0-beta/src/engine/transposition.rs
// Based on this author's work, mainly to understand generic types. Only used for type TableEntry currently.

use std::vec;

use crate::zobrist::PositionHash;
use crate::{util, ShortMove, NULL_SHORT_MOVE};

const DEFAULT_TABLE_SIZE_MB: usize = 200; // in MiB
const NUM_BUCKETS: usize = 3;
const UNINIT_ENTRY: TableEntry = TableEntry {
    bound_type: BoundType::Exact,
    depth: 0,
    eval: 0,
    mv: NULL_SHORT_MOVE,
    valid: false,
};

// TT with generic type T as TableEntry
pub type TranspositionTable<T = TableEntry> = TT<T>;

// TTData trait must be implemented for any type used in the TT
pub trait TTData {
    fn new() -> Self;
    fn get_depth(&self) -> u8;
    fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub enum BoundType {
    Exact,
    Lower,
    Upper,
}

// TODO detect checkmate distance
#[derive(Debug, Clone, Copy)]
pub struct TableEntry {
    pub bound_type: BoundType,
    pub depth: u8,
    pub eval: i32,
    pub mv: ShortMove,
    pub valid: bool,
}
impl TTData for TableEntry {
    fn new() -> Self {
        UNINIT_ENTRY
    }

    fn get_depth(&self) -> u8 {
        self.depth
    }

    fn is_empty(&self) -> bool {
        !self.valid
    }
}

#[derive(Debug)]
pub struct TT<T> {
    table: Vec<Entry<T>>,
    entry_count: usize,
    size_mb: usize,
}
impl<T: TTData + Copy + Clone> TT<T> {
    pub fn new() -> Self {
        Self::with_size(DEFAULT_TABLE_SIZE_MB)
    }

    pub fn with_size(size_mb: usize) -> Self {
        let table = vec![Entry::<T>::new(); Self::mb_to_len(size_mb)];
        //table.shrink_to_fit();
        Self {
            table,
            entry_count: 0,
            size_mb,
        }
    }

    pub fn get(&self, hash: &PositionHash) -> Option<&T> {
        if self.size_mb != 0 {
            self.table[self.get_idx(hash)].get(self.get_bucket_hash(hash))
        } else {
            None
        }
    }

    pub fn insert(&mut self, hash: &PositionHash, data: T) {
        if self.size_mb != 0 {
            let idx = self.get_idx(hash);
            let bucket_hash = self.get_bucket_hash(hash);
            // returns true if the bucket was empty, so we can increment entry_count
            if self.table[idx].insert(bucket_hash, data) {
                self.entry_count += 1;
            }
        }
    }

    pub fn size(&self) -> usize {
        self.table.len() * NUM_BUCKETS
    }

    pub fn heap_alloc_size(&self) -> usize {
        self.table.len() * std::mem::size_of::<Entry<T>>()
    }

    pub fn len(&self) -> usize {
        self.entry_count
    }

    pub fn clear(&mut self) {
        self.entry_count = 0;
        self.table.iter_mut().for_each(|entry| {
            *entry = Entry::new();
        });
    }

    fn mb_to_len(mb_size: usize) -> usize {
        (mb_size * 1024 * 1024) / std::mem::size_of::<Entry<T>>()
    }

    fn get_idx(&self, hash: &PositionHash) -> usize {
        let idx_hash = util::high_bits(*hash); // use high bits for index, and low bits for bucket collision handling
        (idx_hash as usize) % self.table.len()
    }

    fn get_bucket_hash(&self, hash: &PositionHash) -> u32 {
        util::low_bits(*hash)
    }
}

#[derive(Debug, Clone, Copy)]
struct Bucket<T> {
    hash: u32,
    data: T,
}
impl<T: TTData> Bucket<T> {
    fn new() -> Self {
        Self {
            hash: 0,
            data: T::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry<T> {
    buckets: [Bucket<T>; NUM_BUCKETS],
}
impl<T: TTData + Copy + Clone> Entry<T> {
    fn new() -> Self {
        Self {
            buckets: [Bucket::new(); NUM_BUCKETS],
        }
    }

    // returns true if the bucket was empty before data was inserted
    fn insert(&mut self, hash: u32, data: T) -> bool {
        let mut idx = 0;
        for i in 1..self.buckets.len() {
            // skip first bucket as we will start by comparing idx 0
            // replacement strategy is removing lowest depth entry, uninitialised entries are depth 0
            if self.buckets[i].data.get_depth() < self.buckets[idx].data.get_depth() {
                idx = i;
            }
        }
        let was_empty = self.buckets[idx].data.is_empty();
        self.buckets[idx].hash = hash;
        self.buckets[idx].data = data;
        if !was_empty {
            log::trace!("TT bucket collision");
        }
        was_empty
    }

    fn get(&self, hash: u32) -> Option<&T> {
        for bucket in &self.buckets {
            if bucket.hash == hash {
                return Some(&bucket.data);
            }
        }
        None
    }
}
