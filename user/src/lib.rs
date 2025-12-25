#![no_std]

use core::arch::asm;
use aether_abi::HyperCall;

pub fn print(msg: &str) {
    let ptr = msg.as_ptr();
    let len = msg.len();
    
    unsafe {
        asm!(
            "mov x0, {0}",
            "mov x1, {1}",
            "mov x8, {2}",
            "hvc #0",
            in(reg) ptr,
            in(reg) len,
            const HyperCall::Print as u64,
            options(nostack)
        );
    }
}

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

pub const FBI_ADDR: usize = 0x100000;
pub const SCREEN_WIDTH: usize = 640;
pub const SCREEN_HEIGHT: usize = 480;

pub fn draw_pixel(x: usize, y: usize, color: u32) {
    if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT { return; }
    unsafe {
        let ptr = (FBI_ADDR + (y * SCREEN_WIDTH + x) * 4) as *mut u32;
        *ptr = color;
    }
}

pub fn fill_screen(color: u32) {
    unsafe {
        let ptr = FBI_ADDR as *mut u32;
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            *ptr.add(i) = color;
        }
    }
}

pub fn flush() {
    // Optional: Call HVC to notify host (if we weren't polling)
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
