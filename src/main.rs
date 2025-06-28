#![allow(warnings)]

use crate::nodes::{AnyNode, LeafNode, NodeTag, NodeRef};
mod nodes;
mod ordertree;
mod error;

use ordertree::*;

const owner: [u8; 32] = [
        0x3a, 0x7b, 0xc4, 0x1f, 0x90, 0xaa, 0x2d, 0xfe,
        0x45, 0x10, 0x9e, 0xb3, 0x57, 0x6c, 0x11, 0xd2,
        0x8f, 0x22, 0x99, 0x03, 0x4e, 0xbf, 0x6a, 0xcd,
        0x18, 0xef, 0x34, 0x71, 0xb8, 0x5d, 0x2a, 0x7e,
    ];

fn get_key(price: u64) -> u128 {
    return nodes::new_node_key(nodes::Side::Bid, price, 1);
}

fn get_order(key: u128) -> LeafNode {
    return LeafNode::new(
        1,  
        key, 
        owner, 
        500, 
        1750435365, 
        65000, 
        17, 
        1 
    );
}

fn main() {
    


    let temp_anyNode = AnyNode {
        tag: 0, 
        data: [0; 79], 
        force_align: 0
    };

    let mut orderTreeRoot = OrderTreeRoot {
        maybe_node: 0, 
        leaf_count: 0
    };

    let mut orderTree = ordertree::OrderTreeNodes {
        order_tree_type: ordertree::OrderTreeType::Bids.into(), 
        bump_index: 0, 
        free_list_len: 0,  
        free_list_head: 0, 
        nodes: [temp_anyNode; 1024]
    };

    let key1 = get_key(5);
    let key2 = get_key(4);
    let key3 = get_key(6);
    let key4 = get_key(7);
    let key5 = get_key(10);
    let key6 = get_key(15);
    let key7 = get_key(1);
    let key8 = get_key(18);
    let key9 = get_key(20);
    let key10 = get_key(25);
    let key11 = get_key(31);
    let key12 = get_key(40);
    let key13 = get_key(47);
    let key14 = get_key(42);

    let new_order1 = get_order(key1); 
    let new_order2 = get_order(key2); 
    let new_order3 = get_order(key3); 
    let new_order4 = get_order(key4);
    let new_order5 = get_order(key5);
    let new_order6 = get_order(key6);
    let new_order7 = get_order(key7);
    let new_order8 = get_order(key8);
    let new_order9 = get_order(key9);
    let new_order10 = get_order(key10);
    let new_order11 = get_order(key11);
    let new_order12 = get_order(key12);
    let new_order13 = get_order(key13);
    let new_order14 = get_order(key14);

    orderTree.insert_leaf(&mut orderTreeRoot, &new_order1);
    orderTree.insert_leaf(&mut orderTreeRoot, &new_order2);
    orderTree.insert_leaf(&mut orderTreeRoot, &new_order3);
    orderTree.insert_leaf(&mut orderTreeRoot, &new_order4);
    orderTree.insert_leaf(&mut orderTreeRoot, &new_order5);
    orderTree.insert_leaf(&mut orderTreeRoot, &new_order6);
    orderTree.insert_leaf(&mut orderTreeRoot, &new_order7);
    //orderTree.insert_leaf(&mut orderTreeRoot, &new_order8);
    //orderTree.insert_leaf(&mut orderTreeRoot, &new_order9);
    //orderTree.insert_leaf(&mut orderTreeRoot, &new_order10);
    //orderTree.insert_leaf(&mut orderTreeRoot, &new_order11);
    //orderTree.insert_leaf(&mut orderTreeRoot, &new_order12);
    //orderTree.insert_leaf(&mut orderTreeRoot, &new_order13);
    //orderTree.insert_leaf(&mut orderTreeRoot, &new_order14);


    let worst = orderTree.leaf_min_max(true, &orderTreeRoot);

    println!("Root node: {}", orderTreeRoot.maybe_node);


    for i in (0..20) {
        let node: AnyNode = orderTree.nodes[i];
        let myNode = orderTree.node(i as u32).unwrap();

        match myNode.case() {
            None => {
                println!("poda");
            },
            Some(NodeRef::Inner(inner)) => {
                println!("Inner node: {}, 0: {}, 1: {} , Key: {}", i, inner.children[0], inner.children[1], price_data(inner.key));
            },

            Some(NodeRef::Leaf(leaf)) => {
                println!("Leaf node: {}, Price: {}", i, leaf.price_data());
            }

            _ => {
                //println!("shari ");
            },

        }


    }

    println!("The root target is: {}", orderTreeRoot.maybe_node);



}

pub fn price_data(key: u128) -> u64 {
        (key >> 64) as u64
} 

