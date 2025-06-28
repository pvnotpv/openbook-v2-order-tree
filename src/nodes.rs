use std::usize;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use bytemuck::{cast, cast_mut, cast_ref};
use static_assertions::const_assert_eq;
use std::mem::size_of;

const NODE_SIZE: usize = 88;

#[derive(
    Eq,
    PartialEq,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}


#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum NodeTag {
    Uninitialized = 0,
    InnerNode = 1,
    LeafNode = 2,
    FreeNode = 3,
    LastFreeNode = 4,
}

pub fn new_node_key(side: Side, price_data: u64, seq_num: u64) -> u128 {
    let seq_num = if side == Side::Bid { !seq_num } else { seq_num };

    let upper = (price_data as u128) << 64;
    upper | (seq_num as u128)
}

pub type NodeHandle = u32;

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct InnerNode {
    pub tag: u8, 
    pub padding: [u8; 3],
    pub prefix_len: u32,
    pub key: u128,
    pub children: [NodeHandle; 2],
    pub child_earliest_expiry: [u64; 2],
    pub reserved: [u8; 40],
}



impl InnerNode {
    pub fn new(prefix_len: u32, key: u128) -> Self {
        Self {
            tag: NodeTag::InnerNode.into(),
            padding: Default::default(), 
            prefix_len,
            key, 
            children: [0; 2],
            child_earliest_expiry: [u64::MAX; 2],
            reserved: [0; 40],
        }
    }

    pub fn walk_down(&self, search_key: u128) -> (NodeHandle, bool) {
        let crit_bit_mask = 1u128 << (127 - self.prefix_len);
        let crit_bit = (search_key & crit_bit_mask) != 0;  
        (self.children[crit_bit as usize], crit_bit)
    }

    pub fn earliest_expiry(&self) -> u64 {
        std::cmp::min(self.child_earliest_expiry[0], self.child_earliest_expiry[1])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LeafNode {
    pub tag: u8,
    pub owner_slot: u8,
    pub time_in_force: u16,
    pub padding: [u8; 4],
    pub key: u128,
    pub owner: [u8; 32],
    pub quantity: i64,
    pub timestamp: u64,
    pub peg_limit: i64,
    pub client_order_id: u64,
}

impl LeafNode {
    pub fn new(
        owner_slot: u8,
        key: u128,
        owner: [u8; 32],
        quantity: i64,
        timestamp: u64,
        time_in_force: u16,
        peg_limit: i64,
        client_order_id: u64,
    ) -> Self {
        Self {
            tag: NodeTag::LeafNode.into(),
            owner_slot,
            time_in_force,
            padding: Default::default(),
            key,
            owner,
            quantity,
            timestamp,
            peg_limit,
            client_order_id,
        }
    }

    pub fn price_data(&self) -> u64 {
        (self.key >> 64) as u64
    } 

    pub fn expiry(&self) -> u64 {
        if self.time_in_force == 0 {
            u64::MAX
        } else {
            self.timestamp + self.time_in_force as u64
        }
    }

    pub fn is_expired(&self, now_ts: u64) -> bool {
        self.time_in_force > 0 && now_ts >= self.timestamp + self.time_in_force as u64
    }
}


#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct FreeNode {
    pub(crate) tag: u8, // NodeTag
    pub(crate) padding: [u8; 3],
    pub(crate) next: NodeHandle,
    pub(crate) reserved: [u8; NODE_SIZE - 16],
    pub(crate) force_align: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct AnyNode {
    pub tag: u8,
    pub data: [u8; 79],
    pub force_align: u64,
}

pub(crate) enum NodeRef<'a> {
    Inner(&'a InnerNode),
    Leaf(&'a LeafNode),
}

pub(crate) enum NodeRefMut<'a> {
    Inner(&'a mut InnerNode),
    Leaf(&'a mut LeafNode),
}

impl AnyNode {
    pub fn key(&self) -> Option<u128> {
        match self.case()? {
            NodeRef::Inner(inner) => Some(inner.key),
            NodeRef::Leaf(leaf) => Some(leaf.key),
        }
    }

    pub(crate) fn children(&self) -> Option<[NodeHandle; 2]> {
        match self.case().unwrap() {
            NodeRef::Inner(&InnerNode { children, .. }) => Some(children),
            NodeRef::Leaf(_) => None,
        }
    }

    pub(crate) fn case(&self) -> Option<NodeRef> {
        match NodeTag::try_from(self.tag) {
            Ok(NodeTag::InnerNode) => Some(NodeRef::Inner(cast_ref(self))),
            Ok(NodeTag::LeafNode) => Some(NodeRef::Leaf(cast_ref(self))),
            _ => None,
        }
    }

    fn case_mut(&mut self) -> Option<NodeRefMut> {
        match NodeTag::try_from(self.tag) {
            Ok(NodeTag::InnerNode) => Some(NodeRefMut::Inner(cast_mut(self))),
            Ok(NodeTag::LeafNode) => Some(NodeRefMut::Leaf(cast_mut(self))),
            _ => None,
        }
    }

    #[inline]
    pub fn as_leaf(&self) -> Option<&LeafNode> {
        match self.case() {
            Some(NodeRef::Leaf(leaf_ref)) => Some(leaf_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn as_leaf_mut(&mut self) -> Option<&mut LeafNode> {
        match self.case_mut() {
            Some(NodeRefMut::Leaf(leaf_ref)) => Some(leaf_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn as_inner(&self) -> Option<&InnerNode> {
        match self.case() {
            Some(NodeRef::Inner(inner_ref)) => Some(inner_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn as_inner_mut(&mut self) -> Option<&mut InnerNode> {
        match self.case_mut() {
            Some(NodeRefMut::Inner(inner_ref)) => Some(inner_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn earliest_expiry(&self) -> u64 {
        match self.case().unwrap() {
            NodeRef::Inner(inner) => inner.earliest_expiry(),
            NodeRef::Leaf(leaf) => leaf.expiry(),
        }
    }
}

impl AsRef<AnyNode> for InnerNode {
    fn as_ref(&self) -> &AnyNode {
        cast_ref(self)
    }
}

impl AsRef<AnyNode> for LeafNode {
    #[inline]
    fn as_ref(&self) -> &AnyNode {
        cast_ref(self)
    }
}


