use std::vec;

use crate::zobrist::PositionHash;
use crate::movegen::{Move, NULL_MOVE};

const DEFAULT_TABLE_SIZE_MB: usize = 500; // in MiB

#[derive(Debug, Clone, Copy)]
pub enum BoundType {
    Exact,
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy)]
pub struct TableEntry {
    pub hash: PositionHash,
    pub bound_type: BoundType,
    pub depth: u8,
    pub eval: i32,
    pub mv: Move,
    pub age: u8
}

#[derive(Debug)]
pub struct TranspositionTable {
    entries: Box<[TableEntry]>,
}
impl TranspositionTable {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_TABLE_SIZE_MB)
    }

    pub fn with_capacity(mb_size: usize) -> Self {
        let table_len = mb_to_len(mb_size);
        let entries = vec![TableEntry {
            hash: 0,
            bound_type: BoundType::Exact,
            depth: 0,
            eval: 0,
            mv: NULL_MOVE,
            age: 0
        }; table_len].into_boxed_slice();
        Self { entries }
    }

    pub fn get(&self, hash: PositionHash) -> Option<&TableEntry> {
        let idx = (hash as usize) % self.entries.len();
        let first = &self.entries[idx];
        if first.hash == hash {
            return Some(first);
        }
        if idx + 1 < self.entries.len() {
            let second = &self.entries[idx + 1];
            if second.hash == hash {
                return Some(second);
            }
        }
        None
    }

    pub fn insert(&mut self, entry: TableEntry) {
        let idx = (entry.hash as usize) % self.entries.len();
        if idx + 1 == self.entries.len() {
            self.entries[idx] = entry;
            return;
        }
        let first = &self.entries[idx];
        if first.depth <= entry.depth {
            self.entries[idx] = entry;
            return;
        }
        self.entries[idx + 1] = entry;
    }
}

fn mb_to_len(mb_size: usize) -> usize {
    (mb_size * 1024 * 1024) / std::mem::size_of::<TableEntry>()
}