use std::ops;

use crate::{Byte, Word};

pub struct Memory {
    bytes: [Byte; 65536], // 64kb
}

impl Memory {
    pub fn new() -> Self {
        Memory { bytes: [0; 65536] }
    }
}

impl ops::Index<Word> for Memory {
    type Output = Byte;

    fn index(&self, index: Word) -> &Self::Output {
        &self.bytes[index as usize]
    }
}

impl ops::IndexMut<Word> for Memory {
    fn index_mut(&mut self, index: Word) -> &mut Self::Output {
        &mut self.bytes[index as usize]
    }
}