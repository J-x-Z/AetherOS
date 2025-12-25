use super::Backend;

pub struct WindowsBackend;

impl Backend for WindowsBackend {
    fn new() -> Self {
        panic!("Windows Backend (Hyper-V) not implemented yet");
    }
    
    fn name(&self) -> &str {
        "Windows Hypervisor Platform (WHP)"
    }
    
    fn run(&self) {
        unimplemented!();
    }
    
    unsafe fn get_framebuffer(&self, _width: usize, _height: usize) -> &[u32] {
        &[]
    }
}
