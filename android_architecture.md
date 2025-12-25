# AetherOS on Android: Native Hardware Architecture

You are absolutely correct. On Windows/macOS, we run as a "Guest Application" inside a Window (`minifb`). But on Android (specifically as an OS replacement or High-Privilege Service), AetherOS operates differently.

## 1. Execution Context (PID 1 vs HAL)
Instead of running on top of a desktop compositor, AetherOS on Android runs in one of two modes:

### Mode A: The "Hypervisor Shim" (Root/System)
- **Role**: AetherOS replaces the Android Runtime (ART/Zygote).
- **Graphics**: It takes control of the **Hardware Composer (HWC)** or `DRM/KMS` directly.
- **Input**: Reads directly from `/dev/input/event*`.

### Mode B: Bare Metal (future)
- **Role**: AetherOS *is* the Kernel (using the Linux backend as a Type-1 Hypervisor shim).

## 2. Removing the Window System
In `kernel/src/main.rs`, we already gated `minifb` out for Android. The next step is replacing the "Sleep Loop" with a **Direct Rendering Backend**.

### Proposed Architecture for `backend/android.rs`

```rust
pub struct AndroidBackend {
    // Direct access to GPU/Display hardware
    drm_device: std::fs::File, 
    framebuffer_map: *mut u8,
}

impl Backend for AndroidBackend {
    fn new() -> Self {
        // 1. Open /dev/dri/card0 (Direct Rendering Manager)
        // 2. Perform IOCTLs to set video mode (KMS)
        // 3. Map the "Dumb Buffer" (Video RAM) to host memory
        // 4. Return this pointer as the "Framebuffer"
    }

    unsafe fn get_framebuffer(&self) -> &[u32] {
        // Return pointer to ACTUAL Video RAM, not a heap buffer
    }
}
```

## 3. Zero-Copy Performance
In the Desktop version, we copy Guest RAM -> Host RAM -> Window Texture.
In the **Android Native** version:
1.  We configure the Display Controller (CRTC) to scan out **directly** from the Guest's Physical RAM address (if using IOMMU/SMMU).
2.  Or simpler: The Guest writes to a memory region that IS the mapped Video Output.
3.  **Result**: 0% CPU usage for display. Pure hardware scan-out.

## 4. Input Handling
- Desktop: `minifb` events.
- Android: Parse `/dev/input/event0` (Touchscreen) raw byte streams directly in the Kernel thread, converting them to ABI events for the Guest.

## 5. The "Bionic" Challenge & Driver Blobs
You raised a critical point: Vendor drivers (GPU, WiFi) are compiled against `bionic` (Android libc) and often present as proprietary HAL modules.

### Strategy A: Being Native (The `libhybris` Route)
Since AetherOS compiles to `aarch64-linux-android`, it **native links** against Bionic!
- We don't strictly need `libhybris` if we stay within the Android Linker namespace.
- **Challenge**: Vendor drivers speak **HIDL/AIDL**. We would need to implement a "Rust HwBinder" to talk to the GPU HAL.

### Strategy B: The `libhybris` Bridge
If we want to run AetherOS on a standard Linux kernel (bypassing Android Init entirely), we need `libhybris` to load the Android-specific `.so` blobs (e.g., `libGLESv2_adreno.so`) inside a glibc/musl environment.
- **Precedent**: SailfishOS and Ubuntu Touch use this.
- **Recommendation**: For the "Android Replacement" goal, we should stick to **Strategy A**: run *as* an Android executable (using Bionic), but replace the upper Java Framework (SurfaceFlinger/Zygote).

## 6. Security Barriers (SELinux & AVB)
You asked about permissions. Android is notorious for locking down Native processes.

### The Obstacles
1.  **SELinux**: Even as `root`, you cannot just open `/dev/kvm` or `/dev/dri` if the Security Policy (`sepolicy`) forbids your domain from doing so.
2.  **Seccomp**: The Kernel filters "dangerous" syscalls.
3.  **AVB (Verified Boot)**: If we modify `boot.img` to spawn AetherOS, the phone will refuse to boot unless the bootloader is unlocked.

### The Solution: Access Levels

#### Level 1: "Rooted App" (Development)
- Device must be **Unlock Bootloader**.
- We use `su` to launch AetherOS.
- We perform `setenforce 0` (Permissive Mode) to bypass SELinux temporarily.

#### Level 2: "Custom ROM" (Production)
- We modify `init.rc` to start `service aether /system/bin/aether-kernel`.
- We compile a custom **SEPolicy (`.te` file)** granting `allow aether kvm_device:chr_file rw_file_perms;`.
- This is how OEMs add their own services. We play by the same rules.

## Summary
The current `minifb` implementation is a "Simulator" for development.
The Android target will evolve into a "Driver" that talks to silicon, potentially using `libhybris` or native HIDL to bridge proprietary vendor blobs.

