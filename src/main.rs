use std::io::Write;
use std::rc::Rc;
use std::path::Path;
use std::fs;


mod huffman;
use bytes::Bytes;
use huffman::Huffman;












fn main() {
    let file_data = Bytes::from(fs::read("example.txt").unwrap());
    let huffman = Huffman::from_data(&file_data).unwrap();
    fs::write("test", huffman.tree.serialise()).unwrap();
}
