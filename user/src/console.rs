/// TTY Console - Text rendering to framebuffer
use crate::font::{get_char_bitmap, FONT_WIDTH, FONT_HEIGHT};
use crate::{draw_pixel, SCREEN_WIDTH, SCREEN_HEIGHT, FB_ADDR};

/// Console dimensions (in characters)
pub const CONSOLE_COLS: usize = SCREEN_WIDTH / FONT_WIDTH;   // 80 columns
pub const CONSOLE_ROWS: usize = SCREEN_HEIGHT / FONT_HEIGHT; // 30 rows

/// Default colors
pub const DEFAULT_FG: u32 = 0x00FFFFFF; // White
pub const DEFAULT_BG: u32 = 0x00000000; // Black

/// Console state
pub struct Console {
    cursor_x: usize,
    cursor_y: usize,
    fg_color: u32,
    bg_color: u32,
}

impl Console {
    /// Create a new console with default settings
    pub const fn new() -> Self {
        Console {
            cursor_x: 0,
            cursor_y: 0,
            fg_color: DEFAULT_FG,
            bg_color: DEFAULT_BG,
        }
    }

    /// Clear the entire screen
    pub fn clear(&mut self) {
        let fb = FB_ADDR as *mut u32;
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            unsafe {
                fb.add(i).write_volatile(self.bg_color);
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    // ... (omitted)

/// Initialize the console (clear screen)
pub fn init() {
    unsafe {
        CONSOLE.clear();
    }
}

    /// Set foreground color (RGB)
    pub fn set_fg(&mut self, r: u8, g: u8, b: u8) {
        self.fg_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    }

    /// Set background color (RGB)
    pub fn set_bg(&mut self, r: u8, g: u8, b: u8) {
        self.bg_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    }

    /// Draw a single character at (char_x, char_y) position
    fn draw_char(&self, char_x: usize, char_y: usize, c: u8) {
        let bitmap = get_char_bitmap(c);
        let base_x = char_x * FONT_WIDTH;
        let base_y = char_y * FONT_HEIGHT;
        let fb = FB_ADDR as *mut u32;

        for row in 0..FONT_HEIGHT {
            let row_data = bitmap[row];
            for col in 0..FONT_WIDTH {
                let pixel_set = (row_data >> (7 - col)) & 1 != 0;
                let color = if pixel_set { self.fg_color } else { self.bg_color };
                let x = base_x + col;
                let y = base_y + row;
                if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
                    unsafe {
                        fb.add(y * SCREEN_WIDTH + x).write_volatile(color);
                    }
                }
            }
        }
    }

    /// Scroll the screen up by one line
    fn scroll(&mut self) {
        let fb = FB_ADDR as *mut u32;
        let line_pixels = FONT_HEIGHT * SCREEN_WIDTH;
        let total_pixels = SCREEN_WIDTH * SCREEN_HEIGHT;

        // Move all lines up by one
        unsafe {
            core::ptr::copy(
                fb.add(line_pixels),
                fb,
                total_pixels - line_pixels,
            );
        }

        // Clear the last line
        let last_line_start = total_pixels - line_pixels;
        for i in 0..line_pixels {
            unsafe {
                fb.add(last_line_start + i).write_volatile(self.bg_color);
            }
        }
    }

    /// Handle newline
    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= CONSOLE_ROWS {
            self.scroll();
            self.cursor_y = CONSOLE_ROWS - 1;
        }
    }

    /// Handle backspace - delete previous character
    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            // Clear the character by drawing a space
            self.draw_char(self.cursor_x, self.cursor_y, b' ');
        } else if self.cursor_y > 0 {
            // Move to end of previous line
            self.cursor_y -= 1;
            self.cursor_x = CONSOLE_COLS - 1;
            self.draw_char(self.cursor_x, self.cursor_y, b' ');
        }
    }

    /// Draw cursor at current position (block cursor)
    pub fn draw_cursor(&self) {
        let base_x = self.cursor_x * FONT_WIDTH;
        let base_y = self.cursor_y * FONT_HEIGHT;
        let fb = FB_ADDR as *mut u32;
        
        // Draw a filled rectangle as cursor
        for row in 0..FONT_HEIGHT {
            for col in 0..FONT_WIDTH {
                let x = base_x + col;
                let y = base_y + row;
                if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
                    unsafe {
                        fb.add(y * SCREEN_WIDTH + x).write_volatile(self.fg_color);
                    }
                }
            }
        }
    }

    /// Hide cursor (redraw background)
    pub fn hide_cursor(&self) {
        let base_x = self.cursor_x * FONT_WIDTH;
        let base_y = self.cursor_y * FONT_HEIGHT;
        let fb = FB_ADDR as *mut u32;
        
        for row in 0..FONT_HEIGHT {
            for col in 0..FONT_WIDTH {
                let x = base_x + col;
                let y = base_y + row;
                if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
                    unsafe {
                        fb.add(y * SCREEN_WIDTH + x).write_volatile(self.bg_color);
                    }
                }
            }
        }
    }

    /// Print a single character
    pub fn putc(&mut self, c: u8) {
        match c {
            b'\n' => {
                self.newline();
            }
            b'\r' => {
                self.cursor_x = 0;
            }
            b'\t' => {
                // Tab: move to next multiple of 8
                let spaces = 8 - (self.cursor_x % 8);
                for _ in 0..spaces {
                    self.putc(b' ');
                }
            }
            0x08 => {
                // Backspace
                self.backspace();
            }
            _ => {
                self.draw_char(self.cursor_x, self.cursor_y, c);
                self.cursor_x += 1;
                if self.cursor_x >= CONSOLE_COLS {
                    self.newline();
                }
            }
        }
    }

    /// Print a string
    pub fn puts(&mut self, s: &str) {
        for c in s.bytes() {
             self.putc(c);
        }
    }

    /// Print a string with newline
    pub fn println(&mut self, s: &str) {
        self.puts(s);
        self.newline();
    }
}

/// Global console instance
static mut CONSOLE: Console = Console::new();

/// Initialize the console (clear screen)
pub fn init() {
    unsafe {
        CONSOLE.clear();
    }
}

/// Print a string to the console
pub fn print(s: &str) {
    unsafe {
        CONSOLE.puts(s);
    }
}

/// Print a string with newline
pub fn println(s: &str) {
    unsafe {
        CONSOLE.println(s);
    }
}

/// Clear the console
pub fn clear() {
    unsafe {
        CONSOLE.clear();
    }
}

/// Set text colors
pub fn set_colors(fg_r: u8, fg_g: u8, fg_b: u8, bg_r: u8, bg_g: u8, bg_b: u8) {
    unsafe {
        CONSOLE.set_fg(fg_r, fg_g, fg_b);
        CONSOLE.set_bg(bg_r, bg_g, bg_b);
    }
}

/// Check for incoming character (Polling)
pub fn console_getc() -> Option<char> {
    unsafe {
        let status_ptr = crate::KEYBOARD_STATUS as *mut u32;
        if status_ptr.read_volatile() == 1 {
            let data_ptr = crate::KEYBOARD_DATA as *mut u32;
            let c = data_ptr.read_volatile() as u8 as char;
            
            // Acknowledge (Clear Status)
            status_ptr.write_volatile(0);
            
            return Some(c);
        }
    }
    None
}

/// Print a single character to the console
pub fn console_putc(c: char) {
    unsafe {
        CONSOLE.putc(c as u8);
    }
}
