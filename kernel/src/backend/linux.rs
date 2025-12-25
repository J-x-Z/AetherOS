use super::Backend;

pub struct LinuxBackend;

impl Backend for LinuxBackend {
    fn new() -> Self {
        panic!("Linux Backend not yet implemented on this host OS");
    }
    
    fn name(&self) -> &str {
        "KVM (Linux)"
    }
    
    fn run(&self) {
        unimplemented!();
    }
    
    unsafe fn get_framebuffer(&self, _width: usize, _height: usize) -> &[u32] {
        &[]
    }
}
