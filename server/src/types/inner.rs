use alloy::primitives::Address;

use super::Level;
use crate::{
    order_book::{
        Oid,
        types::{Coin, InnerOrder, Px, Side, Sz},
    },
    prelude::*,
    types::{L4Order, OrderDiff, node_data::NodeDataOrderDiff},
};

// L4Order: the struct we keep in the orderbook (computationally better)
#[derive(Debug, Clone)]
pub(crate) struct InnerL4Order {
    pub user: Address,
    pub coin: Coin,
    pub side: Side,
    pub limit_px: Px,
    pub sz: Sz,
    pub oid: u64,
    pub timestamp: u64,
    pub trigger_condition: String,
    pub is_trigger: bool,
    pub trigger_px: String,
    pub is_position_tpsl: bool,
    pub reduce_only: bool,
    pub order_type: String,
    pub tif: Option<String>,
    pub cloid: Option<String>,
}

impl PartialEq for InnerL4Order {
    fn eq(&self, other: &Self) -> bool {
        self.user == other.user
            && self.coin == other.coin
            && self.side == other.side
            && self.limit_px == other.limit_px
            && self.sz == other.sz
            && self.oid == other.oid
            // timestamp and tif are not available in diffs, so we ignore them for equality checks
            // during snapshot validation.
            // && self.timestamp == other.timestamp
            // && self.trigger_condition == other.trigger_condition
            // && self.is_trigger == other.is_trigger
            // && self.trigger_px == other.trigger_px
            // && self.is_position_tpsl == other.is_position_tpsl
            // && self.reduce_only == other.reduce_only // Not available in diffs, ignore for comparison
            // && self.order_type == other.order_type
            // && self.tif == other.tif
            // && self.cloid == other.cloid // Not available in diffs, ignore for comparison
    }
}
impl Eq for InnerL4Order {}

impl InnerOrder for InnerL4Order {
    fn oid(&self) -> Oid {
        Oid::new(self.oid)
    }

    fn side(&self) -> Side {
        self.side
    }

    fn limit_px(&self) -> Px {
        self.limit_px
    }

    fn sz(&self) -> Sz {
        self.sz
    }

    fn decrement_sz(&mut self, dec: Sz) {
        self.sz.decrement_sz(dec.value());
    }

    fn modify_sz(&mut self, sz: Sz) {
        self.sz = sz;
    }

    fn fill(&mut self, maker_order: &mut Self) -> Sz {
        let match_sz = self.sz().min(maker_order.sz());
        self.decrement_sz(match_sz);
        maker_order.decrement_sz(match_sz);
        match_sz
    }

    fn coin(&self) -> Coin {
        self.coin.clone()
    }
}

impl TryFrom<(Address, L4Order)> for InnerL4Order {
    type Error = Error;

    fn try_from(value: (Address, L4Order)) -> Result<Self> {
        let L4Order {
            coin,
            side,
            limit_px,
            sz,
            oid,
            timestamp,
            trigger_condition,
            is_trigger,
            trigger_px,
            is_position_tpsl,
            reduce_only,
            order_type,
            tif,
            cloid,
            ..
        } = value.1;
        let user = value.0;
        let limit_px = Px::parse_from_str(&limit_px)?;
        let sz = Sz::parse_from_str(&sz)?;
        Ok(Self {
            user,
            coin: Coin::new(&coin),
            side,
            limit_px,
            sz,
            oid,
            timestamp,
            trigger_condition,
            is_trigger,
            trigger_px,
            is_position_tpsl,
            reduce_only,
            order_type,
            tif,
            cloid,
        })
    }
}

impl From<InnerL4Order> for L4Order {
    fn from(value: InnerL4Order) -> Self {
        let InnerL4Order {
            user,
            coin,
            side,
            limit_px,
            sz,
            oid,
            timestamp,
            trigger_condition,
            is_trigger,
            trigger_px,
            is_position_tpsl,
            reduce_only,
            order_type,
            tif,
            cloid,
        } = value;
        let limit_px = limit_px.to_str();
        let sz = sz.to_str();
        Self {
            user: Some(user),
            coin: coin.value(),
            side,
            limit_px,
            sz,
            oid,
            timestamp,
            trigger_condition,
            is_trigger,
            trigger_px,
            is_position_tpsl,
            reduce_only,
            order_type,
            tif,
            cloid,
        }
    }
}

impl TryFrom<NodeDataOrderDiff> for InnerL4Order {
    type Error = Error;

    fn try_from(value: NodeDataOrderDiff) -> Result<Self> {
        let sz = match value.diff() {
            OrderDiff::New { sz } => sz,
            _ => return Err("Not a new order diff".into()),
        };
        let sz = Sz::parse_from_str(&sz)?;
        let limit_px = Px::parse_from_str(&value.px)?;

        // These are defaults for a new resting limit order.
        // The node should not be sending diffs for trigger orders that are not on book.
        let is_trigger = false;
        let trigger_px = "0.0".to_string();
        let trigger_condition = "N/A".to_string();
        let tif = Some("Gtc".to_string());
        let order_type = "Limit".to_string();

        Ok(Self {
            user: value.user,
            coin: value.coin(),
            side: value.side,
            limit_px,
            sz,
            oid: value.oid,
            timestamp: 0, // This field is not available in the diff, and not used for L4 book
            trigger_condition,
            is_trigger,
            trigger_px,
            is_position_tpsl: false, // Not available in diff
            reduce_only: false,      // Not available in diff
            order_type,
            tif,
            cloid: None, // Not available in diff
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InnerLevel {
    pub px: Px,
    pub sz: Sz,
    pub n: usize,
}

impl From<InnerLevel> for Level {
    fn from(value: InnerLevel) -> Self {
        Self::new(value.px.to_str(), value.sz.to_str(), value.n)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InnerOrderDiff {
    New,
    #[allow(dead_code)]
    Update {
        orig_sz: Sz,
        new_sz: Sz,
    },
    Remove,
}

impl TryFrom<OrderDiff> for InnerOrderDiff {
    type Error = Error;

    fn try_from(value: OrderDiff) -> Result<Self> {
        Ok(match value {
            OrderDiff::New { .. } => Self::New,
            OrderDiff::Update { orig_sz, new_sz } => {
                Self::Update { orig_sz: Sz::parse_from_str(&orig_sz)?, new_sz: Sz::parse_from_str(&new_sz)? }
            }
            OrderDiff::Remove => Self::Remove,
        })
    }
}
