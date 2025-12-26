# AetherOS

**AetherOS** is a cross-platform software stack that provides a unified runtime environment across desktop operating systems and the Aether kernel.

> Like Android is to Linux, AetherOS is to Aether.

## Features

- 🖥️ **Cross-Platform** - Runs on macOS, Linux, Windows, and Aether kernel
- 🔄 **Unified API** - Same application works everywhere
- 🎮 **Graphics Support** - Built-in framebuffer and windowing
- ⌨️ **Input Handling** - Keyboard and pointing device support
- 🚀 **Unikernel Runtime** - Execute lightweight guest applications

## Supported Platforms

| Platform | Backend | Status |
|----------|---------|--------|
| macOS | Hypervisor.framework | ✅ Working |
| Linux | KVM | ✅ Working |
| Windows | WHP (Hypervisor Platform) | 🔧 In Progress |
| Aether Kernel | Native | 🔧 In Progress |

## Building

```bash
# Build for current platform
cargo build -p aetheros

# Run
cargo run -p aetheros
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Applications                         │
├─────────────────────────────────────────────────────────┤
│                    AetherOS Runtime                     │
│    ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│    │  macOS   │ │  Linux   │ │ Windows  │ │  Aether  │ │
│    │  (Hvf)   │ │  (KVM)   │ │  (WHP)   │ │ (Native) │ │
│    └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
└─────────────────────────────────────────────────────────┘
```

## Project Structure

```
AetherOS/
├── aetheros/      # Main runtime
│   └── src/
│       ├── main.rs
│       └── backend/
│           ├── macos.rs    # Hypervisor.framework
│           ├── linux.rs    # KVM
│           └── windows.rs  # WHP
├── aether-core/   # Shared abstractions
├── abi/           # Application Binary Interface
├── user/          # Userspace library for guests
└── apps/          # Example applications
    ├── hello_world/
    └── wasm_simple/
```

## Related Projects

- [Aether](https://github.com/J-x-Z/Aether) - Bare-metal hybrid kernel

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
