#![no_std]

use core::arch::asm;
use aether_abi::HyperCall;

// Font and Console modules
pub mod font;
pub mod console;

// Re-export console functions for convenience
pub use console::{init as console_init, println as console_println, set_colors, console_getc, console_putc};

// Framebuffer constants
// Re-export constants from ABI
pub use aether_abi::mmio::{FB_ADDR, KEYBOARD_STATUS, KEYBOARD_DATA, DISK_ADDR};

pub mod fs;

pub const SCREEN_WIDTH: usize = 640;
pub const SCREEN_HEIGHT: usize = 480;

/// Print via hypercall (for kernel-level debugging)
pub fn print(msg: &str) {
    let ptr = msg.as_ptr() as u64;
    let len = msg.len() as u64;
    
    #[cfg(target_arch = "aarch64")]
    unsafe {
        asm!(
            "hvc #0",
            in("x0") ptr,
            in("x1") len,
            in("x8") HyperCall::Print as u64,
            options(nostack, nomem)
        );
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!(
            "out dx, al",
            in("dx") 0x500u16,
            in("al") HyperCall::Print as u8,
            in("rdi") ptr,
            in("rsi") len,
            options(nostack, nomem)
        );
    }
}

/// Exit the guest
pub fn exit(code: u64) -> ! {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        asm!(
            "mov x0, {0}",
            "mov x8, {1}",
            "hvc #0",
            in(reg) code,
            const HyperCall::Exit as u64,
            options(noreturn)
        );
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!(
            "out dx, al",
            in("dx") 0x500u16,
            in("al") HyperCall::Exit as u8,
            in("rdi") code,
            options(noreturn)
        );
    }
}

/// Helper to get dynamic FB address
pub fn get_fb_addr() -> usize {
    unsafe { BASE_ADDRESS + (FB_ADDR as usize) }
}

pub fn get_keyboard_status_addr() -> usize {
    unsafe { BASE_ADDRESS + (KEYBOARD_STATUS as usize) }
}

pub fn get_keyboard_data_addr() -> usize {
    unsafe { BASE_ADDRESS + (KEYBOARD_DATA as usize) }
}


/// Draw a single pixel (low-level)
pub fn draw_pixel(x: usize, y: usize, r: u8, g: u8, b: u8) {
    if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT { return; }
    let color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    unsafe {
        let ptr = (get_fb_addr() + (y * SCREEN_WIDTH + x) * 4) as *mut u32;
        ptr.write_volatile(color);
    }
}

/// Fill the entire screen with a color
pub fn fill_screen(r: u8, g: u8, b: u8) {
    let color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    unsafe {
        let ptr = get_fb_addr() as *mut u32;
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            ptr.add(i).write_volatile(color);
        }
    }
}

// --- Heap Allocation ---
use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

extern "C" {
    static _heap_start: usize;
}

pub fn init_heap() {
    unsafe {
        let heap_start = &_heap_start as *const usize as usize;
        // Adjusted for UEFI Relocatable Execution:
        // Use fixed 4MB heap instead of hardcoded 0x7FF000 limit.
        // This ensures we stay within the allocated RAM block (assuming it's > 4MB + code).
        let heap_size = 4 * 1024 * 1024; 
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}

// Enable `extern crate alloc;`
extern crate alloc;

pub static mut BASE_ADDRESS: usize = 0;

#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[no_mangle]
        pub extern "C" fn _start(base_addr: usize) -> ! {
            unsafe { $crate::BASE_ADDRESS = base_addr; }
            
            // Init Heap
            $crate::init_heap();
            
            let f: fn() -> ! = $path;
            f()
        }
        
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    }
}

