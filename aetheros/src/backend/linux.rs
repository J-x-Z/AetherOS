//! Linux KVM Backend for AetherOS
//! 
//! This backend uses Linux KVM to run the guest VM.
//! Currently supports aarch64 guests on aarch64 hosts.

use super::Backend;
use std::ptr;

use aether_abi::mmio::{RAM_SIZE, FB_ADDR, KEYBOARD_STATUS, KEYBOARD_DATA};
// const RAM_SIZE: usize = 4 * 1024 * 1024; 
// const FB_ADDR: usize = 0x100000;         
// const KEYBOARD_STATUS: usize = 0x80000;
// const KEYBOARD_DATA: usize = 0x80004;

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
mod kvm_impl {
    use super::*;
    use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};
    use kvm_bindings::kvm_userspace_memory_region;
    
    pub struct LinuxBackendInner {
        pub kvm: Kvm,
        pub vm: VmFd,
        pub vcpu: VcpuFd,
        pub mem: *mut u8,
    }
    
    impl LinuxBackendInner {
        pub fn new() -> Self {
            println!("[Aether::LinuxBackend] Creating VM ({}MB RAM)...", RAM_SIZE / 1024 / 1024);
            
            // 1. Open KVM
            let kvm = Kvm::new().expect("Failed to open /dev/kvm");
            println!("[Aether::LinuxBackend] KVM API version: {}", kvm.get_api_version());
            
            // 2. Create VM
            let vm = kvm.create_vm().expect("Failed to create VM");
            
            // 3. Allocate memory
            let mem = unsafe {
                let layout = std::alloc::Layout::from_size_align(RAM_SIZE, 0x10000).unwrap();
                std::alloc::alloc_zeroed(layout)
            };
            
            if mem.is_null() {
                panic!("Failed to allocate guest memory");
            }
            
            // 4. Map memory to guest
            let mem_region = kvm_userspace_memory_region {
                slot: 0,
                guest_phys_addr: 0,
                memory_size: RAM_SIZE as u64,
                userspace_addr: mem as u64,
                flags: 0,
            };
            
            unsafe {
                vm.set_user_memory_region(mem_region)
                    .expect("Failed to set memory region");
            }
            
            // 5. Create vCPU
            let vcpu = vm.create_vcpu(0).expect("Failed to create vCPU");
            
            // 6. Initialize vCPU (ARM64)
            {
                use kvm_bindings::kvm_vcpu_init;
                let mut kvi = kvm_vcpu_init::default();
                vm.get_preferred_target(&mut kvi).expect("Failed to get preferred target");
                vcpu.vcpu_init(&kvi).expect("Failed to init vCPU");
            }
            
            // 7. Load guest
            let guest_bin = include_bytes!("../../../apps/hello_world/guest-aarch64.bin");
            unsafe {
                ptr::copy_nonoverlapping(guest_bin.as_ptr(), mem, guest_bin.len());
            }
            println!("[Aether::LinuxBackend] Loaded guest: {} bytes", guest_bin.len());
            
            // 8. Set initial registers (PC=0, SP=top of RAM)
            // Note: ARM64 KVM uses kvm_one_reg for individual registers
            // This is a simplified implementation
            
            LinuxBackendInner { kvm, vm, vcpu, mem }
        }
        
        pub fn step(&self) -> super::ExitReason {
            match self.vcpu.run() {
                Ok(exit_reason) => match exit_reason {
                    VcpuExit::Hvc => {
                        // println!("[Guest] Hypercall received");
                        super::ExitReason::Yield
                    }
                    VcpuExit::MmioRead(addr, _data) => {
                        super::ExitReason::Mmio(addr)
                    }
                    VcpuExit::MmioWrite(addr, _data) => {
                        super::ExitReason::Mmio(addr)
                    }
                    VcpuExit::Shutdown => {
                        println!("[Aether::LinuxBackend] Guest shutdown");
                        super::ExitReason::Halt
                    }
                    VcpuExit::SystemEvent(_event_type, _flags) => {
                        super::ExitReason::Unknown
                    }
                    _ => {
                        super::ExitReason::Unknown
                    }
                },
                Err(e) => {
                    eprintln!("[Error] vCPU run failed: {}", e);
                    super::ExitReason::Halt
                }
            }
        }
        
        pub fn get_mem(&self) -> *mut u8 {
            self.mem
        }
    }
}

pub struct LinuxBackend {
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    inner: kvm_impl::LinuxBackendInner,

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    inner: kvm_impl_x86::LinuxBackendInner,
    
    #[cfg(not(target_os = "linux"))]
    mem: *mut u8,
}

unsafe impl Send for LinuxBackend {}
unsafe impl Sync for LinuxBackend {}

impl LinuxBackend {
    pub fn new() -> Self {
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            LinuxBackend {
                inner: kvm_impl::LinuxBackendInner::new(),
            }
        }
        
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            LinuxBackend {
                inner: kvm_impl_x86::LinuxBackendInner::new(),
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            panic!("Linux Backend not available on this OS");
        }
    }
}

impl Backend for LinuxBackend {

    
    fn name(&self) -> &str {
        "KVM (Linux)"
    }
    
    fn step(&self) -> super::ExitReason {
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            self.inner.step()
        }
        
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            self.inner.step()
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            super::ExitReason::Halt
        }
    }
    unsafe fn get_framebuffer(&self, width: usize, height: usize) -> &[u32] {
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            let ptr = self.inner.get_mem().add(FB_ADDR) as *const u32;
            std::slice::from_raw_parts(ptr, width * height)
        }
        
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            let ptr = self.inner.get_mem().add(FB_ADDR) as *const u32;
            std::slice::from_raw_parts(ptr, width * height)
        }
    }
    
    fn inject_key(&self, c: char) {
        #[cfg(target_os = "linux")]
        unsafe {
            let mem = self.inner.get_mem();
            let status_ptr = mem.add(KEYBOARD_STATUS) as *mut u32;
            let data_ptr = mem.add(KEYBOARD_DATA) as *mut u32;
            
            if std::ptr::read_volatile(status_ptr) == 0 {
                std::ptr::write_volatile(data_ptr, c as u32);
                std::ptr::write_volatile(status_ptr, 1);
            }
        }
    }
}
