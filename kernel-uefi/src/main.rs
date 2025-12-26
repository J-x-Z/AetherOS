#![no_std]
#![no_main]

extern crate alloc;

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::cstr16;
use uefi::proto::media::file::File; 
use core::result::Result::{Ok, Err};

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    
    // Reset console
    system_table.stdout().reset(false).unwrap();
    
    log::info!("AetherOS Hybrid Kernel (UEFI Mode)");

    // Get GOP (Graphics)
    let bt = system_table.boot_services();
    let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>()
        .expect("GOP not found");
    let mut gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .expect("Failed to open GOP");
    
    // Query Mode
    let mode_info = gop.current_mode_info();
    let (width, height) = mode_info.resolution();
    log::info!("Resolution: {}x{}", width, height);

    // Get Framebuffer
    let mut fb = gop.frame_buffer();
    let fb_ptr = fb.as_mut_ptr();
    let size = fb.size();
    
    log::info!("Framebuffer: {:p}, Size: {} bytes", fb_ptr, size);
    
    // Clear Screen loop
    unsafe {
        let ptr = fb_ptr as *mut u32;
        let pixels = size / 4;
        for i in 0..pixels {
            *ptr.add(i) = 0x00000080; 
        }
    }
    
    log::info!("Screen Cleared.");

    // Load Guest Kernel
    // 1. Get LoadedImage to find out which device we booted from
    let loaded_image = bt.open_protocol_exclusive::<uefi::proto::loaded_image::LoadedImage>(image_handle)
        .expect("Failed to open LoadedImage");
    let device_handle = loaded_image.device().expect("Device handle missing");
    
    // 2. Open FileSystem on that device
    let mut sfs = bt.open_protocol_exclusive::<uefi::proto::media::fs::SimpleFileSystem>(device_handle)
        .expect("Failed to open SimpleFileSystem");
    let mut root = sfs.open_volume().expect("Failed to open volume");
    
    // 3. Open guest file
    let filename = cstr16!("guest-x86_64.bin");
    let file_handle = root.open(filename, uefi::proto::media::file::FileMode::Read, uefi::proto::media::file::FileAttribute::empty());
    
    match file_handle {
        Ok(mut file) => {
             let mut file = file.into_regular_file().expect("Not a regular file");
             log::info!("Found guest kernel: guest-x86_64.bin");
             
             // Get file size
             file.set_position(0xFFFFFFFFFFFFFFFF).unwrap(); // Seek to end
             let size = file.get_position().unwrap();
             file.set_position(0).unwrap(); // Rewind
             
             log::info!("Guest Size: {} bytes", size);
             
             // Allocate memory for guest
             // For now just allocate a buffer (Pool). Real kernel needs Page allocation at fixed address.
             let mut buffer = alloc::vec![0u8; size as usize];
             let read_size = file.read(&mut buffer).unwrap();
             
             log::info!("Read {} bytes into memory at {:p}", read_size, buffer.as_ptr());
             
             // Verification: Print first bytes
             if read_size > 4 {
                 log::info!("Header: {:02x} {:02x} {:02x} {:02x}", buffer[0], buffer[1], buffer[2], buffer[3]);
             }
        },
        Err(e) => {
            log::error!("Failed to open guest-x86_64.bin: {:?}", e);
            log::warn!("Ensure guest-x86_64.bin is in the root of the EFI partition.");
        }
    }

    log::info!("System halted. Stalling...");
    bt.stall(10_000_000);
    
    Status::SUCCESS
}
