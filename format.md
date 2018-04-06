# Compact Sequence and Quality Format

## Feature specification

* overall aim: to be superior to gzip for compressing fastq files in most aspects
* streaming: one-pass read
* fast linear access; reasonable random access
* fast decoding; reasonable compression
* low memory requirement
* early detection of truncated data
* data integrity check and error recovery with limited data loss
* support for short or long reads of fixed or variable length
* support for 'N' bases
* optional lossy compression of quality score into 16 bins

## Format specification

- all multi-byte values are little endian
- `[a]` denotes array of elements of type `a`; empty arrays are skipped
- each page contains only one type of data; possible page types: names, sequenes, qualities
- max page body size before compression is 64 kb (cannot be increased without changing u16 types)
- pages may be encoded and/or compressed; they are then written back-to-back within a block until maximum block size
- default maximum block size is 2048 kb
- long sequence and quality may be split into multiple pages (e.g. PacBio data);
  first page is `fresh` and remaining are `continued`
- page types are interleaved in the order: names, sequences, qualities
- continuation pages are to be concatenated with the last page of the same type
- example read name schema: `@{enum}:{u16}:{enum}:{u8}:{uint}:{uint}:{uint} {u8}:{char}:{u16}:{str}`
- to avoid vector resizing, N bases are replaced by A, but they will be masked over by N during decompression
- sequencing encoding `bitpack2`: bitpacked in 2 bit encoding (00: A, 01: C, 10: G, 11: T)
- quality encoding `lossy_bitpack4`: binned into 16 bins and bitpacked in 4 bits

```
File {
    FileHeader
    [Block]
    FileFooter
}

FileHeader {
4B  u32  magic number (C S Q 26)
1B  u8   version number
8B  u64  total number of bytes in data blocks
4B  u32  offset to start of data
4B  u32  writer program commit digest (first 4 bytes)
    FieldsMeta
4B  u32  XxHash32 checksum of header
}

FieldsMeta {
4B  u32  read length (0 indicates variable length)
1B  u8   read name type enum (none, schema)
1B  u8   read name encoding (plain)
1B  u8   read name compression enum (none, lz4, zstd)
1B  u8   sequence type enum (none, generic, Illumina, Pacbio)
1B  u8   sequence encoding (plain, bitpack2)
1B  u8   sequence compression enum (none, lz4, zstd)
1B  u8   quality score type enum (none, Phred+33, Phred+64)
1B  u8   quality score encoding (plain, lossy_bitpack4)
1B  u8   quality score compression enum (none, lz4, zstd)
    ReadNameSchema
}

ReadNameSchema {
4B  u32   length of read name schema
XB  [u8]  read name schema string
}

Block {
    BlockHeader
    [Page]
    BlockFooter
}

BlockHeader {
4B  u64  block size after compression
4B  u64  number of pages
}

Page {
    PageHeader
    PageBody
    PageFooter
}

PageHeader {
1B  u8     bitflags (page type, 2 bits; fresh, continued; ...)
2B  u16    number of bytes in page body after compression
2B  u16    number of bytes in page body before compression
2B  u16    number of variable-length reads (0 if fixed read length)
XB  [u16]  array of end positions of data for each read (skipped if fixed read length)
2B  u16    number of stretches of N bases
XB  [u16]  array of start and end positions of stretches of N bases
}

PageBody {
    Names | Sequences | Qualities
}

Names {
XB  [u8]  concatenated read names
}

Sequences {
XB  [u8]  concatenated sequences
}

Qualities {
XB  [u8]  concatenated quality scores
}

PageFooter {
4B  u64  XxHash32 checkum of page (or 0 if checksum is disabled)
}

BlockFooter {
1B  u8  end of block marker (23)
}

FileFooter {
1B  u8  end of file marker (0)
}
```
