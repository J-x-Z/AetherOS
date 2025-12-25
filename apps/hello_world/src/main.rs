#![no_std]
#![no_main]

use aether_user::{print, fill_screen, draw_pixel, console_init, console_println, set_colors, entry_point, SCREEN_WIDTH};

fn main() -> ! {
    print("Guest: Starting Main...\n");
    // 1. Initialize Console (Clears screen to black)
    console_init();
    
    // 2. Print Header
    set_colors(0, 255, 0, 0, 0, 64);  // Green text, Blue bg
    console_println("========================================");
    console_println("       Welcome to AetherOS v0.4.0       ");
    console_println("========================================");
    console_println("");
    
    // 3. Print Status
    set_colors(255, 255, 255, 0, 0, 0); // White on Black
    console_println("TTY Console is active.");
    console_println("Graphics Mode: 640x480");
    console_println("");
    
    print("Guest: Text printed.\n");
    
    // 4. Draw Colored Bars (Visual confirmation)
    // Draw at the bottom half of the screen
    let bar_height = 100;
    let start_y = 300;
    
    for x in 0..SCREEN_WIDTH {
        let color_index = x / (SCREEN_WIDTH / 8);
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
        for y in start_y..(start_y + bar_height) {
            draw_pixel(x, y, r, g, b);
        }
    }
    
    print("Guest: Bars drawn. Entering idle loop.\n");

    // Idle loop
    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}

entry_point!(main);
