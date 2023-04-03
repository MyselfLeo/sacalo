use std::fs;


mod huffman;
use bytes::Bytes;
use huffman::{Huffman, HuffmanTree};












fn main() {
    /*let file_data = Bytes::from(fs::read("example2.txt").unwrap());
    let res = Huffman::compress(&file_data).unwrap();
    fs::write("example2.txt.cpr", res).unwrap();

    let compressed_data = Bytes::from(fs::read("example2.txt.cpr").unwrap());
    let decompression = Huffman::decompress(&compressed_data).unwrap();
    fs::write("result.txt", decompression).unwrap();*/
    let bytes = vec![1, 34, 64, 124, 255, 1, 1];
    let tree = Huffman::from_data(&Bytes::from(bytes.clone())).unwrap();
    println!("registered bytes: {:?}", tree.get_all_bytes());

    for b in bytes {
        println!("Current byte: {b}");
        let path = tree.get_path(b).unwrap();
        println!("\tPath: {:?}", path);
        let data = tree.get_data_from_path(path).unwrap();
        println!("\t resulting data: {data}");
    }


}