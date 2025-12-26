use crate::DISK_ADDR;

const BLOCK_SIZE: usize = 1024;

// --- Headers ---
#[repr(C)]
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
    // ...
}

#[repr(C)]
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

#[repr(C)]
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

#[repr(C, packed)]
struct DirEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
    // name follows
}

pub struct Ext2Driver {
    base_addr: usize,
    inode_table_block: u32,
}

impl Ext2Driver {
    pub fn new() -> Option<Self> {
        unsafe {
            let sb = &*((DISK_ADDR + 1024) as *const Superblock);
            if sb.magic != 0xEF53 {
                crate::console_println("Ext2: Invalid Magic (Not 0xEF53)");
                return None;
            }
            
            // Assume Block Group 0
            // Group Descriptor at 2048 (Block 2)
            let gd = &*((DISK_ADDR + 2048) as *const GroupDescriptor);
            
            crate::console_println("Ext2: Mounted Successfully.");
            
            Some(Ext2Driver {
                base_addr: DISK_ADDR,
                inode_table_block: gd.inode_table,
            })
        }
    }
    
    fn get_inode(&self, index: u32) -> Option<&Inode> {
        // Simple implementation: only support BG0 inodes (< inodes_per_group)
        // Inode index starts at 1
        if index < 1 { return None; }
        
        let table_addr = self.base_addr + (self.inode_table_block as usize * BLOCK_SIZE);
        let inode_size = core::mem::size_of::<Inode>(); // 128
        let offset = (index - 1) as usize * inode_size;
        
        unsafe {
            Some(&*((table_addr + offset) as *const Inode))
        }
    }
    
    pub fn list_root(&self) {
        crate::console_println("--- Listing / ---");
        let root = self.get_inode(2).expect("No Root Inode");
        
        // Read Block 0 of Root Inode
        let block_idx = root.block[0];
        let block_addr = self.base_addr + (block_idx as usize * BLOCK_SIZE);
        
        let mut offset = 0;
        let mut count = 0;
        
        unsafe {
            while offset < BLOCK_SIZE {
                let ent = &*((block_addr + offset) as *const DirEntry);
                if ent.inode == 0 { break; } // End
                
                let name_ptr = (block_addr + offset + 8) as *const u8;
                let name_len = ent.name_len as usize;
                
                // Print name char by char
                for i in 0..name_len {
                    crate::console_putc(*name_ptr.add(i) as char);
                }
                
                if ent.file_type == 2 {
                    crate::console_putc('/');
                }
                crate::console_putc('\n');
                
                count += 1;
                offset += ent.rec_len as usize;
                if ent.rec_len == 0 { break; } // Safety
            }
        }
        if count == 0 {
             crate::console_println("(Empty or Error)");
        }
        crate::console_println("-----------------");
    }
    
    pub fn read_file(&self, filename: &str) -> Option<alloc::vec::Vec<u8>> {
        // 1. Scan Root Dir for filename
        let root = self.get_inode(2).expect("No Root Inode");
        let block_idx = root.block[0];
        let block_addr = self.base_addr + (block_idx as usize * BLOCK_SIZE);
        
        let mut target_inode_idx = 0;
        let mut offset = 0;
        
        unsafe {
            while offset < BLOCK_SIZE {
                let ent = &*((block_addr + offset) as *const DirEntry);
                if ent.inode == 0 { break; }
                
                let name_ptr = (block_addr + offset + 8) as *const u8;
                let name_slice = core::slice::from_raw_parts(name_ptr, ent.name_len as usize);
                
                // Compare name
                let mut match_name = true;
                if name_slice.len() != filename.len() {
                    match_name = false;
                } else {
                    for i in 0..filename.len() {
                        if name_slice[i] != filename.as_bytes()[i] {
                            match_name = false;
                            break;
                        }
                    }
                }
                
                if match_name {
                    target_inode_idx = ent.inode;
                    break;
                }
                
                offset += ent.rec_len as usize;
                if ent.rec_len == 0 { break; }
            }
        }
        
        if target_inode_idx == 0 {
            return None;
        }
        
        // 2. Read File Data
        let inode = self.get_inode(target_inode_idx).unwrap();
        let file_size = inode.size as usize;
        let mut buffer = alloc::vec::Vec::with_capacity(file_size);
        
        // Read blocks
        let num_blocks = (file_size + BLOCK_SIZE - 1) / BLOCK_SIZE;
        
        for i in 0..num_blocks {
            // Support only direct blocks (0-11)
            if i >= 12 {
                crate::console_println("Error: File too large (>12 blocks)");
                break;
            }
            
            let block_idx = inode.block[i];
            if block_idx == 0 { break; } // Standard says sparse files are 0, but here implies end
            
            let block_ptr = (self.base_addr + (block_idx as usize * BLOCK_SIZE)) as *const u8;
            
            let bytes_to_read = if i == num_blocks - 1 {
                file_size % BLOCK_SIZE
            } else {
                BLOCK_SIZE
            };
            let bytes_to_read = if bytes_to_read == 0 { BLOCK_SIZE } else { bytes_to_read };
            
            unsafe {
                for j in 0..bytes_to_read {
                    buffer.push(*block_ptr.add(j));
                }
            }
        }
        
        Some(buffer)
    }

    pub fn cat(&self, filename: &str) {
        if let Some(data) = self.read_file(filename) {
             crate::console_println("--- File Content ---");
             for b in data {
                 crate::console_putc(b as char);
             }
             crate::console_putc('\n');
             crate::console_println("--------------------");
        } else {
            crate::console_println("File not found.");
        }
    }
}
