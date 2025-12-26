use super::Backend;

#[cfg(target_os = "windows")]
use windows::{
    Win32::System::Hypervisor::*,
    Win32::Foundation::*,
};
use std::ptr;

// Memory layout constants (same as other backends)
const RAM_SIZE: usize = 4 * 1024 * 1024; // 4MB
const FB_ADDR: usize = 0x100000;         // 1MB offset for framebuffer
const KEYBOARD_STATUS: usize = 0x80000;
const KEYBOARD_DATA: usize = 0x80004;

pub struct WindowsBackend {
    #[cfg(target_os = "windows")]
    partition: WHV_PARTITION_HANDLE,
    mem: *mut u8,
}

// SAFETY: The memory pointer is owned by this struct
unsafe impl Send for WindowsBackend {}
unsafe impl Sync for WindowsBackend {}

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
}

impl Backend for WindowsBackend {
    #[cfg(target_os = "windows")]
    fn new() -> Self {
        println!("[Aether::WindowsBackend] Creating VM ({}MB RAM) for x86_64...", RAM_SIZE / 1024 / 1024);
        
        unsafe {
            // 1. Check WHV capability
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
            
            // 2. Create partition
            let mut partition = WHV_PARTITION_HANDLE::default();
            WHvCreatePartition(&mut partition).expect("Failed to create partition");
            
            // 3. Set processor count
            let processor_count: u32 = 1;
            WHvSetPartitionProperty(
                partition,
                WHvPartitionPropertyCodeProcessorCount,
                &processor_count as *const _ as *const _,
                std::mem::size_of::<u32>() as u32,
            ).expect("Failed to set processor count");
            
            // 4. Setup partition
            WHvSetupPartition(partition).expect("Failed to setup partition");
            
            // 5. Allocate memory
            let mem = {
                let layout = std::alloc::Layout::from_size_align(RAM_SIZE, 4096).unwrap();
                std::alloc::alloc_zeroed(layout)
            };
            
            if mem.is_null() {
                panic!("Failed to allocate guest memory");
            }
            
            // 6. Map memory
            WHvMapGpaRange(
                partition,
                mem as *const _,
                0, // Guest physical address
                RAM_SIZE as u64,
                WHvMapGpaRangeFlagRead | WHvMapGpaRangeFlagWrite | WHvMapGpaRangeFlagExecute,
            ).expect("Failed to map memory");
            
            // 7. Create virtual processor
            WHvCreateVirtualProcessor(partition, 0, 0).expect("Failed to create vCPU");
            
            // 8. Setup Long Mode
            Self::setup_long_mode(partition, mem);
            
            let backend = WindowsBackend { partition, mem };
            backend.load_guest();
            backend
        }
    }

    #[cfg(target_os = "windows")]
    unsafe fn setup_long_mode(partition: WHV_PARTITION_HANDLE, mem: *mut u8) {
        // Constants (Same as Linux)
        const PML4_START: u64 = 0x9000;
        const PDPTE_START: u64 = 0xA000;
        const PDE_START: u64 = 0xB000;

        // 1. Setup Page Tables
        let pml4 = std::slice::from_raw_parts_mut(mem.add(PML4_START as usize) as *mut u64, 512);
        let pdpte = std::slice::from_raw_parts_mut(mem.add(PDPTE_START as usize) as *mut u64, 512);
        let pde = std::slice::from_raw_parts_mut(mem.add(PDE_START as usize) as *mut u64, 512);

        pml4[0] = PDPTE_START | 0x3;
        pdpte[0] = PDE_START | 0x3;
        pde[0] = 0x0 | 0x83; // 2MB Huge Page
        pde[1] = 0x200000 | 0x83;

        // 2. Prepare Registers
        // In WHP we set registers via an array of Names and Values
        use windows::Win32::System::Hypervisor::*;

        let reg_names = [
            WHvX64RegisterCr0,
            WHvX64RegisterCr3,
            WHvX64RegisterCr4,
            WHvX64RegisterEfer,
            WHvX64RegisterCs,
            WHvX64RegisterDs,
            WHvX64RegisterEs,
            WHvX64RegisterFs,
            WHvX64RegisterGs,
            WHvX64RegisterSs,
            WHvX64RegisterRip,
            WHvX64RegisterRsp,
            WHvX64RegisterRflags,
        ];

        let mut reg_values = [WHV_REGISTER_VALUE::default(); 13];

        // CR0: PE | PG
        reg_values[0].Reg64 = 0x80000001; 
        // CR3: PML4
        reg_values[1].Reg64 = PML4_START;
        // CR4: PAE
        reg_values[2].Reg64 = 1 << 5;
        // EFER: LME | LMA
        reg_values[3].Reg64 = 0x500;

        // Helper for Segment
        fn make_segment(selector: u16, type_: u32) -> WHV_X64_SEGMENT_REGISTER {
            WHV_X64_SEGMENT_REGISTER {
                Base: 0,
                Limit: 0xffffffff,
                Selector: selector,
                Attributes: (1 << 7) | (1 << 4) | (1 << 12) | // Present | S | Granularity
                           (type_ as u16) | // Type
                           (1 << 13), // Long Mode (CS only?) - NO, L bit is in attributes bit 13?
                           // Actually WHP attributes are bitfields.
                           // Cleaner would be manual construction if needed, but lets assume minimal valid.
                           // Attributes format:
                           // type:4, s:1, dpl:2, p:1, avl:1, l:1, db:1, g:1
                           // type=11 (Code), s=1, p=1, l=1, g=1 => 0b1010_0000_1001_1011 = 0xA09B
                 ..Default::default()
            }
        }
        
        // Let's use raw u16 for Attributes because the struct helper is tricky in Windows crate
        // 0xA09B: P=1, L=1, S=1, Type=11 (Code), G=1
        
        let mut cs = WHV_X64_SEGMENT_REGISTER::default();
        cs.Base = 0;
        cs.Limit = 0xffffffff;
        cs.Selector = 1 << 3;
        cs.Attributes = 0xA09B; // Long Mode Code

        let mut ds = WHV_X64_SEGMENT_REGISTER::default();
        ds.Base = 0;
        ds.Limit = 0xffffffff;
        ds.Selector = 2 << 3;
        ds.Attributes = 0xC093; // 0b1100_0000_1001_0011 (P=1, DB=1, G=1, S=1, Type=3 Data)

        reg_values[4].Segment = cs;
        reg_values[5].Segment = ds;
        reg_values[6].Segment = ds; // ES
        reg_values[7].Segment = ds; // FS
        reg_values[8].Segment = ds; // GS
        reg_values[9].Segment = ds; // SS (Actually SS not used in long mode but good to set)

        // RIP
        reg_values[10].Reg64 = 0x0;
        // RSP
        reg_values[11].Reg64 = 0x200000;
        // RFLAGS
        reg_values[12].Reg64 = 0x2;

        WHvSetVirtualProcessorRegisters(
            partition,
            0,
            &reg_names as *const _ as *const _,
            13,
            &reg_values as *const _ as *const _,
        ).expect("Failed to set registers");
    }
    
    #[cfg(not(target_os = "windows"))]
    fn new() -> Self {
        panic!("Windows Backend not available on this platform");
    }
    
    fn name(&self) -> &str {
        "WHV (Windows)"
    }
    
    #[cfg(target_os = "windows")]
    fn run(&self) {
        println!("[Aether::WindowsBackend] Starting vCPU Loop...");
        
        unsafe {
            loop {
                let mut exit_context = WHV_RUN_VP_EXIT_CONTEXT::default();
                
                let result = WHvRunVirtualProcessor(
                    self.partition,
                    0, // vCPU index
                    &mut exit_context as *mut _ as *mut _,
                    std::mem::size_of::<WHV_RUN_VP_EXIT_CONTEXT>() as u32,
                );
                
                if result.is_err() {
                    eprintln!("[Error] vCPU run failed");
                    break;
                }
                
                match exit_context.ExitReason {
                    WHvRunVpExitReasonMemoryAccess => {
                        // Handle MMIO
                        println!("[Debug] Memory access exit");
                    }
                    WHvRunVpExitReasonX64Halt | WHvRunVpExitReasonCanceled => {
                        println!("[Aether::WindowsBackend] Guest halted");
                        break;
                    }
                    _ => {
                        println!("[Debug] Exit reason: {:?}", exit_context.ExitReason);
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    fn run(&self) {
        unimplemented!();
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
