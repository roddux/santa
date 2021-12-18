//! Santa - a basic ELF packer
/*
### Process
#### V1 - memfd
- read ELF
- deflate/zip the data
- encrypt the data
- write new binary that will:
  - create an anonymous memory filedescriptor
  - unzip bundled data into said memory
  - decypt the data in-place
  - memfd_exec the memory file descriptor

#### V2 - inplace:
- read (position-independent) ELF
- parse LOAD addresses, etc
- lzdeflate/zip data
- write new binary that will:
  - unzip bundled data into memory
  - map LOAD segments to current address space
  - jump to start address
*/

use std::io::Write;
use rand::Rng;

/*
/// Parse the passed ELF
/// returns an object with lifetime 'b'
fn parse_elf<'b>(data:&'b Vec<u8>) -> goblin::elf::Elf<'b> {
    println!("parsing the ELF");
    let elf = goblin::Object::parse(&data);
    match elf.unwrap() {
        goblin::Object::Elf(elf) => { println!("got an ELF!"); return elf; },
        _ => { panic!("invalid file"); },
    }
}
*/

fn encrypt_and_return_key<'key>(buf: &mut [u8]) -> Vec<u8> {
    let mut enc_key = Vec::<u8>::new();
    let mut rng = rand::thread_rng();

    for _ in 0..64 {
        enc_key.push(rng.gen());
    }

    for n in 0..buf.len() {
//    if(n%4==0) { print!("\n"); }
//    print!("{:?} = {:?}\t", buf.get(n), buf.get(n).unwrap() ^enc_key.get(n%64).unwrap());
        buf[n] = buf.get(n).unwrap() ^ enc_key.get(n%64).unwrap();
    }

    enc_key
}

/// Primary entrypoint 
fn main() {
    println!("Santa - a basic ELF packer");

    // input filename is argv[1]
    let fname = &std::env::args().collect::<Vec<String>>()[1];

    // read the file
    let fdata = (std::fs::read(fname)).expect("Failed to read file");
    println!("Read {:?} bytes", fdata.len());

    // parse the ELF metadata (not used in this version)
    // let elf = parse_elf(&fdata);
    // println!("{:?}", elf);

    // allocate a buffer the size of the original file
    let mut buf = vec![0;fdata.len()];
    println!("Allocated vec of len {:?}", buf.len());

    // cursor on the file for zipwriter, and so we can track size
    let mut cur = std::io::Cursor::new( &mut buf[..] ); 
    let mut zdata = zip::ZipWriter::new( &mut cur );

    println!("Compressing data...");

    // CompressionMethod::Deflated seems to give smallest output in (brief) testing
    let options = zip::write::FileOptions::default().compression_method(
        zip::CompressionMethod::Deflated
    );

    // write the zip data to our buffer
    zdata.start_file("bin", options);
    zdata.write_all(&fdata);
    zdata.finish();
    drop(zdata); // we want to borrow cur again, so drop zdata here early

    // store the size of our zip data
    let sz = cur.position() as usize;
//    println!("Position: {:?}", sz);
    drop(cur); // we want buf now, so drop cur as we no longer need it

    println!("Compressed to {:?} bytes", sz);

    //let mut buf = &buf[0..sz]; // shadow old buf with a slice of the correct size

//    std::fs::write("./out.zip", &buf[0..sz]); // write to file

    println!("Encrypting data...");
    let key = encrypt_and_return_key(&mut buf[0..sz]);

//    println!("Key is: {:?}", key);


    // write to src/bin/loader.rs
let loader_format_str = r#"
#![feature(asm)]
static memFdName: &'static str = "\0";
static fdPath:    &'static str = "/proc/self/fd/3\0";
use std::os::unix::io::FromRawFd;
fn main() {
    let buf = include_bytes!("{FORMAT_ZIP}");
    let enc_key = {FORMAT_KEY};
    let mut out = Vec::<u8>::with_capacity(buf.len());
    for n in 0..buf.len() {
        out.push( buf.get(n).unwrap() ^ enc_key.get(n%64).unwrap() );
    }
    let mut zipcur = zip::ZipArchive::new(std::io::Cursor::new(&out[..]));
    let mut zipcur = zipcur.unwrap();
    let mut zipf = zipcur.by_name("bin").unwrap();
    unsafe {
        let mut asm_ret:u64;
        asm!(
            "lea   rdi, [{}]",
            "mov   rsi, 1",   // MFD_CLOEXEC
            "mov   rax, 319", // SYS_MEMFD_CREATE
            "syscall",
            "mov   rax, {}",
            sym memFdName,
            out(reg) asm_ret,
        );
        if asm_ret == 0 {
            panic!("failed");
        }
        let mut memfd = std::fs::File::from_raw_fd( asm_ret as i32 );
        std::io::copy(&mut zipf, &mut memfd);
        asm!(
            "mov   rdi, {}", // MFD_CLOEXEC
            "mov   rsi, 0",
            "mov   rdx, 0",
            "mov   r10, 0",
            "mov   r8, 0",
            "mov   r9, 0",
            "mov   rax, 59", // SYS_execve
            "syscall",
            sym fdPath,
        );
    }
}
"#;
    let mut zpath = String::from(std::env::current_dir().unwrap().to_str().unwrap());
    zpath.push_str("/out-enc.zip");

    let s = loader_format_str.replace("{FORMAT_ZIP}", &zpath);
    let s = s.replace("{FORMAT_KEY}", &format!("{:?}", key).to_string());
//    println!("Formatted src:\n{:}", s);
    println!("Writing output executable template...");

    std::fs::write("src/bin/loader.rs", s);

    println!("Writing encrypted zip to {}...", zpath);
    std::fs::write(zpath, &buf[0..sz]); // write to file

    println!("Compiling...");
    std::process::Command::new("cargo")
        .env("RUSTFLAGS", "-C relocation-model=dynamic-no-pic")
        .args(["build","--bin","loader"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // $ pushd /tmp
    // $ cargo new loader
    // overwrite src/main.rs
    // RUSTFLAGS="" cargo build
    // $ popd
    // $ cp /tmp/loader/target/release/loader .
    // $ strip loader
}
