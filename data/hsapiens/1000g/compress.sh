
echo "gzip"
# default zip compression level is 6
time gzip -k ERR047877_1.filt.head.fastq
time gzip -k ERR047877_2.filt.head.fastq

echo "xz"
time xz -k ERR047877_1.filt.head.fastq
time xz -k ERR047877_2.filt.head.fastq

echo "bzip2"
time bzip2 -k ERR047877_1.filt.head.fastq
time bzip2 -k ERR047877_2.filt.head.fastq

echo "lz4"
time lz4 -k ERR047877_1.filt.head.fastq
time lz4 -k ERR047877_2.filt.head.fastq

echo "zstd"
time zstd -k ERR047877_1.filt.head.fastq
time zstd -k ERR047877_2.filt.head.fastq
