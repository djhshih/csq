extern crate bio;
extern crate lz4;
extern crate zstd;

mod nucl;
mod illumina;

use std::iter::FromIterator;
use std::env;
use std::fs::File;
use std::io::{Result, Read, Write};
use std::path::Path;

use bio::io::fastq;


fn main() {
    println!("LZ4 version: {}", lz4::version());
    let suffix = ".csq";

    for arg in Vec::from_iter(env::args())[1..].iter() {
        if arg.ends_with(suffix) {
            //decompress(&Path::new(arg), &Path::new(&arg[0..arg.len() - suffix.len()])).unwrap();
        } else {
            //compress(&Path::new(arg), &Path::new(&(arg.to_string() + suffix))).unwrap();
            compress_fastq(&Path::new(arg), &Path::new(&(arg.to_string() + suffix))).unwrap();
        }
    }
}

fn compress_fastq(src: &Path, dst: &Path) -> Result<()> {
    let reader = fastq::Reader::new(try!(File::open(src)));

    // parse first record
    let rec = reader.records().next().expect("No records")
        .ok().expect("Error read frist record.");

    let len_offset = rec.seq().len();

    let mut len_diffs: Vec<u16> = Vec::new();
    let mut names: Vec<u8> = Vec::new();
    let mut seq: Vec<u8> = Vec::new();
    let mut qual: Vec<u8> = Vec::new();

    let mut seqs: Vec<Vec<u8>> = Vec::new();

    let mut n = 0;
    let reader = fastq::Reader::new(try!(File::open(src)));
    for r in reader.records() {
        let rec = r.ok().expect("Error reading record.");

        let name = match rec.desc() {
            None => rec.id().to_owned() + "\n",
            Some(s) => rec.id().to_owned() + " " + s + "\n",
        };

        len_diffs.push((rec.seq().len() - len_offset) as u16);
        qual.extend(rec.qual());
        seq.extend(rec.seq());
        //seqs.push(rec.seq().to_owned());
        n += 1;

        names.extend(name.as_bytes());
    }

    /*
    seqs.sort();
    let mut nlines = 0;
    for s in seqs.iter() {
        nlines += 1;
        seq.extend(s);
    }
    */

    let mut encoder = try!(nucl::encoder::EncoderBuilder::new().build(Vec::new()));
    try!(encoder.write_all(&seq));
    let pseq = match encoder.finish() {
        (encoded, _) => encoded
    };

    println!("{:?}", std::str::from_utf8(&seq[..1000]).unwrap());
    println!("{:?}", &pseq[..1000]);

    let mut encoder = try!(illumina::encoder::EncoderBuilder::new().build(Vec::new()));
    try!(encoder.write_all(&qual));
    let pqual = match encoder.finish() {
        (encoded, _) => encoded
    };

    /*
    let mut encoder = try!(lz4::EncoderBuilder::new().build(Vec::new()));
    try!(encoder.write_all(&names));
    let cnames = match encoder.finish() {
        (encoded, _) => encoded
    };

    let mut encoder = try!(lz4::EncoderBuilder::new().build(Vec::new()));
    try!(encoder.write_all(&pseq));
    let cseq = match encoder.finish() {
        (encoded, _) => encoded
    };

    let mut encoder = try!(lz4::EncoderBuilder::new().build(Vec::new()));
    try!(encoder.write_all(&qual));
    let cqual = match encoder.finish() {
        (encoded, _) => encoded
    };
    */

    let mut encoder = try!(zstd::stream::Encoder::new(Vec::new(), 0));
    try!(encoder.write_all(&pseq));
    let cseq = encoder.finish().unwrap();

    let mut encoder = try!(zstd::stream::Encoder::new(Vec::new(), 0));
    try!(encoder.write_all(&pqual));
    let cqual = encoder.finish().unwrap();

    /*
    let mut encoder = try!(lz4::EncoderBuilder::new().build(Vec::new()));
    try!(encoder.write_all(&pqual));
    let cqual = match encoder.finish() {
        (encoded, _) => encoded
    };
    */

    let bqual: Vec<u8> = qual.iter().map(|&x| illumina::qual::bin(x)).collect();
    let mut encoder = try!(zstd::stream::Encoder::new(Vec::new(), 0));
    try!(encoder.write_all(&bqual));
    let cbqual = encoder.finish().unwrap();

    let mut encoder = try!(zstd::stream::Encoder::new(Vec::new(), 19));
    try!(encoder.write_all(&names));
    let cnames = encoder.finish().unwrap();

    println!("reads: {}", n);
    println!("before: names: {}, seq: {}, qual: {}", names.len(), seq.len(), qual.len());
    println!("pseq: {}, pqual: {}", pseq.len(), pqual.len());
    println!("cbqual: {}", cbqual.len());
    println!("after: names: {}, seq: {}, qual: {}", cnames.len(), cseq.len(), cqual.len());

    let mut fo = try!(File::create(dst));
    fo.write_all(&cnames);
    fo.write_all(&cseq);
    fo.write_all(&cqual);

    Ok(())
}

fn compress(src: &Path, dst: &Path) -> Result<()> {
    println!("Compression : {:?} -> {:?}", src, dst);
    let mut fi = try!(File::open(src));
    let mut fo = try!(lz4::EncoderBuilder::new().build(try!(File::create(dst))));
    try!(copy(&mut fi, &mut fo));
    match fo.finish() {
        (_, result) => result
    }
}

fn decompress(src: &Path, dst: &Path) -> Result<()> {
    println!("Decompressing: {:?} -> {:?}", src, dst);
    let mut fi = try!(lz4::Decoder::new(try!(File::open(src))));
    let mut fo = try!(File::create(dst));
    copy(&mut fi, &mut fo)
}

fn copy(src: &mut Read, dst: &mut Write) -> Result<()> {
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        // copy a chunk of bytes at a time
        let len = try!(src.read(&mut buffer));
        if len == 0 {
            break;
        }
        try!(dst.write_all(&buffer[0..len]));
    }
    Ok(())
}
