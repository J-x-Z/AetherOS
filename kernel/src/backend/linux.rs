//! Linux KVM Backend for AetherOS
//! 
//! This backend uses Linux KVM to run the guest VM.
//! Currently supports aarch64 guests on aarch64 hosts.

use super::Backend;
use std::ptr;

// Memory layout constants (same as macOS)
const RAM_SIZE: usize = 4 * 1024 * 1024; // 4MB
const FB_ADDR: usize = 0x100000;         // 1MB offset for framebuffer
const KEYBOARD_STATUS: usize = 0x80000;
const KEYBOARD_DATA: usize = 0x80004;

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
        
        pub fn run(&self) {
            println!("[Aether::LinuxBackend] Starting vCPU Loop...");
            
            loop {
                match self.vcpu.run() {
                    Ok(exit_reason) => {
                        match exit_reason {
                            VcpuExit::Hvc => {
                                println!("[Guest] Hypercall received");
                                // TODO: Handle hypercall
                            }
                            VcpuExit::MmioRead(addr, _data) => {
                                println!("[Debug] MMIO Read: 0x{:x}", addr);
                            }
                            VcpuExit::MmioWrite(addr, _data) => {
                                println!("[Debug] MMIO Write: 0x{:x}", addr);
                            }
                            VcpuExit::Shutdown => {
                                println!("[Aether::LinuxBackend] Guest shutdown");
                                break;
                            }
                            VcpuExit::SystemEvent(event_type, flags) => {
                                println!("[Debug] System event: type={}, flags={:?}", event_type, flags);
                            }
                            other => {
                                println!("[Debug] Unhandled exit: {:?}", other);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[Error] vCPU run failed: {}", e);
                        break;
                    }
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod kvm_impl_x86 {
    use super::*;
    use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};
    use kvm_bindings::{
        kvm_userspace_memory_region, kvm_sregs, kvm_regs, kvm_segment,
        KVM_MEM_LOG_DIRTY_PAGES,
    };

    pub struct LinuxBackendInner {
        pub kvm: Kvm,
        pub vm: VmFd,
        pub vcpu: VcpuFd,
        pub mem: *mut u8,
    }

    // x86_64 Long Mode Setup Constants
    const PML4_START: u64 = 0x9000;
    const PDPTE_START: u64 = 0xA000;
    const PDE_START: u64 = 0xB000;

    impl LinuxBackendInner {
        pub fn new() -> Self {
            println!("[Aether::LinuxBackend] Creating VM ({}MB RAM) for x86_64...", RAM_SIZE / 1024 / 1024);
            
            let kvm = Kvm::new().expect("Failed to open /dev/kvm");
            let vm = kvm.create_vm().expect("Failed to create VM");

            // Allocate Memory
            let mem = unsafe {
                let layout = std::alloc::Layout::from_size_align(RAM_SIZE, 4096).unwrap();
                std::alloc::alloc_zeroed(layout)
            };

            let mem_region = kvm_userspace_memory_region {
                slot: 0,
                guest_phys_addr: 0,
                memory_size: RAM_SIZE as u64,
                userspace_addr: mem as u64,
                flags: 0,
            };

            unsafe {
                vm.set_user_memory_region(mem_region).expect("Failed to set memory region");
            }

            let vcpu = vm.create_vcpu(0).expect("Failed to create vCPU");

            // Setup Long Mode (Page Tables + SREGS + REGS)
            unsafe {
                Self::setup_long_mode(&vcpu, mem);
            }

            // Load Guest
            let guest_bin = include_bytes!("../../../apps/hello_world/guest-x86_64.bin");
            unsafe {
                ptr::copy_nonoverlapping(guest_bin.as_ptr(), mem, guest_bin.len());
            }
            println!("[Aether::LinuxBackend] Loaded guest: {} bytes", guest_bin.len());

            LinuxBackendInner { kvm, vm, vcpu, mem }
        }

        unsafe fn setup_long_mode(vcpu: &VcpuFd, mem: *mut u8) {
            // 1. Setup Page Tables (Identity Mapping first 4MB)
            // PML4[0] -> PDPTE
            // PDPTE[0] -> PDE
            // PDE[0..2] -> 2MB Pages (Identity)
            
            let pml4 = std::slice::from_raw_parts_mut(mem.add(PML4_START as usize) as *mut u64, 512);
            let pdpte = std::slice::from_raw_parts_mut(mem.add(PDPTE_START as usize) as *mut u64, 512);
            let pde = std::slice::from_raw_parts_mut(mem.add(PDE_START as usize) as *mut u64, 512);

            pml4[0] = PDPTE_START | 0x3; // Present | Write
            pdpte[0] = PDE_START | 0x3;  // Present | Write

            // Map 4MB (2 x 2MB Huge Pages)
            // Page 0: 0x0 - 0x200000
            pde[0] = 0x0 | 0x83; // Present | Write | Huge(2MB)
            // Page 1: 0x200000 - 0x400000
            pde[1] = 0x200000 | 0x83; 

            // 2. Setup SREGS (Segments for 64-bit mode)
            let mut sregs: kvm_sregs = vcpu.get_sregs().expect("get sregs");
            
            let code_seg = kvm_segment {
                base: 0,
                limit: 0xffffffff,
                selector: 1 << 3,
                type_: 11, // Code: execute/read, accessed
                present: 1,
                dpl: 0,
                db: 0,
                s: 1, // Code/Data
                l: 1, // Long mode
                g: 1, // 4KB granularity
                avl: 0,
                unusable: 0,
                padding: 0,
            };

            let data_seg = kvm_segment {
                base: 0,
                limit: 0xffffffff,
                selector: 2 << 3,
                type_: 3, // Data: read/write, accessed
                present: 1,
                dpl: 0,
                db: 0,
                s: 1,
                l: 0,
                g: 1,
                avl: 0,
                unusable: 0,
                padding: 0,
            };

            sregs.cs = code_seg;
            sregs.ds = data_seg;
            sregs.es = data_seg;
            sregs.fs = data_seg;
            sregs.gs = data_seg;
            sregs.ss = data_seg;

            // Enable Long Mode in EFER
            sregs.efer |= 0x500; // LME | LMA
            
            // Enable Paging in CR0 and PAE in CR4
            sregs.cr3 = PML4_START;
            sregs.cr4 |= 1 << 5; // PAE
            sregs.cr0 |= 0x80000001; // PG | PE

            vcpu.set_sregs(&sregs).expect("set sregs");

            // 3. Setup REGS (Instruction Pointer)
            let mut regs: kvm_regs = vcpu.get_regs().expect("get regs");
            regs.rflags = 0x2; // Always set bit 1
            regs.rip = 0x0;    // Entry point at 0
            regs.rsp = 0x200000; // Stack at 2MB (Initial, will be moved by guest)

            vcpu.set_regs(&regs).expect("set regs");
        }

        pub fn run(&self) {
            println!("[Aether::LinuxBackend] Starting vCPU Loop...");
            loop {
                match self.vcpu.run() {
                    Ok(exit_reason) => match exit_reason {
                        VcpuExit::IoOut(addr, data) => {
                            // Simple IO Out for debugging
                            if addr == 0x3f8 && !data.is_empty() {
                                print!("{}", data[0] as char);
                            }
                        }
                        VcpuExit::MmioWrite(addr, _data) => {
                             // Handle MMIO Write (FB, Keyboard, etc.)
                             // Ideally we would map this to the same logic as ARM
                        }
                        VcpuExit::MmioRead(addr, _data) => {
                             // Handle MMIO Read
                        }
                        VcpuExit::Hlt => {
                            println!("[Guest] HLT");
                            break;
                        }
                         _ => { /* Ignore or debug */ }
                    },
                    Err(e) => {
                        eprintln!("vCPU Run failed: {}", e);
                        break;
                    }
                }
            }
        }
        
        pub fn get_mem(&self) -> *mut u8 {
            self.mem
        }
    }
}

impl Backend for LinuxBackend {
    fn new() -> Self {
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
    
    fn name(&self) -> &str {
        "KVM (Linux)"
    }
    
    fn run(&self) {
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            self.inner.run();
        }
        
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            self.inner.run();
        }

        #[cfg(not(target_os = "linux"))]
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

        #[cfg(not(target_os = "linux"))]
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
