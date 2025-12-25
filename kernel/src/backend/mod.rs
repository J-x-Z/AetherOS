pub trait Backend: Sync + Send {
    fn new() -> Self;
    fn name(&self) -> &str;
    fn run(&self);
    // Unsafe because it returns a slice to raw memory modified by another thread
    unsafe fn get_framebuffer(&self, width: usize, height: usize) -> &[u32];
}

#[cfg(target_os = "macos")]
mod macos;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "freebsd")]
mod freebsd;

#[cfg(target_os = "macos")]
pub use macos::MacBackend as CurrentBackend;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use linux::LinuxBackend as CurrentBackend;

#[cfg(target_os = "windows")]
pub use windows::WindowsBackend as CurrentBackend;

#[cfg(target_os = "freebsd")]
pub use freebsd::FreeBsdBackend as CurrentBackend;
