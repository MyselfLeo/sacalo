use std::fs;


mod huffman;
use bytes::Bytes;
use huffman::Huffman;












fn main() {
    let file_data = Bytes::from(fs::read("example.txt").unwrap());
    let huffman = Huffman::from_data(&file_data).unwrap();

    let tree_data = huffman.tree.borrow().serialise();
    fs::write("test", tree_data).unwrap();

    let new_huffman = 

    println!("Data stored: {:?}", huffman.get_all_bytes());
    println!("{:?}", huffman.get_path(101));
}