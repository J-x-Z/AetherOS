# AetherOS: Feature Complete

## Accomplishments
1.  **Modular Architecture**: Refactored `kernel` into `backend/{macos,linux}`.
2.  **Graphics System**:
    - Host: Added `minifb` window (640x480).
    - Memory: Mapped 4MB RAM, Framebuffer at `0x100000`.
    - Guest: Added `draw_pixel` to `aether-user`.
    - Demo: `hello_world` draws a gradient.
3.  **Linux Support**:
    - Added `backend/linux.rs` scaffolding for KVM support (ready for implementation).

## Runtime Status (macOS M1/M2)
- The Kernel launches, creates the VM, and opens the Graphics Window.
- The Guest executes fully (Run Loop active).
- **Fixed "Black Screen" (Translation Fault)**: The `0x20` Translation Fault was caused by 16KB vs 64KB page alignment mismatches in Stage 2 mapping.
- **Solution**:
  1. Increased Host Memory Allocation alignment to 64KB (`0x10000`).
  2. Mapped memory as a single contiguous `RWX` block to match hardware block sizes.
  3. This successfully eliminated the Stage 2 fault.
- Implement `backend/linux.rs` logic using `kvm-ioctls` crate.
- Add `HyperCall::PollInput` for Keyboard/Mouse.
