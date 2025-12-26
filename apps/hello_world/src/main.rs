#![no_std]
#![no_main]

use aether_user::{print, console_init, console_println, set_colors, entry_point, console_getc, console_putc};

// Shell input buffer
const MAX_INPUT: usize = 256;

fn main() -> ! {
    // Initialize Console
    console_init();
    
    // Print Header
    set_colors(0, 255, 0, 0, 0, 64);  // Green text, Dark Blue bg
    console_println("           AETHER OS           ");
    console_println("===============================");
    
    set_colors(255, 255, 255, 0, 0, 0); // White on Black
    console_println("Welcome to AetherOS Shell!");
    console_println("Type 'help' for available commands.");
    console_println("");
    
    print("Guest: Shell initialized.\n");
    
    // Shell main loop
    let mut input_buffer: [u8; MAX_INPUT] = [0; MAX_INPUT];
    let mut input_len: usize = 0;
    let mut loop_ctr: u64 = 0;
    
    loop {
        loop_ctr += 1;
        if loop_ctr % 100000 == 0 {
             // print("."); 
        }

        // Print prompt if needed (simple logic to avoid spamming prompt)
        // Ideally we only print prompt once.
        // For now, let's just loop.
        
        // Read char (Non-blocking check? No, console_getc is polling but returns Option immediately if implementing it right)
        // Wait, our console_getc implementation:
        // if status == 1 { return Some } else { return None }
        // So it is NON-BLOCKING.
        
        if let Some(c) = console_getc() {
             // Handle Input
             match c {
                '\n' => {
                    console_putc('\n');
                    if input_len > 0 {
                        execute_command(&input_buffer[..input_len]);
                        input_len = 0;
                    }
                    // Reprint prompt
                    set_colors(0, 255, 255, 0, 0, 0);
                    console_putc('>');
                    console_putc(' ');
                    set_colors(255, 255, 255, 0, 0, 0);
                }
                '\x08' => {
                    if input_len > 0 {
                        input_len -= 1;
                        console_putc('\x08');
                    }
                }
                _ => {
                    if input_len < MAX_INPUT - 1 {
                        input_buffer[input_len] = c as u8;
                        input_len += 1;
                        console_putc(c);
                    }
                }
            }
        }
    }
}

fn execute_command(cmd: &[u8]) {
    if starts_with(cmd, b"help") {
        cmd_help();
    } else if starts_with(cmd, b"ls") {
        cmd_ls();
    } else if starts_with(cmd, b"cat ") {
        cmd_cat(&cmd[4..]);
    } else if starts_with(cmd, b"clear") {
        cmd_clear();
    } else if starts_with(cmd, b"info") {
        cmd_info();
    } else if starts_with(cmd, b"wasm ") {
        cmd_wasm(&cmd[5..]);
    } else {
        console_println("Unknown command.");
    }
}

fn cmd_ls() {
    if let Some(fs) = aether_user::fs::Ext2Driver::new() {
        fs.list_root();
    } else {
        console_println("Error: Failed to mount Ext2 filesystem.");
    }
}

// ... cat ... wasm ...

fn cmd_help() {
    console_println("Commands: help, ls, cat <file>, wasm <file>, clear, info");
}

fn cmd_clear() {
    console_init();
}

fn cmd_info() {
    console_println("AetherOS v0.3 / 8MB RAM / Ext2 FS");
}

fn starts_with(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() { return false; }
    for i in 0..needle.len() {
        if haystack[i] != needle[i] { return false; }
    }
    true
}

entry_point!(main);
