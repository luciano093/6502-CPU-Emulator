use std::ops::{self, Deref, DerefMut};

use crate::{Byte, Word};

pub struct Memory {
    pub bytes: [Byte; 65536], // 64kb
}

impl Memory {
    pub fn new() -> Self {
        Memory { bytes: [0; 65536] }
    }
}

impl Deref for Memory {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for Memory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
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