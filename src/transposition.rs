use std::vec;

use crate::movegen::{Move, NULL_MOVE};
use crate::zobrist::PositionHash;

const DEFAULT_TABLE_SIZE_MB: usize = 256; // in MiB
const NUM_BUCKETS: usize = 3;
const UNINIT_ENTRY: TableEntry = TableEntry {
    bound_type: BoundType::Exact,
    depth: 0,
    eval: 0,
    mv: NULL_MOVE,
    valid: false,
};

fn high_bits(x: u64) -> u32 {
    (x >> 32) as u32
}

fn low_bits(x: u64) -> u32 {
    x as u32
}

// https://github.com/mvanthoor/rustic/blob/4.0-beta/src/engine/transposition.rs learning from here on how to implement a generic TT
pub trait TTData {
    fn new() -> Self;
    fn get_depth(&self) -> u8;
    fn is_empty(&self) -> bool;
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
impl<T: TTData + Copy> Entry<T> {
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
    pub mv: Move,
    pub valid: bool,
}
//impl TableEntry {
// pub fn new(hash: PositionHash, bound_type: BoundType, depth: u8, eval: i32, mv: Move) -> Self {
//     Self {
//         hash,
//         bound_type,
//         depth,
//         eval,
//         mv,
//     }
// }
//}
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

impl Default for TableEntry {
    fn default() -> Self {
        UNINIT_ENTRY
    }
}

pub type TranspositionTable<T = TableEntry> = TT<T>;
// generic transposition table named TT, pub type TranspositionTable with default TableEntry as T
#[derive(Debug)]
pub struct TT<T> {
    table: Vec<Entry<T>>,
    entry_count: usize,
}
impl<T: TTData + Copy + Clone + Default> TT<T> {
    pub fn new() -> Self {
        Self::with_size(DEFAULT_TABLE_SIZE_MB)
    }

    pub fn with_size(size_mb: usize) -> Self {
        let table = vec![Entry::<T>::new(); Self::mb_to_len(size_mb)];
        //table.shrink_to_fit();
        Self {
            table,
            entry_count: 0,
        }
    }

    fn mb_to_len(mb_size: usize) -> usize {
        (mb_size * 1024 * 1024) / std::mem::size_of::<Entry<T>>()
    }

    pub fn get(&self, hash: PositionHash) -> Option<&T> {
        self.table[self.get_idx(hash)].get(self.get_bucket_hash(hash))
    }

    fn get_idx(&self, hash: PositionHash) -> usize {
        let idx_hash = high_bits(hash); // use high bits for index, and low bits for bucket collision handling
        (idx_hash as usize) % self.table.len()
    }

    fn get_bucket_hash(&self, hash: PositionHash) -> u32 {
        low_bits(hash)
    }

    pub fn insert(&mut self, hash: PositionHash, data: T) {
        let idx = self.get_idx(hash);
        let bucket_hash = self.get_bucket_hash(hash);
        // returns true if the bucket was empty, so we can increment entry_count
        if self.table[idx].insert(bucket_hash, data) {
            self.entry_count += 1;
        }
    }

    pub fn size(&self) -> usize {
        self.table.len()
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
}
// #[derive(Debug)]
// pub struct TranspositionTable {
//     table: Box<[TableEntry]>,
//     entry_count: usize,
// }
// impl TranspositionTable {
//     pub fn new() -> Self {
//         Self::with_capacity(DEFAULT_TABLE_SIZE_MB)
//     }

//     pub fn with_capacity(mb_size: usize) -> Self {
//         let table_len = mb_to_len(mb_size);
//         let table = vec![UNINIT_ENTRY; table_len].into_boxed_slice();
//         Self { table, entry_count: 0 }
//     }

//     pub fn get(&self, hash: PositionHash) -> Option<&TableEntry> {
//         let idx = (hash as usize) % self.table.len();
//         let first = &self.table[idx];
//         if first.hash == hash {
//             return Some(first);
//         }
//         if idx + 1 < self.table.len() {
//             let second = &self.table[idx + 1];
//             if second.hash == hash {
//                 return Some(second);
//             }
//         }
//         None
//     }

//     pub fn insert(&mut self, entry: TableEntry) {
//         let idx = (entry.hash as usize) % self.table.len();
//         if idx + 1 == self.table.len() {
//             let old = &mut self.table[idx];
//             if old.hash == 0 {
//                 self.entry_count += 1;
//             }
//             *old = entry;
//             return;
//         }
//         let first = &self.table[idx];
//         if first.depth <= entry.depth {
//             let old = &mut self.table[idx];
//             if old.hash == 0 {
//                 self.entry_count += 1;
//             }
//             *old = entry;
//             return;
//         }
//         let old = &mut self.table[idx + 1];
//         if old.hash == 0 {
//             self.entry_count += 1;
//         }
//         *old = entry;
//     }

//     // max entries in table
//     pub fn size(&self) -> usize {
//         self.table.len()
//     }

//     // size of allocated memory
//     pub fn heap_alloc_size(&self) -> usize {
//         self.table.len() * std::mem::size_of::<(PositionHash, (BoundType, u8, i32, Move))>()
//     }

//     // number of valid entries in table
//     pub fn len(&self) -> usize {
//         self.entry_count
//     }

//     pub fn clear(&mut self) {
//         self.entry_count = 0;
//         self.table = vec![UNINIT_ENTRY; self.table.len()].into_boxed_slice();

//     }
// }
