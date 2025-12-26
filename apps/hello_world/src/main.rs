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

fn cmd_cat(filename: &[u8]) {
    // Convert bytes to string for lookup
    let name = core::str::from_utf8(filename).unwrap_or("?");
    if let Some(fs) = aether_user::fs::Ext2Driver::new() {
        if let Some(data) = fs.read_file(name) {
            // Print file contents as string
            if let Ok(s) = core::str::from_utf8(&data) {
                console_println(s);
            } else {
                console_println("[Binary data]");
            }
        } else {
            console_println("File not found.");
        }
    } else {
        console_println("Error: FS not mounted.");
    }
}

fn cmd_wasm(filename: &[u8]) {
    let name = core::str::from_utf8(filename).unwrap_or("?");
    console_println("Loading WASM...");
    if let Some(fs) = aether_user::fs::Ext2Driver::new() {
        if let Some(wasm_bytes) = fs.read_file(name) {
            run_wasm(&wasm_bytes);
        } else {
            console_println("WASM file not found.");
        }
    } else {
        console_println("Error: FS not mounted.");
    }
}

fn run_wasm(bytes: &[u8]) {
    use wasmi::{Engine, Linker, Module, Store, Caller, Func};
    
    let engine = Engine::default();
    let module = match Module::new(&engine, bytes) {
        Ok(m) => m,
        Err(e) => {
            console_println("WASM parse error");
            return;
        }
    };
    
    // Create linker with host function
    let mut linker = <Linker<()>>::new(&engine);
    
    // Define host function: env.print
    linker.define("env", "print", Func::wrap(&engine, |_caller: Caller<'_, ()>, val: i32| {
        // Simple print implementation
        let mut buf = [0u8; 16];
        let s = format_i32(val, &mut buf);
        console_println(s);
    })).unwrap();
    
    let mut store = Store::new(&engine, ());
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(i) => match i.start(&mut store) {
            Ok(i) => i,
            Err(_) => { console_println("WASM start failed"); return; }
        },
        Err(_) => { console_println("WASM instantiate failed"); return; }
    };
    
    // Call run() if it exists
    if let Some(run) = instance.get_func(&store, "run") {
        let _ = run.call(&mut store, &[], &mut []);
    }
    console_println("WASM execution complete.");
}

fn format_i32(val: i32, buf: &mut [u8; 16]) -> &str {
    // Simple i32 to string
    let mut v = val;
    let mut i = buf.len() - 1;
    let negative = v < 0;
    if negative { v = -v; }
    loop {
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 { break; }
        if i == 0 { break; }
        i -= 1;
    }
    if negative && i > 0 { i -= 1; buf[i] = b'-'; }
    core::str::from_utf8(&buf[i..]).unwrap_or("?")
}

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
