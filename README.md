# Santa - an ELF packer/loader

I realised that you can call `execve` on a `memfd` via `/proc/<pid>/fd/n`. Of
course, it figures that someone else created a cool tool to do this, too. I
took most inspiration from [this blog post](https://www.guitmz.com/running-elf-from-memory/).

TL;DR: `santa` will use `cargo` to compile a given executable into another. The
new executable will contain the existing executable, but zipped and `xor`'d
with a random key. On running the new executable, it will decrypt and
decompress the original to a `memfd` and then execute it with `exec`.

This was a fun project to learn some more about Rust. I may yet extend this
tool to actually directly load the ELF into memory, rather than the `memfd` ->
`exec` route. _Could_ also extend it to incorporate a template binary that
uses predefined sections, to remove the reliance on `cargo`. Then we'd just
use goblin to insert our new blob and key into the template binary.

This repo also contains some binaries to test the tool in `example`.

## Usage
```
$ cargo build
$ pushd example; make; popd
$ ./target/debug/santa ./example/example_c
Santa - a basic ELF packer
Read 14408 bytes
Allocated vec of len 14408
Compressing data...
Compressed to 1883 bytes
Encrypting data...
Writing output executable template...
Writing encrypted zip to /home/user/santa/out-enc.zip...
Compiling...
    [ cargo output ]
Cleanup...
Output: ./example/example_c.packed
$ strip example/example_c.packed
$ example/example_c
hello, world!
$ ./example_c.packed
hello, world!
```

## TODO
- loader: mutate encrypted buffer in-place, instead of allocating a new one
- general: do it without `cargo`?
	- use goblin to add a 'encrypted_zip' section to a base executable
	- target binary retrieves data from section and 
- general: error checking, bit of a tidy-up
- general: can we trim crate requirements to save space? ;o

