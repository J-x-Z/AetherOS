#![no_std]
#![no_main]

use aether_user::{print, exit, entry_point, draw_pixel, SCREEN_WIDTH, SCREEN_HEIGHT};

entry_point!(main);

fn main() -> ! {
    print("Hello from AetherOS Graphics!\n");
    print("Drawing RGB Gradient...\n");
    
    // Draw Gradient
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let r = (x * 255 / SCREEN_WIDTH) as u32;
            let g = (y * 255 / SCREEN_HEIGHT) as u32;
            let b = 128;
            let color = (r << 16) | (g << 8) | b;
            draw_pixel(x, y, color);
        }
    }
    
    print("Done! Looping forever to keep window open.\n");
    loop {}
    // exit(0);
}
