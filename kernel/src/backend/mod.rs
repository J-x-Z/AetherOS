#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExitReason {
    Yield,      // Time slice expired or voluntary yield
    Io(u16),    // IO Access (Port)
    Mmio(u64),  // MMIO Access (Address)
    Halt,       // Guest Halted
    Unknown,    // Other exit reasons
}

pub trait Backend: Sync + Send {
    fn new() -> Self where Self: Sized;
    fn name(&self) -> &str;
    
    /// Run the vCPU corresponding to one time slice or until an exit event.
    fn step(&self) -> ExitReason;
    
    /// Legacy run loop (will be deprecated by Scheduler)
    fn run(&self) {
        loop {
            if self.step() == ExitReason::Halt {
                break;
            }
        }
    }
    
    // Unsafe because it returns a slice to raw memory modified by another thread
    unsafe fn get_framebuffer(&self, width: usize, height: usize) -> &[u32];

    // Inject a key press into the Guest (Default: Do nothing)
    fn inject_key(&self, _c: char) {}
}

// ===== Platform-specific module declarations =====

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "freebsd")]
mod freebsd;

#[cfg(target_os = "openbsd")]
mod openbsd;

#[cfg(target_os = "netbsd")]
mod netbsd;

#[cfg(target_os = "dragonfly")]
mod dragonfly;

// ===== Platform-specific backend exports =====

#[cfg(target_os = "macos")]
pub use macos::MacBackend as CurrentBackend;

#[cfg(target_os = "linux")]
pub use linux::LinuxBackend as CurrentBackend;

#[cfg(target_os = "android")]
pub use android::AndroidBackend as CurrentBackend;

#[cfg(target_os = "windows")]
pub use windows::WindowsBackend as CurrentBackend;

#[cfg(target_os = "freebsd")]
pub use freebsd::FreeBsdBackend as CurrentBackend;

#[cfg(target_os = "openbsd")]
pub use openbsd::OpenBsdBackend as CurrentBackend;

#[cfg(target_os = "netbsd")]
pub use netbsd::NetBsdBackend as CurrentBackend;

#[cfg(target_os = "dragonfly")]
pub use dragonfly::DragonFlyBackend as CurrentBackend;
