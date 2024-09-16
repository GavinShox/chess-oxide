use std::vec;

use slint::Model;

use crate::zobrist::PositionHash;
use crate::movegen::{Move, NULL_MOVE};

const DEFAULT_TABLE_SIZE_MB: usize = 500; // in MiB
const NUM_BUCKETS: usize = 2;
const UNINIT_ENTRY: TableEntry = TableEntry {
    hash: 0,
    bound_type: BoundType::Exact,
    depth: 0,
    eval: 0,
    mv: NULL_MOVE,
    age: 0
};

fn high_bits(x: u64) -> u32 {
    (x >> 32) as u32
}

fn low_bits(x: u64) -> u32 {
    x as u32
}

// https://github.com/mvanthoor/rustic/blob/4.0-beta/src/engine/transposition.rs learning from here on how to implement a generic TT
trait TTData {
    fn new() -> Self;
    fn get_depth(&self) -> u8;
}

#[derive(Clone, Copy)]
struct Bucket<T> {
    hash: u32,
    data: T,
}

impl<T: TTData> Bucket<T> {
    fn new() -> Self {
        Self { hash: 0, data: T::new() }
    }
}

#[derive(Clone)]
struct Entry<T> {
    buckets: [Bucket<T>; NUM_BUCKETS],
}
impl<T: TTData + Copy> Entry<T> {
    fn new() -> Self {
        Self { buckets: [Bucket::new(); NUM_BUCKETS] }
    }

    fn insert(&mut self, hash: u32, data: T) {
        let mut idx = 0;
        for i in 1..self.buckets.len() {  // skip first bucket as we will start by comparing idx 0
            // replacement strategy is removing lowest depth entry
            if self.buckets[i].data.get_depth() < self.buckets[idx].data.get_depth() {
                idx = i;
            }
        }
        self.buckets[idx].hash = hash;
        self.buckets[idx].data = data;
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
    pub hash: PositionHash,
    pub bound_type: BoundType,
    pub depth: u8,
    pub eval: i32,
    pub mv: Move,
    pub age: u8
}
impl TableEntry {
    pub fn new(hash: PositionHash, bound_type: BoundType, depth: u8, eval: i32, mv: Move, age: u8) -> Self {
        Self { hash, bound_type, depth, eval, mv, age }
    }
}

#[derive(Debug)]
pub struct TranspositionTable {
    table: Box<[TableEntry]>,
    entry_count: usize,
}
impl TranspositionTable {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_TABLE_SIZE_MB)
    }

    pub fn with_capacity(mb_size: usize) -> Self {
        let table_len = mb_to_len(mb_size);
        let table = vec![UNINIT_ENTRY; table_len].into_boxed_slice();
        Self { table, entry_count: 0 }
    }

    pub fn get(&self, hash: PositionHash) -> Option<&TableEntry> {
        let idx = (hash as usize) % self.table.len();
        let first = &self.table[idx];
        if first.hash == hash {
            return Some(first);
        }
        if idx + 1 < self.table.len() {
            let second = &self.table[idx + 1];
            if second.hash == hash {
                return Some(second);
            }
        }
        None
    }

    pub fn insert(&mut self, entry: TableEntry) {
        let idx = (entry.hash as usize) % self.table.len();
        if idx + 1 == self.table.len() {
            let old = &mut self.table[idx];
            if old.hash == 0 {
                self.entry_count += 1;
            }
            *old = entry;
            return;
        }
        let first = &self.table[idx];
        if first.depth <= entry.depth {
            let old = &mut self.table[idx];
            if old.hash == 0 {
                self.entry_count += 1;
            }
            *old = entry;
            return;
        }
        let old = &mut self.table[idx + 1];
        if old.hash == 0 {
            self.entry_count += 1;
        }
        *old = entry;
    }

    // max entries in table
    pub fn size(&self) -> usize {
        self.table.len()
    }

    // size of allocated memory
    pub fn heap_alloc_size(&self) -> usize {
        self.table.len() * std::mem::size_of::<(PositionHash, (BoundType, u8, i32, Move))>()
    }

    // number of valid entries in table
    pub fn len(&self) -> usize {
        self.entry_count
    }

    pub fn clear(&mut self) {
        self.entry_count = 0;
        self.table = vec![UNINIT_ENTRY; self.table.len()].into_boxed_slice();

    }
}

fn mb_to_len(mb_size: usize) -> usize {
    (mb_size * 1024 * 1024) / std::mem::size_of::<TableEntry>()
}