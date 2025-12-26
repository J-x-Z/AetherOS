use std::fs::File;
use std::io::{Write, Seek, SeekFrom};

const BLOCK_SIZE: usize = 1024;
const BLOCK_COUNT: u32 = 1024; // 1MB disk
const INODES_COUNT: u32 = 32;

// Utility to convert struct to bytes
unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        std::mem::size_of::<T>(),
    )
}

#[repr(C)]
#[derive(Debug)]
struct Superblock {
    inodes_count: u32,
    blocks_count: u32,
    r_blocks_count: u32,
    free_blocks_count: u32,
    free_inodes_count: u32,
    first_data_block: u32,
    log_block_size: u32,
    log_frag_size: u32,
    blocks_per_group: u32,
    frags_per_group: u32,
    inodes_per_group: u32,
    mtime: u32,
    wtime: u32,
    mnt_count: u16,
    max_mnt_count: u16,
    magic: u16,
    state: u16,
    errors: u16,
    minor_rev_level: u16,
    lastcheck: u32,
    checkinterval: u32,
    creator_os: u32,
    rev_level: u32,
    def_resuid: u16,
    def_resgid: u16,
    // ... padding
    padding: [u8; 940],
}

impl Default for Superblock {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug)]
struct GroupDescriptor {
    block_bitmap: u32,
    inode_bitmap: u32,
    inode_table: u32,
    free_blocks_count: u16,
    free_inodes_count: u16,
    used_dirs_count: u16,
    pad: u16,
    reserved: [u32; 3],
}

impl Default for GroupDescriptor {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug)]
struct Inode {
    mode: u16,
    uid: u16,
    size: u32,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    gid: u16,
    links_count: u16,
    blocks: u32,
    flags: u32,
    osd1: u32,
    block: [u32; 15],
    generation: u32,
    file_acl: u32,
    dir_acl: u32,
    faddr: u32,
    osd2: [u8; 12],
}

impl Default for Inode {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C, packed)]
#[derive(Debug)]
struct DirEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
    // Name is variable length, but we don't map it here directly
}

fn main() -> std::io::Result<()> {
    let mut file = File::create("disk.img")?;
    file.set_len((BLOCK_SIZE as u64) * (BLOCK_COUNT as u64))?;

    // 1. Write Superblock (Block 1)
    let mut s_block = Superblock::default();
    s_block.inodes_count = INODES_COUNT;
    s_block.blocks_count = BLOCK_COUNT;
    s_block.r_blocks_count = 0;
    s_block.free_blocks_count = BLOCK_COUNT - 10;
    s_block.free_inodes_count = INODES_COUNT - 2;
    s_block.first_data_block = 1;
    s_block.log_block_size = 0;
    s_block.log_frag_size = 0;
    s_block.blocks_per_group = 8192;
    s_block.frags_per_group = 8192;
    s_block.inodes_per_group = INODES_COUNT;
    s_block.magic = 0xEF53;
    
    file.seek(SeekFrom::Start(1024))?;
    unsafe { file.write_all(as_u8_slice(&s_block))?; }

    // 2. Write Group Descriptor (Block 2)
    let mut gd = GroupDescriptor::default();
    gd.block_bitmap = 3;
    gd.inode_bitmap = 4;
    gd.inode_table = 5;
    gd.free_blocks_count = (BLOCK_COUNT - 9) as u16;
    gd.free_inodes_count = (INODES_COUNT - 2) as u16;
    gd.used_dirs_count = 1;

    file.seek(SeekFrom::Start(2048))?;
    unsafe { file.write_all(as_u8_slice(&gd))?; }

    // 3. Write Inode Table (Block 5)
    // Inode 2: Root Directory
    let mut root_inode = Inode::default();
    root_inode.mode = 0x41ED; // Directory + rwxr-xr-x
    root_inode.size = 1024;
    root_inode.links_count = 2; // . and ..
    root_inode.blocks = 2;
    root_inode.block[0] = 7; // Root Directory Data at Block 7
    
    // Inode 11: hello.txt
    let mut hello_inode = Inode::default();
    hello_inode.mode = 0x81A4; // File + rw-r--r--
    hello_inode.size = 13;     // "Hello, World!"
    hello_inode.links_count = 1;
    hello_inode.blocks = 2;
    hello_inode.block[0] = 8; // File Data at Block 8

    let inode_size = std::mem::size_of::<Inode>();
    // Offset for Inode 2 (Index 1)
    file.seek(SeekFrom::Start(5 * 1024 + 1 * inode_size as u64))?;
    unsafe { file.write_all(as_u8_slice(&root_inode))?; }

    // Offset for Inode 11 (Index 10)
    file.seek(SeekFrom::Start(5 * 1024 + 10 * inode_size as u64))?;
    unsafe { file.write_all(as_u8_slice(&hello_inode))?; }

    // 4. Write Root Directory Data (Block 7)
    let mut dir_data = vec![0u8; 1024];
    let mut w = std::io::Cursor::new(&mut dir_data);
    
    // .
    w.write_all(&2u32.to_le_bytes())?; // inode
    w.write_all(&12u16.to_le_bytes())?; // rec_len
    w.write_all(&1u8.to_le_bytes())?; // name_len
    w.write_all(&2u8.to_le_bytes())?; // type
    w.write_all(b".\0\0\0")?; // name + pad

    // ..
    w.write_all(&2u32.to_le_bytes())?; // inode
    w.write_all(&12u16.to_le_bytes())?; // rec_len
    w.write_all(&2u8.to_le_bytes())?; // name_len
    w.write_all(&2u8.to_le_bytes())?; // type
    w.write_all(b"..\0\0")?; // name + pad

    // hello.txt
    let name = b"hello.txt";
    w.write_all(&11u32.to_le_bytes())?; // inode
    w.write_all(&(1000u16).to_le_bytes())?; // rec_len (rest of block)
    w.write_all(&(name.len() as u8).to_le_bytes())?; // name_len
    w.write_all(&1u8.to_le_bytes())?; // type
    w.write_all(name)?; // name
    
    file.seek(SeekFrom::Start(7 * 1024))?;
    file.write_all(&dir_data)?;

    // ... (Previous write of hello.txt at Inode 11 calls) ...

    // 5. Write File Data (Block 8)
    file.seek(SeekFrom::Start(8 * 1024))?;
    file.write_all(b"Hello, World!")?;

    // --- Dynamic File Injection (test.wasm) ---
    // Usage: mkext2 <path_to_wasm>
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let wasm_path = &args[1];
        let wasm_data = std::fs::read(wasm_path).expect("Failed to read input file");
        let file_size = wasm_data.len() as u32;
        
        let num_blocks = (file_size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32;
        let start_block = 9; // Next free block after hello.txt (Block 8)
        
        if start_block + num_blocks > BLOCK_COUNT {
             panic!("File too large for disk!");
        }

        // Write Inode 12
        let mut wasm_inode = Inode::default();
        wasm_inode::mode = 0x81A4;
        wasm_inode::size = file_size;
        wasm_inode::links_count = 1;
        wasm_inode::blocks = num_blocks * 2; // Sectors? usually blocks * (block_size/512). Ext2 `blocks` field is in 512-byte sectors!
        // Wait, standard linux ext2 `i_blocks` is 512-byte sectors.
        // My previous code: `root_inode.blocks = 2`. root dir is 1024 bytes (1 block). 1 block = 2 sectors. Correct.
        // So `blocks` = `num_blocks * 2`.
        
        for i in 0..num_blocks {
             if i < 12 { // Only direct blocks support in this simple tool
                 wasm_inode::block[i as usize] = start_block + i;
             }
        }
        
        // Write Inode 12 (Index 11)
        file.seek(SeekFrom::Start(5 * 1024 + 11 * inode_size as u64))?;
        unsafe { file.write_all(as_u8_slice(&wasm_inode))?; }
        
        // Add Directory Entry
        // We append to Root Dir Data (Block 7).
        // Previous contents: ., .., hello.txt
        
        // Re-read Block 7 (or just simulate append since we know offset)
        // . (12 bytes), .. (12 bytes), hello.txt (rec_len=1000).
        // Effectively `hello.txt` claimed the rest of the block.
        // We need to split the last entry or just rewrite the dir block.
        
        // Rewrite Block 7
        let mut dir_data = vec![0u8; 1024];
        let mut w = std::io::Cursor::new(&mut dir_data);
        
        // .
        w.write_all(&2u32.to_le_bytes())?;
        w.write_all(&12u16.to_le_bytes())?;
        w.write_all(&1u8.to_le_bytes())?;
        w.write_all(&2u8.to_le_bytes())?;
        w.write_all(b".\0\0\0")?;
        
        // ..
        w.write_all(&2u32.to_le_bytes())?;
        w.write_all(&12u16.to_le_bytes())?;
        w.write_all(&2u8.to_le_bytes())?;
        w.write_all(&2u8.to_le_bytes())?;
        w.write_all(b"..\0\0")?;
        
        // hello.txt
        let name_hello = b"hello.txt";
        let hello_len = 8 + name_hello.len() + 3; // 20 bytes -> align 4 -> 20.
        // header(8) + name(9) + pad(3) = 20.
        // previous header was 8? 
        // inode(4) + rec_len(2) + name_len(1) + type(1) = 8.
        let hello_rec_len = 20u16; 
        
        w.write_all(&11u32.to_le_bytes())?;
        w.write_all(&hello_rec_len.to_le_bytes())?;
        w.write_all(&(name_hello.len() as u8).to_le_bytes())?;
        w.write_all(&1u8.to_le_bytes())?;
        w.write_all(name_hello)?; 
        w.write_all(b"\0\0\0")?; // Pad to 4 bytes boundary
        
        // test.wasm
        let name_wasm = b"test.wasm";
        let wasm_rec_len = (1024 - 12 - 12 - 20) as u16; // Rest of block
        
        w.write_all(&12u32.to_le_bytes())?; // Inode 12
        w.write_all(&wasm_rec_len.to_le_bytes())?;
        w.write_all(&(name_wasm.len() as u8).to_le_bytes())?;
        w.write_all(&1u8.to_le_bytes())?;
        w.write_all(name_wasm)?;
        
        // Write dir block
        file.seek(SeekFrom::Start(7 * 1024))?;
        file.write_all(&dir_data)?;
        
        // Write Data Blocks
        file.seek(SeekFrom::Start((start_block as u64) * 1024))?;
        file.write_all(&wasm_data)?;
        
        println!("Injected {} as test.wasm ({} bytes)", wasm_path, file_size);
    }

    println!("Created disk.img (1MB EXT2)");
    Ok(())
}
