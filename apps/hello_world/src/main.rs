#![no_std]
#![no_main]

use aether_user::{console_init, console_println, console_print, set_colors, entry_point};

fn main() -> ! {
    // Initialize the console (clears screen)
    console_init();
    
    // Print welcome message
    console_println("========================================");
    console_println("       Welcome to AetherOS v0.4.0       ");
    console_println("========================================");
    console_println("");
    console_println("TTY Console initialized successfully!");
    console_println("");
    
    // Demo: Change colors
    set_colors(0, 255, 0, 0, 0, 0);  // Green text on black
    console_println("[OK] Graphics subsystem ready");
    
    set_colors(255, 255, 0, 0, 0, 0);  // Yellow text
    console_println("[INFO] Running on AetherOS Microkernel");
    
    set_colors(0, 255, 255, 0, 0, 0);  // Cyan text
    console_println("[INFO] Platform: Universal (8 OS backends)");
    
    set_colors(255, 255, 255, 0, 0, 0);  // White text
    console_println("");
    console_println("This text is rendered directly to the");
    console_println("framebuffer using an 8x16 VGA font.");
    console_println("");
    console_println("Supported features:");
    console_println("  - 80x30 character display");
    console_println("  - Automatic line wrapping");
    console_println("  - Screen scrolling");
    console_println("  - Foreground/background colors");
    console_println("");
    
    set_colors(128, 128, 128, 0, 0, 0);  // Gray text
    console_print("Guest is now idle. Press ESC to exit.");
    
    // Idle loop
    loop {
        // In future: poll for input events
        unsafe { core::arch::asm!("wfe"); }
    }
}

entry_point!(main);
