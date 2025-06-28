use std::error;
use std::error::Error;

use crate::nodes::{AnyNode, NodeHandle, NodeTag, NodeRef, LeafNode, InnerNode};
use crate::price_data;

use bytemuck::{cast, cast_mut, cast_ref};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use super::nodes::Side;
use super::nodes::FreeNode;
use crate::error::OpenBookError;
use static_assertions::const_assert_eq;

#[derive(
    Eq,
    PartialEq,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[repr(u8)]
pub enum OrderTreeType {
    Bids, 
    Asks
}

impl OrderTreeType {
    pub fn side(&self) -> Side {
        match *self {
            Self::Bids => Side::Bid, 
            Self::Asks => Side::Ask
        }
    }
}

pub struct OrderTreeRoot {
    pub maybe_node: NodeHandle,
    pub leaf_count: u32
}

impl OrderTreeRoot {
    pub fn node(&self) -> Option<NodeHandle> {
        if self.leaf_count == 0 {
            None
        } else {
            Some(self.maybe_node)
        }
    }
}

pub struct OrderTreeNodes {
    pub order_tree_type: u8, 
    pub bump_index: u32, 
    pub free_list_len: u32, 
    pub free_list_head: NodeHandle, 
    pub nodes: [AnyNode; 1024]
}

impl OrderTreeNodes {
    pub fn order_tree_type(&self) -> OrderTreeType {
        OrderTreeType::try_from(self.order_tree_type).unwrap()
    }

    pub fn node(&self, handle: NodeHandle) -> Option<&AnyNode> {
        let node = &self.nodes[handle as usize];
        let tag = NodeTag::try_from(node.tag);

        match tag {
            Ok(NodeTag::InnerNode) | Ok(NodeTag::LeafNode) => Some(node),
            _ => None,
        }
    }

    pub fn node_mut(&mut self, handle: NodeHandle) -> Option<&mut AnyNode> {
        let node = &mut self.nodes[handle as usize];
        let tag = NodeTag::try_from(node.tag);
        match tag {
            Ok(NodeTag::InnerNode) | Ok(NodeTag::LeafNode) => Some(node),
            _ => None,
        }
    }


    fn remove(&mut self, key: NodeHandle) -> Option<AnyNode> {
        let val = *self.node(key)?;

        self.nodes[key as usize] = cast(FreeNode {
            tag: if self.free_list_len == 0 {
                NodeTag::LastFreeNode.into()
            } else {
                NodeTag::FreeNode.into()
            },
            padding: Default::default(),
            next: self.free_list_head,
            reserved: [0; 72],
            force_align: 0,
        });

        self.free_list_len += 1;
        self.free_list_head = key;
        Some(val)
    }


    fn insert(&mut self, val: &AnyNode) -> std::result::Result<NodeHandle, std::io::ErrorKind> {
        match NodeTag::try_from(val.tag) {
            Ok(NodeTag::InnerNode) | Ok(NodeTag::LeafNode) => (),
            _ => unreachable!(),
        };

        if self.free_list_len == 0 {

            if (self.bump_index as usize) >= self.nodes.len()  || self.bump_index >= u32::MAX { 
                println!("Something fucking wrong");
            }

            self.nodes[self.bump_index as usize] = *val;
            let key = self.bump_index;
            self.bump_index += 1;
            return Ok(key);
        }

        let key = self.free_list_head;
        let node = &mut self.nodes[key as usize];

        match NodeTag::try_from(node.tag) {
            Ok(NodeTag::FreeNode) => assert!(self.free_list_len > 1),
            Ok(NodeTag::LastFreeNode) => assert_eq!(self.free_list_len, 1),
            _ => unreachable!(),
        };

        self.free_list_head = cast_ref::<AnyNode, FreeNode>(node).next;
        self.free_list_len -= 1;
        *node = *val;
        Ok(key)
    }

    pub fn insert_leaf(
        &mut self,
        root: &mut OrderTreeRoot,
        new_leaf: &LeafNode,
    ) -> Result<(NodeHandle, Option<LeafNode>), std::io::ErrorKind> {
        // path of InnerNode handles that lead to the new leaf

        let mut stack: Vec<(NodeHandle, bool)> = vec![];

        // deal with inserts into an empty tree
        let mut parent_handle: NodeHandle = match root.node() {
            Some(h) => h,
            None => {
                // create a new root if none exists
                let handle = self.insert(new_leaf.as_ref())?;
                root.maybe_node = handle;
                root.leaf_count = 1;
                return Ok((handle, None));
            }
        };

        let parent_contents_ = *self.node(parent_handle).unwrap();
        let parent_key_ = parent_contents_.key().unwrap();
        println!("The parent key is: {}", price_data(parent_key_));


        // walk down the tree until we find the insert location
        loop {
            // require if the new node will be a child of the root
            let parent_contents = *self.node(parent_handle).unwrap();
            let parent_key = parent_contents.key().unwrap();

            let shared_prefix_len: u32 = (parent_key ^ new_leaf.key).leading_zeros();
            println!("The here is: {} {}: {}", price_data(parent_key), price_data(new_leaf.key), shared_prefix_len);
            

            match parent_contents.case() {
                None => unreachable!(),
                Some(NodeRef::Inner(inner)) => {
                    let keep_old_parent = shared_prefix_len >= inner.prefix_len;
                    println!("Keep old parent, Inner prefix len: {}", inner.prefix_len);
                    if keep_old_parent {
                        let (child, crit_bit) = inner.walk_down(new_leaf.key);
                        stack.push((parent_handle, crit_bit));
                        parent_handle = child;
                        continue;
                    };
                }
                _ => (),
            };
            // implies parent is a Leaf or Inner where shared_prefix_len < prefix_len
            // we'll replace parent with a new InnerNode that has new_leaf and parent as children

            // change the parent in place to represent the LCA of [new_leaf] and [parent]
            let crit_bit_mask: u128 = 1u128 << (127 - shared_prefix_len);

            let new_leaf_crit_bit = (crit_bit_mask & new_leaf.key) != 0;
            let old_parent_crit_bit = !new_leaf_crit_bit;

            let new_leaf_handle = self.insert(new_leaf.as_ref())?;
            let moved_parent_handle = match self.insert(&parent_contents) {
                Ok(h) => h,
                Err(e) => {
                    self.remove(new_leaf_handle).unwrap();
                    return Err(e);
                }
            };

            let new_parent: &mut InnerNode = cast_mut(self.node_mut(parent_handle).unwrap());
            *new_parent = InnerNode::new(shared_prefix_len, new_leaf.key);

            new_parent.children[new_leaf_crit_bit as usize] = new_leaf_handle;
            new_parent.children[old_parent_crit_bit as usize] = moved_parent_handle;

            let new_leaf_expiry = new_leaf.expiry();
            let old_parent_expiry = parent_contents.earliest_expiry();

            new_parent.child_earliest_expiry[new_leaf_crit_bit as usize] = new_leaf_expiry;
            new_parent.child_earliest_expiry[old_parent_crit_bit as usize] = old_parent_expiry;

            // walk up the stack and fix up the new min if needed
            if new_leaf_expiry < old_parent_expiry {
                self.update_parent_earliest_expiry(&stack, old_parent_expiry, new_leaf_expiry);
            }

            root.leaf_count += 1;
            println!("----");
            return Ok((new_leaf_handle, None));
        }
    }

    pub fn update_parent_earliest_expiry(
        &mut self,
        stack: &[(NodeHandle, bool)],
        mut outdated_expiry: u64,
        mut new_expiry: u64,
    ) {
        // Walk from the top of the stack to the root of the tree.
        // Since the stack grows by appending, we need to iterate the slice in reverse order.
        for (parent_h, crit_bit) in stack.iter().rev() {
            let parent = self.node_mut(*parent_h).unwrap().as_inner_mut().unwrap();
            if parent.child_earliest_expiry[*crit_bit as usize] != outdated_expiry {
                break;
            }
            outdated_expiry = parent.earliest_expiry();
            parent.child_earliest_expiry[*crit_bit as usize] = new_expiry;
            new_expiry = parent.earliest_expiry();
        }
    }

    pub fn leaf_min_max(
        &self,
        find_max: bool,
        root: &OrderTreeRoot,
    ) -> Option<(NodeHandle, &LeafNode)> {
        let mut node_handle: NodeHandle = root.node()?;

        let i = usize::from(find_max);
        loop {
            let node_contents = self.node(node_handle)?;
            match node_contents.case()? {
                NodeRef::Inner(inner) => {
                    node_handle = inner.children[i];
                }
                NodeRef::Leaf(leaf) => {
                    return Some((node_handle, leaf));
                }
            }
        }
    }



}
