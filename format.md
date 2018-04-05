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

All multi-byte values are little endian.

```

File {
    FileHeader
    [Block]
    FileFooter
}

FileHeader {
4B  u32  magic number (C S Q 26)
1B  u8   version number
8B  u64  length of data
4B  u32  offset to start of data
1B  u8   sequence enum (generic, Illumina, Pacbio)
1B  u8   quality score type (none, Phred+33, Phred+64)
4B  u32  read length (0 indicates variable length)
4B  u32  writer program commit digest (first 4 bytes)
3B  FieldsMeta
XB  ReadNameSchema
4B  u32  XxHash32 checksum of header
}

FieldsMeta {
1B  u8  read name compression enum (none, lz4, zstd)
1B  u8  sequence compression enum (none)
1B  u8  quality score compression enum (none, lz4, zstd)
}

ReadNameSchema {
48  u32   size of read name schema
XB  [u8]  read name schema string
}

Block {
    BlockHeader
    [Page]
    BlockFotter
}

BlockHeader {
1B  u8   block size (kb) in power of two (default 20, indicates 2^20 kb = 2024 kb)
8B  u64  number of pages
}

Page {
    PageHeader
    PageBody
    PageFooter
}

PageHeader {
2B  u16  number of bytes (max size is 64 kb)
         number of reads
         number of end positions (0 for sequence and quality if fixed-length reads)
         list of end positions of data for each read
         bitflags (page type 2 bits; compressed; fresh, continuation; ...)
}

PageBody {
   ReadNames | Sequences | Qualities
}

ReadNames {
         concatenated read names
}

Sequences {
         concatenated sequences in 2 bit encoding (00: A, 01: C, 10: G, 11: T)
}

Qualities {
         concatenated quality scores
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

## Comments

- each page contains only one type of data (names, sequences, or qualities)
- reads are sorted by implicit read number
- sequence and quality may be split into multiple blocks (e.g. PacBio data)
- compressed pages are packed into block until target size

Handling of N

option 1
- quality of 1 indicates sequence = N
- quality of 0 can be reserved for special purpose...

option 2
- to avoid vector resizing, N bases are replaced by A, but they will be masked over by N during decompression.
- number of N blocks
- (start of N block, end of N block)
