use std::path::{Path, PathBuf};

use alloy::primitives::Address;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::{
    order_book::{Coin, Oid, Side},
    types::{Fill, OrderDiff},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NodeDataOrderDiff {
    pub(crate) user: Address,
    pub(crate) oid: u64,
    pub(crate) px: String,
    pub(crate) coin: String,
    pub(crate) side: Side,
    pub(crate) raw_book_diff: OrderDiff,
}

impl NodeDataOrderDiff {
    pub(crate) fn diff(&self) -> OrderDiff {
        self.raw_book_diff.clone()
    }
    pub(crate) fn oid(&self) -> Oid {
        Oid::new(self.oid)
    }

    pub(crate) fn coin(&self) -> Coin {
        Coin::new(&self.coin)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NodeDataFill(pub Address, pub Fill);

#[derive(Clone, Copy, strum_macros::Display)]
pub(crate) enum EventSource {
    Fills,
    OrderDiffs,
}

impl EventSource {
    #[must_use]
    pub(crate) fn event_source_dir(self, dir: &Path) -> PathBuf {
        match self {
            Self::Fills => dir.join("hl/data/node_fills_by_block"),
            Self::OrderDiffs => dir.join("hl/data/node_raw_book_diffs_by_block"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Batch<E> {
    local_time: NaiveDateTime,
    block_time: NaiveDateTime,
    block_number: u64,
    events: Vec<E>,
}

impl<E> Batch<E> {
    #[allow(clippy::unwrap_used)]
    pub(crate) fn block_time(&self) -> u64 {
        self.block_time.and_utc().timestamp_millis().try_into().unwrap()
    }

    pub(crate) const fn block_number(&self) -> u64 {
        self.block_number
    }

    pub(crate) fn events(self) -> Vec<E> {
        self.events
    }
}
