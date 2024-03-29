use nonmax::NonMaxU32;

use crate::{
    compile::{MakeWithFile, WithFile},
    syntax::ast::visit::{BlockIndex, ItemIndex},
};

/// BlockId identifies a code block in a file.
///
/// It is defined as an index of item in a file and an index of block in an item.
///
/// (Item being either a block set or a function definition)
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct BlockId {
    item_index: NonMaxU32,
    /// Index of the block in the item or None if it's referring to a function
    block_index: Option<NonMaxU32>,
}

impl BlockId {
    pub const DUMMY: Self = Self {
        item_index: unsafe { NonMaxU32::new_unchecked(u32::MAX - 1) },
        block_index: None,
    };

    pub fn new_block(item_index: ItemIndex, block_index: BlockIndex) -> Self {
        Self {
            item_index: NonMaxU32::new(item_index.into()).unwrap(),
            block_index: Some(NonMaxU32::new(block_index.into()).unwrap()),
        }
    }

    pub fn new_function(item_index: ItemIndex) -> Self {
        Self {
            item_index: NonMaxU32::new(item_index.into()).unwrap(),
            block_index: None,
        }
    }

    pub fn repr(self) -> BlockIdRepr {
        if self == Self::DUMMY {
            BlockIdRepr::Dummy
        } else if let Some(block_index) = self.block_index {
            BlockIdRepr::Block {
                item_index: self.item_index.get().into(),
                block_index: block_index.get().into(),
            }
        } else {
            BlockIdRepr::Function {
                item_index: self.item_index.get().into(),
            }
        }
    }
}

pub type BlockIdWithFile = WithFile<BlockId>;
impl MakeWithFile for BlockId {}

#[salsa::interned]
pub struct SalsaBlockIdWithFile {
    pub block_id: BlockIdWithFile,
}

#[derive(Debug, Copy, Clone)]
pub enum BlockIdRepr {
    Dummy,
    Block {
        item_index: ItemIndex,
        block_index: BlockIndex,
    },
    Function {
        item_index: ItemIndex,
    },
}
