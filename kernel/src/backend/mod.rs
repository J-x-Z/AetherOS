pub mod macos;

pub trait Backend: Sync + Send {
    fn new() -> Self;
    fn name(&self) -> &str;
    fn run(&self);
    // Unsafe because it returns a slice to raw memory modified by another thread
    unsafe fn get_framebuffer(&self, width: usize, height: usize) -> &[u32];
}

#[cfg(target_os = "macos")]
pub use macos::MacBackend as CurrentBackend;

#[cfg(target_os = "linux")]
pub use linux::LinuxBackend as CurrentBackend;

mod linux;
