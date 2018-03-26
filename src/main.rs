extern crate bio;
extern crate lz4;
extern crate zstd;
extern crate byteorder;

mod nucl;
mod illumina;

use std::iter::FromIterator;
use std::env;
use std::fs::File;
use std::io::{Result, Read, Write};
use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};

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

    let name = match rec.desc() {
        None => rec.id().to_owned() + "\n",
        Some(s) => rec.id().to_owned() + " " + s + "\n",
    };

    let len_offset = rec.seq().len();
    let annot0 = parse_name(&name).unwrap();

    let mut len_diffs: Vec<u16> = Vec::new();
    let mut names: Vec<u8> = Vec::new();
    let mut seq: Vec<u8> = Vec::new();
    let mut qual: Vec<u8> = Vec::new();

    let mut seqs: Vec<Vec<u8>> = Vec::new();
    let mut metas = Vec::new();
    let mut n = 0;
    let reader = fastq::Reader::new(try!(File::open(src)));
    for r in reader.records() {
        let rec = r.ok().expect("Error reading record.");

        let name = match rec.desc() {
            None => rec.id().to_owned() + "\n",
            Some(s) => rec.id().to_owned() + " " + s + "\n",
        };

        let annot = parse_name(&name).unwrap();

        //struct ReadAnnotation(u8, u16, u32, u32, u8);
        
        metas.write_u8(annot.0).unwrap();

        metas.write_u16::<LittleEndian>(annot.1).unwrap();
        metas.write_u32::<LittleEndian>(annot.2).unwrap();
        metas.write_u32::<LittleEndian>(annot.3).unwrap();
        
        // NB this would probably compromise random access...
        // differences were also harder to compressed with zstd
        //metas.write_i8((annot.1 as i16 - annot0.1 as i16) as i8).unwrap();
        //metas.write_i16::<LittleEndian>((annot.2 as i32 - annot0.2 as i32) as i16).unwrap();
        //metas.write_i16::<LittleEndian>((annot.3 as i32 - annot0.3 as i32) as i16).unwrap();
        
        metas.write_u8(annot.4).unwrap();


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

    let mut encoder = try!(zstd::stream::Encoder::new(Vec::new(), 19));
    try!(encoder.write_all(&metas));
    let cmetas = encoder.finish().unwrap();

    println!("reads: {}", n);
    println!("before: names: {}, seq: {}, qual: {}", names.len(), seq.len(), qual.len());
    println!("pseq: {}, pqual: {}, metas: {}", pseq.len(), pqual.len(), metas.len());
    println!("cmetas: {}", cmetas.len());
    println!("cbqual: {}", cbqual.len());
    println!("after: names: {}, seq: {}, qual: {}", cnames.len(), cseq.len(), cqual.len());

    let mut fo = try!(File::create(dst));
    //fo.write_all(&cnames);
    fo.write_all(&cmetas);
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

/// Match literal and return rest of string
/*
fn parse_literal(x: &str, literal: &str) -> Option<((), &str)> {
    if x.len() > literal.len() {
        if x[0..literal.len()] == literal {
            Some((), &x[literal.len()..])
        } else {
            None
        }
    } else if x.len() == literal.len() {
        Some((), &[])
    }
    None
}
*/

struct ReadAnnotation(u8, u16, u32, u32, u8);

// ERR047877.10 FCB09RWABXX:1:1101:1345:2223/1
// {sample}.{read_idx} {instructment}:{lane}:{tile}:{x_pos}:{y_pos}/{pair}
// {literal}.{increment} {literal}:{u8}:{u16}:{u32}:{u32}/{u8}
fn parse_name(x: &str) -> Option<ReadAnnotation>  {
    match x.find(':' ) {
        None => None,
        Some(i) => {
            let x = &x[(i+1)..];
            match x.find(':') {
                None => None, 
                Some(j) => {
                    let a1 = x[..j].parse::<u8>().unwrap();
                    let x = &x[(j+1)..];
                    match x.find(':') {
                        None => None,
                        Some(j) => {
                            let a2 = x[..j].parse::<u16>().unwrap();
                            let x = &x[(j+1)..];
                            match x.find(':') {
                                None => None,
                                Some(j) => {
                                    let a3 = x[..j].parse::<u32>().unwrap();
                                    let x = &x[(j+1)..];
                                    match x.find('/') {
                                        None => None,
                                        Some(j) => {
                                            let a4 = x[..j].parse::<u32>().unwrap();
                                            let x = &x[(j+1)..];
                                            // strip new line
                                            let a5 = x[..x.len()-1].parse::<u8>().unwrap();
                                            Some(ReadAnnotation(a1, a2, a3, a4, a5))
                                        },
                                    }
                                },
                            }
                        },
                    }
                },
            }
        },
    }
}

