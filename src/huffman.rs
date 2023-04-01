use std::rc::Rc;
use bytes::{BytesMut, BufMut, Bytes, Buf};




/// Structure storing a [HuffmanTree] and allowing retrieval of the path from the root to a particular leaf
pub struct Huffman {
    pub tree: Rc<HuffmanTree>,
    pub leaves: Vec<Rc<HuffmanTree>>
}


impl Huffman {
    /// Return the [Huffman] struct corresponding to the given data.
    /// see https://en.wikipedia.org/wiki/Huffman_Coding
    pub fn from_data(data: &Bytes) -> Option<Huffman> {
        if data.len() == 0 {return None}

        // count the nb. of occurences of each bytes
        let mut occurences: [u128; 256] = [0; 256];
        for b in data {occurences[*b as usize] += 1;}

        // creation of the leaves of the tree
        let mut nodes: Vec<Rc<HuffmanTree>> = Vec::new();
        for (byte, occurence) in occurences.iter().enumerate() {
            if occurence > &0 {
                nodes.push(Rc::new(HuffmanTree::Leaf(*occurence, None, false, byte as u8)));
            }
        }
        // sort leaves by weight (decreasing order as specified by HuffmanTree's PartialOrd impl)
        nodes.sort();

        // copy vec of references of the leaves for futur use
        let leaves = nodes.clone();
        
        // now, we'll create nodes from the 2 nodes with the lowest weight (last 2 nodes),
        // give it as weight the sums of the 2 nodes' weight and insert this new node
        // at its sorted position in the vector.
        // at the end, we'll only have one node, the root of the HuffmanTree tree.
        while nodes.len() > 1 {

            let mut node_1 = nodes.pop().expect("Unexpected end of node vector");
            let mut node_2 = nodes.pop().expect("Unexpected end of node vector");

            let new_node = Rc::new(HuffmanTree::Node(node_1.get_weight() + node_2.get_weight(), None, false, node_1, node_2));

            // update children
            match Rc::get_mut(&mut node_1).unwrap() {
                HuffmanTree::Node(_, parent, left_right, _, _) => {
                    *parent = Some(new_node.clone());
                    *left_right = true;
                },
                HuffmanTree::Leaf(_, parent, left_right, _) => {
                    *parent = Some(new_node.clone());
                    *left_right = true;
                },
            }

            // update children
            match Rc::get_mut(&mut node_2).unwrap() {
                HuffmanTree::Node(_, parent, left_right, _, _) => {
                    *parent = Some(new_node.clone());
                    *left_right = false;
                },
                HuffmanTree::Leaf(_, parent, left_right, _) => {
                    *parent = Some(new_node.clone());
                    *left_right = false;
                },
            }


            // insert back at the correct position of the vector
            match nodes.binary_search(&new_node) {
                Ok(_) => {unreachable!()}
                Err(pos) => nodes.insert(pos, new_node)
            }
        }

        // now, we can return the whole structure
        Some(Huffman {
            tree: nodes[0].clone(),
            leaves,
        })
    }
}




/// Represents a Huffman tree, used to store and access bytes based on their number of occurences
/// in the file.
#[derive(PartialEq, Eq, Ord)]
pub enum HuffmanTree {
    Node(u128, Option<Rc<HuffmanTree>>, bool, Rc<HuffmanTree>, Rc<HuffmanTree>), // weight, ref to parent, left or right, left branch, right branch
    Leaf(u128, Option<Rc<HuffmanTree>>, bool, u8)                                // weight, ref to parent, left or right, data byte
}



impl HuffmanTree {
    /// Return a [Bytes] representing the HuffmanTree tree in bytes which can them be deserialised.
    pub fn serialise(&self) -> Bytes {

        // For each elements, the first two bytes represent the size of the branch.
        // if those 2 bytes == 1, the next byte is the data, and the branch is finished.
        match self {
            HuffmanTree::Leaf(weight, _, _, data) => {
                let mut res = BytesMut::with_capacity(19);
                res.put_u128(*weight);
                res.put_u16(1);
                res.put_u8(*data);

                res.freeze()
            },

            HuffmanTree::Node(weight, _, _, left_branch, right_branch) => {
                let left_bytes = left_branch.serialise();
                let right_bytes = right_branch.serialise();

                let mut res = BytesMut::with_capacity(18 + left_bytes.len() + right_bytes.len());

                res.put_u128(*weight);
                
                let size_left: u16 = left_bytes.len().try_into().unwrap();
                let size_right: u16 = right_bytes.len().try_into().unwrap();
                res.put_u16(size_left + size_right);
                
                res.extend_from_slice(left_bytes.as_ref());
                res.extend_from_slice(right_bytes.as_ref());

                res.freeze()
            }
        }
    }



    /// Create a HuffmanTree tree based on a stream of bytes (Bytes).
    pub fn deserialise(mut bytes: Bytes) -> Result<Rc<HuffmanTree>, String> {
        // first 2 bytes are the size of the tree branch. If bytes.len() != 2 + size, the
        // bytes are not valid
        let bytes_len = bytes.len();
        let weight = bytes.get_u128();
        let size = bytes.get_u16();
        if bytes_len != (size + 18) as usize {return Err(format!("Invalid bytes size (expected {}, got {})", size, bytes_len))}

        
        // case of a leaf
        if size == 1 {
            Ok(Rc::new(HuffmanTree::Leaf(weight, None, false, bytes.get_u8())))
        }

        // case of a node
        else {
            // weight & size of the first branch
            let left_weight = bytes.get_u128();
            let left_size = bytes.get_u16();
            let mut left_branch = BytesMut::with_capacity(18 + left_size as usize);
            left_branch.put_u128(left_weight);
            left_branch.put_u16(left_size);
            left_branch.extend_from_slice(bytes.slice(0..(left_size as usize)).as_ref());

            let right_branch = bytes.slice((left_size as usize)..bytes.len());

            let mut left = HuffmanTree::deserialise(left_branch.freeze())?;
            let mut right = HuffmanTree::deserialise(right_branch)?;


            let new = Rc::new(HuffmanTree::Node(weight, None, false, left, right));


            // update children
            match Rc::get_mut(&mut left).unwrap() {
                HuffmanTree::Node(_, parent, left_right, _, _) => {
                    *parent = Some(new.clone());
                    *left_right = true;
                },
                HuffmanTree::Leaf(_, parent, left_right, _) => {
                    *parent = Some(new.clone());
                    *left_right = true;
                },
            }
            match Rc::get_mut(&mut right).unwrap() {
                HuffmanTree::Node(_, parent, left_right, _, _) => {
                    *parent = Some(new.clone());
                    *left_right = false;
                },
                HuffmanTree::Leaf(_, parent, left_right, _) => {
                    *parent = Some(new.clone());
                    *left_right = false;
                },
            }


            Ok(new)
        }
    }





    pub fn get_weight(&self) -> u128 {
        match self {
            HuffmanTree::Node(w, ..) => *w,
            HuffmanTree::Leaf(w, ..) => *w,
        }
    }

}



impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.get_weight().partial_cmp(&self.get_weight())
    }
}