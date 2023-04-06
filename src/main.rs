use std::fs;


mod huffman;
use bytes::Bytes;
use huffman::{Huffman, HuffmanTree};


const FILE_NAME: &str = "test2";









fn main() {
    let file_data = Bytes::from(fs::read(format!("{FILE_NAME}.txt")).unwrap());
    let res = Huffman::compress(&file_data).unwrap();
    fs::write(format!("{FILE_NAME}.txt.cpr"), res).unwrap();

    let compressed_data = Bytes::from(fs::read(format!("{FILE_NAME}.txt.cpr")).unwrap());
    let (decompression, resulting_tree) = Huffman::decompress(&compressed_data);
    let decompression = decompression.unwrap();

    fs::write(format!("{FILE_NAME}.result.txt"), decompression).unwrap();

    let expected_treetree = Huffman::from_data(&Bytes::from(file_data.clone())).unwrap();

    println!("\n\n");
    println!("Expected:");
    expected_treetree.tree.borrow().print_hierarchy(0);
    println!("\n\n");
    println!("Got:");
    resulting_tree.borrow().print_hierarchy(0);
    println!("\n\n");
}