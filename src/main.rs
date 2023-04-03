use std::fs;


mod huffman;
use bytes::Bytes;
use huffman::Huffman;












fn main() {
    let file_data = Bytes::from(fs::read("example.txt").unwrap());
    let res = Huffman::compress(&file_data).unwrap();
    fs::write("example.txt.cpr", res).unwrap();
}