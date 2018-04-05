# Compact Sequence and Quality Format

## Feature specification

* streaming: one-pass read
* fast linear access; reasonable random access
* fast decoding; reasonable compression
* low memory requirement
* data integrity check and error recovery with limited data loss
* support short or long reads of fixed or variable length
* support 'N' bases
* optional lossy compression of quality score into 16 bins
* superior to gzip in most aspects

## Format specification

- all multi-byte values are little endian
- reads are sorted by implicit read number
- each page contains only one type of data (names, sequences, or qualities)
- max page size before compression is 64 kb (cannot be increased without changing u16 types)
- default maximum block size is 2048 kb
- compressed pages are packed into block until maximum block size
- sequence and quality may be split into multiple blocks (e.g. PacBio data)
- to avoid vector resizing, N bases are replaced by A, but they will be masked over by N during decompression

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
1B  u8   sequence compression enum (none)
1B  u8   sequence enum (generic, Illumina, Pacbio)
1B  u8   quality score compression enum (none, lz4, zstd)
1B  u8   quality score enum (none, Phred+33, Phred+64)
1B  u8   read name compression enum (none, lz4, zstd)
    ReadNameSchema
}

ReadNameSchema {
48  u32   length of read name schema
XB  [u8]  read name schema string (e.g. @{enum}:{u16}:{enum}:{u8}:{uint}:{uint}:{uint} {u8}:{char}:{u16}:{str})
}

Block {
    BlockHeader
    [Page]
    BlockFooter
}

BlockHeader {
8B  u64  block size after compression
8B  u64  number of pages
}

Page {
    PageHeader
    PageBody
    PageFooter
}

PageHeader {
2B  u16    number of bytes after compression
1B  u8     bitflags (page type 2 bits; fresh, continuation; ...)
2B  u16    number of reads
XB  [u16]  list of end positions of data for each read (0 if read length is fixed)
2B  u16    number of stretches of N bases
XB  [u16]  list of start and end positions of stretches of N bases
}

PageBody {
    ReadNames | Sequences | Qualities
}

ReadNames {
XB  [u8]  concatenated read names, possibly compressed
}

Sequences {
XB  [u8]  concatenated bitpacked sequences in 2 bit encoding (00: A, 01: C, 10: G, 11: T)
}

Qualities {
XB  [u8]  concatenated quality scores, possibly compressed
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
