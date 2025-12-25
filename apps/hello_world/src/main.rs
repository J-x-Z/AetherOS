#![no_std]
#![no_main]

use aether_user::{print, fill_screen, draw_pixel, console_init, console_println, set_colors, entry_point, SCREEN_WIDTH};

fn main() -> ! {
    // Test 1: Print to console (HVC hypercall)
    print("Guest: Starting TTY Console Test\n");
    
    // Test 2: Fill screen with dark blue background
    fill_screen(0, 0, 64);
    
    // Test 3: Draw colored bars
    for x in 0..SCREEN_WIDTH {
        let color_index = x / 80;
        let (r, g, b) = match color_index {
            0 => (255, 0, 0),    // Red
            1 => (0, 255, 0),    // Green
            2 => (0, 0, 255),    // Blue
            3 => (255, 255, 0),  // Yellow
            4 => (255, 0, 255),  // Magenta
            5 => (0, 255, 255),  // Cyan
            6 => (255, 255, 255),// White
            _ => (128, 128, 128),// Gray
        };
        for y in 100..200 {
            draw_pixel(x, y, r, g, b);
        }
    }
    
    // Test 4: Initialize TTY console and print text
    console_init();
    
    set_colors(0, 255, 0, 0, 0, 64);  // Green on dark blue
    console_println("========================================");
    console_println("       Welcome to AetherOS v0.4.0       ");
    console_println("========================================");
    console_println("");
    
    set_colors(255, 255, 255, 0, 0, 64);  // White
    console_println("TTY Console is now working!");
    console_println("");
    console_println("This text is rendered directly to the");
    console_println("framebuffer using an 8x16 VGA font.");
    
    print("Guest: TTY Console Test Complete!\n");
    
    // Idle loop
    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}

entry_point!(main);
