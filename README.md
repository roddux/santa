# Santa - an ELF loader

I realised that you can call `execfd` on a `memfd`. Of course, it figures that
someone else created a cool tool to do this, too. I took most inspiration from
[this blog post](https://www.guitmz.com/running-elf-from-memory/). I added
compression and some xor encryption to the mix.

TL;DR: `santa` will use `cargo` to compile a given executable into another. The
new executable will contain the existing executable, but zipped and `xor`'d
with a random key. On running the new executable, it will decrypt and
decompress the original to a `memfd` and then execute it with `execfd`.

This was a fun project to learn some more about Rust. I may yet extend this
tool to actually directly load the ELF into memory, rather than the `memfd` ->
`execfd` route. _Could_ also extend it to incorporate a template binary that
uses predefined sections, to remove the reliance on `cargo`. Then we'd just
use goblin to insert our new blob and key into the template binary.

This repo also contains some binaries to test the tool.

## TODO
- santa: use the loader_template.rs file instead of static string
- general: figure out `cargo` flags to avoid including loader_template.rs
- loader: mutate encrypted buffer in-place, instead of allocating a new one
- general: do it without `cargo`?
	- use goblin to add a 'encrypted_zip' section to a base executable
	- target binary retrieves data from section and 
