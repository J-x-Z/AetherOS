#![no_std]

use core::arch::asm;
use aether_abi::HyperCall;

// Font and Console modules
pub mod font;
pub mod console;

// Re-export console functions for convenience
pub use console::{init as console_init, print as console_print, println as console_println, clear as console_clear, set_colors};

// Framebuffer constants
pub const FB_ADDR: usize = 0x100000;
pub const SCREEN_WIDTH: usize = 640;
pub const SCREEN_HEIGHT: usize = 480;

/// Print via hypercall (for kernel-level debugging)
pub fn print(msg: &str) {
    let ptr = msg.as_ptr() as u64;
    let len = msg.len() as u64;
    
    unsafe {
        asm!(
            "hvc #0",
            in("x0") ptr,
            in("x1") len,
            in("x8") HyperCall::Print as u64,
            options(nostack, nomem)
        );
    }
}

/// Exit the guest
pub fn exit(code: u64) -> ! {
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
}

/// Draw a single pixel (low-level)
pub fn draw_pixel(x: usize, y: usize, r: u8, g: u8, b: u8) {
    if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT { return; }
    let color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    unsafe {
        let ptr = (FB_ADDR + (y * SCREEN_WIDTH + x) * 4) as *mut u32;
        ptr.write_volatile(color);
    }
}

/// Fill the entire screen with a color
pub fn fill_screen(r: u8, g: u8, b: u8) {
    let color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    unsafe {
        let ptr = FB_ADDR as *mut u32;
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            ptr.add(i).write_volatile(color);
        }
    }
}

#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[no_mangle]
        pub extern "C" fn _start() -> ! {
            let f: fn() -> ! = $path;
            f()
        }
        
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }
    }
}
