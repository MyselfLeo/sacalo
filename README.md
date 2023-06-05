# Crappy Algorithm for Compressing Admittedly Large Objects (Sacalo)

Believe it or not, it kinda works for large files. The file `example.txt`, for _example_, get compressed by a whooping **_46%!!_**

## Usage

```bash
$ sacalo compress FILE [OUTPUT]
```
```bash
$ sacalo decompress FILE.scl [OUTPUT]
```

The output file name is optional.

## Install 

Download the source and compile it yourself (fairly simple with Rust, run `cargo build` from the repository root).
Why would you, tho?


## License

This project is licensed under **Mozilla Public License 2.0**. See `LICENSE.txt`.