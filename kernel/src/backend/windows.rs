use super::Backend;

#[cfg(target_os = "windows")]
use windows::{
    Win32::System::Hypervisor::*,
    Win32::Foundation::*,
};
use std::ptr;

use aether_abi::mmio::{RAM_SIZE, FB_ADDR, KEYBOARD_STATUS, KEYBOARD_DATA};
// const RAM_SIZE: usize = 4 * 1024 * 1024; 
// const FB_ADDR: usize = 0x100000;         
// const KEYBOARD_STATUS: usize = 0x80000;
// const KEYBOARD_DATA: usize = 0x80004;

pub struct WindowsBackend {
    #[cfg(target_os = "windows")]
    partition: WHV_PARTITION_HANDLE,
    mem: *mut u8,
}

// SAFETY: The memory pointer is owned by this struct
unsafe impl Send for WindowsBackend {}
unsafe impl Sync for WindowsBackend {}

// ===== Helper methods (not part of Backend trait) =====
#[cfg(target_os = "windows")]
impl WindowsBackend {
    fn load_guest(&self) {
        #[cfg(target_arch = "aarch64")]
        let guest_bin = include_bytes!("../../../apps/hello_world/guest-aarch64.bin");
        #[cfg(target_arch = "x86_64")]
        let guest_bin = include_bytes!("../../../apps/hello_world/guest-x86_64.bin");

        unsafe {
            ptr::copy_nonoverlapping(guest_bin.as_ptr(), self.mem, guest_bin.len());
        }
        println!("[Aether::WindowsBackend] Loaded guest: {} bytes", guest_bin.len());
    }
    
    unsafe fn setup_long_mode(partition: WHV_PARTITION_HANDLE, mem: *mut u8) {
        const PML4_START: u64 = 0x9000;
        const PDPTE_START: u64 = 0xA000;
        const PDE_START: u64 = 0xB000;

        let pml4 = std::slice::from_raw_parts_mut(mem.add(PML4_START as usize) as *mut u64, 512);
        let pdpte = std::slice::from_raw_parts_mut(mem.add(PDPTE_START as usize) as *mut u64, 512);
        let pde = std::slice::from_raw_parts_mut(mem.add(PDE_START as usize) as *mut u64, 512);

        pml4[0] = PDPTE_START | 0x3;
        pdpte[0] = PDE_START | 0x3;
        pde[0] = 0x0 | 0x83;
        pde[1] = 0x200000 | 0x83;

        let reg_names = [
            WHvX64RegisterCr0, WHvX64RegisterCr3, WHvX64RegisterCr4, WHvX64RegisterEfer,
            WHvX64RegisterCs, WHvX64RegisterDs, WHvX64RegisterEs, WHvX64RegisterFs,
            WHvX64RegisterGs, WHvX64RegisterSs, WHvX64RegisterRip, WHvX64RegisterRsp,
            WHvX64RegisterRflags,
        ];

        let mut reg_values = [WHV_REGISTER_VALUE::default(); 13];
        reg_values[0].Reg64 = 0x80000001;
        reg_values[1].Reg64 = PML4_START;
        reg_values[2].Reg64 = 1 << 5;
        reg_values[3].Reg64 = 0x500;

        let mut cs = WHV_X64_SEGMENT_REGISTER::default();
        cs.Base = 0;
        cs.Limit = 0xffffffff;
        cs.Selector = 1 << 3;
        cs.Anonymous.Attributes = 0xA09B;

        let mut ds = WHV_X64_SEGMENT_REGISTER::default();
        ds.Base = 0;
        ds.Limit = 0xffffffff;
        ds.Selector = 2 << 3;
        ds.Anonymous.Attributes = 0xC093;

        reg_values[4].Segment = cs;
        reg_values[5].Segment = ds;
        reg_values[6].Segment = ds;
        reg_values[7].Segment = ds;
        reg_values[8].Segment = ds;
        reg_values[9].Segment = ds;
        reg_values[10].Reg64 = 0x0;
        reg_values[11].Reg64 = 0x200000;
        reg_values[12].Reg64 = 0x2;

        WHvSetVirtualProcessorRegisters(
            partition, 0,
            &reg_names as *const _ as *const _,
            13,
            &reg_values as *const _ as *const _,
        ).expect("Failed to set registers");
    }
}

// ===== Backend trait implementation (single block for all trait methods) =====
impl Backend for WindowsBackend {
    #[cfg(target_os = "windows")]
    fn new() -> Self {
        println!("[Aether::WindowsBackend] Creating VM ({}MB RAM) for x86_64...", RAM_SIZE / 1024 / 1024);
        
        unsafe {
            let mut capability = WHV_CAPABILITY::default();
            let result = WHvGetCapability(
                WHvCapabilityCodeHypervisorPresent,
                &mut capability as *mut _ as *mut _,
                std::mem::size_of::<WHV_CAPABILITY>() as u32,
                None,
            );
            
            if result.is_err() {
                panic!("Windows Hypervisor Platform not available. Enable Hyper-V.");
            }
            
            let partition = WHvCreatePartition().expect("Failed to create partition");
            
            let processor_count: u32 = 1;
            WHvSetPartitionProperty(
                partition,
                WHvPartitionPropertyCodeProcessorCount,
                &processor_count as *const _ as *const _,
                std::mem::size_of::<u32>() as u32,
            ).expect("Failed to set processor count");
            
            WHvSetupPartition(partition).expect("Failed to setup partition");
            
            let mem = {
                let layout = std::alloc::Layout::from_size_align(RAM_SIZE, 4096).unwrap();
                std::alloc::alloc_zeroed(layout)
            };
            
            if mem.is_null() {
                panic!("Failed to allocate guest memory");
            }
            
            WHvMapGpaRange(
                partition,
                mem as *const _,
                0,
                RAM_SIZE as u64,
                WHvMapGpaRangeFlagRead | WHvMapGpaRangeFlagWrite | WHvMapGpaRangeFlagExecute,
            ).expect("Failed to map memory");
            
            WHvCreateVirtualProcessor(partition, 0, 0).expect("Failed to create vCPU");
            
            Self::setup_long_mode(partition, mem);
            
            let backend = WindowsBackend { partition, mem };
            backend.load_guest();
            backend
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    fn new() -> Self {
        panic!("Windows Backend not available on this platform");
    }

    fn name(&self) -> &str {
        "WHV (Windows)"
    }
    
    #[cfg(target_os = "windows")]
    fn step(&self) -> super::ExitReason {
        unsafe {
            let mut exit_context = WHV_RUN_VP_EXIT_CONTEXT::default();
            
            let result = WHvRunVirtualProcessor(
                self.partition,
                0,
                &mut exit_context as *mut _ as *mut _,
                std::mem::size_of::<WHV_RUN_VP_EXIT_CONTEXT>() as u32,
            );
            
            if result.is_err() {
                eprintln!("[Error] vCPU run failed");
                return super::ExitReason::Halt;
            }
            
            match exit_context.ExitReason {
                WHvRunVpExitReasonMemoryAccess => {
                    // Check if Read or Write (simplification: assume Read for now or check access info)
                    // The AccessInfo is in exit_context.MemoryAccess
                    let addr = exit_context.Anonymous.MemoryAccess.Gpa;
                    super::ExitReason::Mmio(addr)
                }
                WHvRunVpExitReasonX64Halt | WHvRunVpExitReasonCanceled => {
                    println!("[Aether::WindowsBackend] Guest halted");
                    super::ExitReason::Halt
                }
                WHvRunVpExitReasonX64IoPortAccess => {
                    let port = exit_context.Anonymous.IoPortAccess.PortNumber;
                    super::ExitReason::Io(port)
                }
                _ => {
                    println!("[Debug] Exit reason: {:?}", exit_context.ExitReason);
                    super::ExitReason::Unknown
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    fn step(&self) -> super::ExitReason {
        super::ExitReason::Halt
    }
    
    unsafe fn get_framebuffer(&self, width: usize, height: usize) -> &[u32] {
        let ptr = self.mem.add(FB_ADDR) as *const u32;
        std::slice::from_raw_parts(ptr, width * height)
    }
    
    fn inject_key(&self, c: char) {
        unsafe {
            let status_ptr = self.mem.add(KEYBOARD_STATUS) as *mut u32;
            let data_ptr = self.mem.add(KEYBOARD_DATA) as *mut u32;
            
            if std::ptr::read_volatile(status_ptr) == 0 {
                std::ptr::write_volatile(data_ptr, c as u32);
                std::ptr::write_volatile(status_ptr, 1);
            }
        }
    }
}
