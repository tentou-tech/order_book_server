use crate::{
    listeners::order_book::{L2Snapshots, TimedSnapshots, utils::compute_l2_snapshots},
    order_book::{
        Coin,
        multi_book::{OrderBooks, Snapshots},
    },
    prelude::*,
    types::{
        inner::{InnerL4Order, InnerOrderDiff},
        node_data::{Batch, NodeDataOrderDiff},
    },
};
use std::collections::HashSet;

#[derive(Clone)]
pub(super) struct OrderBookState {
    order_book: OrderBooks<InnerL4Order>,
    height: u64,
    time: u64,
    snapped: bool,
    ignore_spot: bool,
}

impl OrderBookState {
    pub(super) fn from_snapshot(
        snapshot: Snapshots<InnerL4Order>,
        height: u64,
        time: u64,
        ignore_triggers: bool,
        ignore_spot: bool,
    ) -> Self {
        Self {
            ignore_spot,
            time,
            height,
            order_book: OrderBooks::from_snapshots(snapshot, ignore_triggers),
            snapped: false,
        }
    }

    pub(super) const fn height(&self) -> u64 {
        self.height
    }

    // forcibly take snapshot - (time, height, snapshot)
    pub(super) fn compute_snapshot(&self) -> TimedSnapshots {
        TimedSnapshots { time: self.time, height: self.height, snapshot: self.order_book.to_snapshots_par() }
    }

    // (time, snapshot)
    pub(super) fn l2_snapshots(&mut self, prevent_future_snaps: bool) -> Option<(u64, L2Snapshots)> {
        if self.snapped {
            None
        } else {
            self.snapped = prevent_future_snaps || self.snapped;
            Some((self.time, compute_l2_snapshots(&self.order_book)))
        }
    }

    pub(super) fn compute_universe(&self) -> HashSet<Coin> {
        self.order_book.as_ref().keys().cloned().collect()
    }

    pub(super) fn apply_updates(
        &mut self,
        order_diffs: Batch<NodeDataOrderDiff>,
    ) -> Result<()> {
        let height = order_diffs.block_number();
        let time = order_diffs.block_time();
        if height > self.height + 1 {
            return Err(format!("Expecting block {}, got block {}", self.height + 1, height).into());
        } else if height <= self.height {
            // This is not an error in case we started caching long before a snapshot is fetched
            return Ok(());
        }
        let diffs = order_diffs.events();
        for diff in diffs {
            let oid = diff.oid();
            let coin = diff.coin();
            if coin.is_spot() && self.ignore_spot {
                continue;
            }
            let inner_diff: InnerOrderDiff = diff.diff().try_into()?;
            match inner_diff {
                InnerOrderDiff::New => {
                    let inner_order: InnerL4Order = diff.try_into()?;
                    self.order_book.add_order(inner_order);
                }
                InnerOrderDiff::Update { new_sz, .. } => {
                    self.order_book.modify_sz(oid, coin, new_sz);
                }
                InnerOrderDiff::Remove => {
                    self.order_book.cancel_order(oid, coin);
                }
            }
        }
        self.height += 1;
        self.time = time;
        self.snapped = false;
        Ok(())
    }
}
