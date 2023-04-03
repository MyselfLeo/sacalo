use std::fs;


mod huffman;
use bytes::Bytes;
use huffman::Huffman;












fn main() {
    let file_data = Bytes::from(fs::read("example2.txt").unwrap());
    let res = Huffman::compress(&file_data).unwrap();
    fs::write("example2.txt.cpr", res).unwrap();

    let compressed_data = Bytes::from(fs::read("example2.txt.cpr").unwrap());
    let decompression = Huffman::decompress(&compressed_data).unwrap();
    fs::write("result.txt", decompression).unwrap();
}