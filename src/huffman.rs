use std::rc::{Rc, Weak};
use std::cell::RefCell;
use bytes::{BytesMut, BufMut, Bytes, Buf};




/// Structure storing a [HuffmanTree] and allowing retrieval of the path from the root to a particular leaf
pub struct Huffman {
    pub tree: Rc<RefCell<HuffmanTree>>,
    pub leaves: Vec<Rc<RefCell<HuffmanTree>>>
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
        let mut nodes: Vec<Rc<RefCell<HuffmanTree>>> = Vec::new();
        for (byte, occurence) in occurences.iter().enumerate() {
            if occurence > &0 {
                nodes.push(Rc::new(RefCell::new(HuffmanTree::Leaf(*occurence, None, false, byte as u8))));
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


            let node_1 = nodes.pop().expect("Unexpected end of node vector");
            let node_2 = nodes.pop().expect("Unexpected end of node vector");


            let new_weight = node_1.borrow().get_weight() + node_2.borrow().get_weight();
            let new_node = Rc::new(RefCell::new(HuffmanTree::Node(new_weight, None, false, Rc::clone(&node_1), Rc::clone(&node_2))));


            // update children
            match node_1.try_borrow_mut() {
                Ok(mut borrow) => {
                    (*borrow).set_parent(Some(Rc::downgrade(&new_node)), true)
                },
                Err(_) => panic!(),
            }


            match node_2.try_borrow_mut() {
                Ok(mut borrow) => {
                    (*borrow).set_parent(Some(Rc::downgrade(&new_node)), false)
                },
                Err(_) => panic!(),
            }


            // insert back at the correct position
            nodes.push(new_node);
            nodes.sort();
        }


        // now, we can return the whole structure
        Some(Huffman {
            tree: nodes[0].clone(),
            leaves,
        })
    }



    fn from_tree(tree: Rc<RefCell<HuffmanTree>>) -> Huffman {
        let mut leaves = vec![];
        let mut buf = vec![tree.clone()];

        while !buf.is_empty() {
            let parent = buf.pop().unwrap();
            let parent_deref = &*parent.borrow();
            
            match parent_deref {
                HuffmanTree::Node(_, _, _, lc, rc) => {
                    buf.push(lc.clone());
                    buf.push(rc.clone());
                },
                HuffmanTree::Leaf(_, _, _, _) => leaves.push(parent.clone()),
            }
        }

        Huffman { tree: tree.clone(), leaves: leaves }
    }




    /// return the path to the given data
    fn get_path(&self, data: u8) -> Option<Vec<bool>> {
        let mut leaf_id = -1;
        for (i, b) in self.get_all_bytes().iter().enumerate() {
            if *b == data {leaf_id = i as isize; break}
        }
        if leaf_id == -1 {return None}

        let mut node = self.leaves[leaf_id as usize].clone();

        let mut res = vec![];
        while node.borrow().get_parent().is_some() {
            res.push(node.borrow().get_left_right());
            let parent = node.borrow().get_parent().unwrap();
            node = parent.upgrade().unwrap();
        }

        Some(res)
    }


    /// Return the list of all bytes present in the tree
    fn get_all_bytes(&self) -> Vec<u8> {
        let mut res = vec![];

        for l in &self.leaves {
            res.push(l.borrow().get_data().unwrap())
        }

        res
    }



    /// Return the bytes representing the given data
    fn encode(&self, data: &Bytes) -> Option<Bytes> {
        let mut res = BytesMut::new();

        // encode each bytes
        let mut byte: u8 = 0;
        let mut nb_bits_written = 0;
        for  b in data {

            for left in self.get_path(*b)? {
                // byte is filled, we push it to the result BytesMut
                if nb_bits_written == 8 {
                    res.put_u8(byte);
                    nb_bits_written = 0;
                    byte = 0;
                }

                // write the bit to the current byte
                if left {byte |= 1 << nb_bits_written;}
                else {} // the bit is already at 0, nothing to do

                nb_bits_written += 1;
            }
        }
        if nb_bits_written > 0 {res.put_u8(byte)}


        Some(res.freeze())
    }



    /// Compress the data
    pub fn compress(data: &Bytes) -> Option<Bytes> {
        let mut res = BytesMut::new();
        
        let tree = Huffman::from_data(data)?;
        let tree_rep = tree.tree.borrow().serialise();
        let encoded_data = tree.encode(data)?;

        res.extend_from_slice(&tree_rep);
        res.extend_from_slice(&encoded_data);

        println!("compressed data length: {}", res.len());

        Some(res.freeze())
    }


    /// Decompress the data
    pub fn decompress(data: &Bytes) -> Option<Bytes> {
        let mut data = data.clone();

        // retrieve the huffman tree
        let tree_data_weight = data.get_u128();
        let tree_data_length = data.get_u16();
        println!("tree data length: {tree_data_length}");
        println!("data length: {}", data.len());
        
        let mut tree_data = BytesMut::new();
        tree_data.put_u128(tree_data_weight);
        tree_data.put_u16(tree_data_length);
        println!("1");
        tree_data.extend_from_slice(&data.slice(0..tree_data_length as usize));

        let mut compressed_data = BytesMut::new();
        println!("2");
        compressed_data.extend_from_slice(&data.slice(tree_data_length as usize..data.len()));

        println!("3");

        let tree = HuffmanTree::deserialise(tree_data.freeze()).unwrap();

        println!("4");


        // now, we decompress the data. We iter throught each bit until we meet a leaf
        let mut current_node = tree.clone();
        let mut res = BytesMut::new();

        for b in compressed_data {
            for i in 0..8 {

                let new_node;
                match &*current_node.borrow() {
                    HuffmanTree::Node(_, _, _, cl, cr) => {
                        let bit = (b & 1 << i) == (1 << i);
                        new_node = if bit {cl.clone()} else {cr.clone()}
                    },
                    
                    HuffmanTree::Leaf(_, _, _, d) => {
                        res.put_u8(*d);
                        new_node = tree.clone();
                    },
                }

                current_node = new_node;
            }
        }
        
        return Some(res.freeze())
    }
}




/// Represents a Huffman tree, used to store and access bytes based on their number of occurences
/// in the file.
#[derive(Debug)]
pub enum HuffmanTree {
    Node(u128, Option<Weak<RefCell<HuffmanTree>>>, bool, Rc<RefCell<HuffmanTree>>, Rc<RefCell<HuffmanTree>>), // weight, ref to parent, left or right, left branch, right branch
    Leaf(u128, Option<Weak<RefCell<HuffmanTree>>>, bool, u8)                                                  // weight, ref to parent, left or right, data byte
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
                let left_bytes = left_branch.borrow().serialise();
                let right_bytes = right_branch.borrow().serialise();

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
    pub fn deserialise(mut bytes: Bytes) -> Result<Rc<RefCell<HuffmanTree>>, String> {
        // first 2 bytes are the size of the tree branch. If bytes.len() != 2 + size, the
        // bytes are not valid
        println!("a");
        let bytes_len = bytes.len();

        let weight = bytes.get_u128();
        let size = bytes.get_u16();
        if bytes_len != (size + 18) as usize {return Err(format!("Invalid bytes size (expected {}, got {})", size, bytes_len))}

        println!("b");
        // case of a leaf
        if size == 1 {
            Ok(Rc::new(RefCell::new(HuffmanTree::Leaf(weight, None, false, bytes.get_u8()))))
        }

        // case of a node
        else {
            // weight & size of the first branch
            let left_weight = bytes.get_u128();
            let left_size = bytes.get_u16();
            let mut left_branch = BytesMut::with_capacity(18 + left_size as usize);
            left_branch.put_u128(left_weight);
            left_branch.put_u16(left_size);
            println!("5");
            left_branch.extend_from_slice(bytes.slice(0..(left_size as usize)).as_ref());

            println!("6");
            let right_branch = bytes.slice((left_size as usize)..bytes.len());

            println!("7");

            let left = HuffmanTree::deserialise(left_branch.freeze())?;
            let right = HuffmanTree::deserialise(right_branch)?;


            let new = Rc::new(RefCell::new(HuffmanTree::Node(weight, None, false, Rc::clone(&left), Rc::clone(&right))));


            // update children
            match left.try_borrow_mut() {
                Ok(mut borrow) => {
                    (*borrow).set_parent(Some(Rc::downgrade(&new)), true)
                },
                Err(_) => panic!(),
            }
            match right.try_borrow_mut() {
                Ok(mut borrow) => {
                    (*borrow).set_parent(Some(Rc::downgrade(&new)), false)
                },
                Err(_) => panic!(),
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


    pub fn get_parent(&self) -> Option<Weak<RefCell<HuffmanTree>>> {
        match self {
            HuffmanTree::Node(_, p, _, _, _) => p.clone(),
            HuffmanTree::Leaf(_, p, _, _) => p.clone(),
        }
    }


    pub fn get_left_right(&self) -> bool {
        match self {
            HuffmanTree::Node(_, _, lr, _, _) => *lr,
            HuffmanTree::Leaf(_, _, lr, _) => *lr,
        }
    }


    pub fn set_parent(&mut self, parent: Option<Weak<RefCell<HuffmanTree>>>, left_right: bool) {
        match self {
            HuffmanTree::Node(_, p, lr, _, _) => {
                *p = parent;
                *lr = left_right;
            },
            HuffmanTree::Leaf(_, p, lr, _) => {
                *p = parent;
                *lr = left_right;
            },
        }
    }


    pub fn get_data(&self) -> Option<u8> {
        match self {
            HuffmanTree::Node(_, _, _, _, _) => None,
            HuffmanTree::Leaf(_, _, _, d) => Some(*d),
        }
    }

}



impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.get_weight().partial_cmp(&self.get_weight())
    }
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.get_weight().cmp(&self.get_weight())
    }
}


impl PartialEq for HuffmanTree {
    fn eq(&self, other: &Self) -> bool {
        self.get_weight() == other.get_weight()
    }
}

impl Eq for HuffmanTree {}