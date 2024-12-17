use std::cell::RefCell;

use indexmap::IndexMap;

use crate::format::{bustup::BustupBlockDesc, picture::PicBlock};

pub(super) struct BustupBlockPromisesOwner {
    pub counters: Vec<RefCell<u32>>,
}

impl BustupBlockPromisesOwner {
    pub fn new(count: usize) -> Self {
        BustupBlockPromisesOwner {
            counters: vec![RefCell::new(0); count],
        }
    }

    pub fn bind(&self, block_ids: impl Iterator<Item = u32>) -> BustupBlockPromisesIssuer {
        let promises = (0..)
            .zip(self.counters.iter())
            .zip(block_ids)
            .map(|((index, count), block_id)| {
                *count.borrow_mut() += 1;

                BustupBlockPromise {
                    index,
                    count,
                    block_id,
                }
            })
            .collect::<Vec<_>>()
            .into_iter();

        BustupBlockPromisesIssuer { promises }
    }
}

pub(super) struct BustupBlockPromisesIssuer<'a> {
    promises: std::vec::IntoIter<BustupBlockPromise<'a>>,
}

impl<'a> BustupBlockPromisesIssuer<'a> {
    pub fn visit(&mut self, descriptor: &BustupBlockDesc) -> BustupBlockPromise<'a> {
        assert!(!descriptor.is_null(), "BUG: visiting null block descriptor");

        let promise = self.promises.next().expect("BUG: not enough promises");
        assert_eq!(promise.block_id, descriptor.block_id, "BUG: wrong promise");
        promise
    }

    pub fn visit_opt(&mut self, descriptor: &BustupBlockDesc) -> Option<BustupBlockPromise<'a>> {
        if descriptor.is_null() {
            None
        } else {
            Some(self.visit(descriptor))
        }
    }
}

// Skeleton objects contain promises to decoded picture blocks
// this allows the API user to decide which blocks they are interested in, so they are later decoded
pub struct BustupBlockPromise<'a> {
    index: u32,
    count: &'a RefCell<u32>,
    block_id: u32,
}

impl BustupBlockPromise<'_> {
    pub fn get<'a, T>(self, token: &'a BustupBlockPromiseToken<'a, T>) -> &'a T {
        token.decoded_blocks[self.index as usize]
            .as_ref()
            .expect("BUG: block was not decoded")
    }
}

impl Clone for BustupBlockPromise<'_> {
    fn clone(&self) -> Self {
        *self.count.borrow_mut() += 1;

        BustupBlockPromise {
            index: self.index,
            block_id: self.block_id,
            count: self.count,
        }
    }
}

impl Drop for BustupBlockPromise<'_> {
    fn drop(&mut self) {
        let mut count = self.count.borrow_mut();
        if let Some(new_count) = count.checked_sub(1) {
            *count = new_count;
        } else if !std::thread::panicking() {
            panic!("BUG: block promise was dropped more than once");
        }
    }
}

pub struct BustupBlockPromiseToken<'a, T> {
    pub(super) decoded_blocks: &'a [Option<T>],
}

pub struct BustupSkeleton<'a> {
    pub origin_x: i16,
    pub origin_y: i16,
    pub effective_width: u16,
    pub effective_height: u16,
    pub bustup_id: u32,

    pub base_blocks: Vec<BustupBlockPromise<'a>>,
    pub expressions: IndexMap<String, BustupExpressionSkeleton<'a>>,
}
pub struct BustupExpressionSkeleton<'a> {
    pub face1: Option<BustupBlockPromise<'a>>,
    pub face2: Option<BustupBlockPromise<'a>>,
    pub mouth_blocks: Vec<BustupBlockPromise<'a>>,
    pub eye_blocks: Vec<BustupBlockPromise<'a>>,
}

pub trait BustupBuilder {
    type Args: Sync;
    type Skeleton<'a>;
    type BlockType: Send + 'static;
    type Output;

    fn new<'a>(args: &Self::Args, skeleton: BustupSkeleton<'a>) -> Self::Skeleton<'a>;

    fn new_block(args: &Self::Args, block: PicBlock) -> anyhow::Result<Self::BlockType>;

    fn build(
        skeleton: Self::Skeleton<'_>,
        token: BustupBlockPromiseToken<Self::BlockType>,
    ) -> anyhow::Result<Self::Output>;
}
