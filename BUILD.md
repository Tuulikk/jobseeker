# Build Instructions - Jobseeker

This guide will help you build Jobseeker from source on different operating systems.

> **Note:** This project is developed primarily on **Fedora Linux**. While it builds on all major platforms, Fedora instructions are listed first and are the most tested.

## Prerequisites

### All Platforms
- **Rust** (latest stable): Install from [rustup.rs](https://rustup.rs/)
- **Git**: For cloning the repository

### Linux (Fedora/RHEL)
```bash
# Install system dependencies
sudo dnf install -y \
    gcc \
    gcc-c++ \
    pkg-config \
    fontconfig-devel \
    gtk3-devel \
    openssl-devel \
    cmake
```

### Linux (Ubuntu/Debian)
```bash
# Install system dependencies
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libgtk-3-dev \
    libssl-dev \
    cmake
```

### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Optional: Install via Homebrew for additional tools
brew install pkg-config
```

### Windows
- Install **Visual Studio Build Tools** OR **MinGW-w64**
  - Visual Studio: Download from [visualstudio.microsoft.com](https://visualstudio.microsoft.com/downloads/)
  - Select "Desktop development with C++" workload
  - OR use MinGW-w64 from [mingw-w64.org](https://www.mingw-w64.org/)

## Building

### 1. Clone the Repository
```bash
git clone https://github.com/Gnaw-Software/Jobseeker.git
cd Jobseeker
```

### 2. Build Release Version
```bash
# Build optimized release binary
cargo build --release

# The binary will be located at:
# Linux/macOS: ./target/release/Jobseeker
# Windows: .\target\release\Jobseeker.exe
```

### 3. Run the Application
```bash
# Linux/macOS
./target/release/Jobseeker

# Windows
.\target\release\Jobseeker.exe
```

## Development Build

For faster compilation during development (but slower runtime):
```bash
cargo build
cargo run
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run clippy (linter)
cargo clippy

# Check without building
cargo check
```

## Troubleshooting

### Linux: "failed to run custom build command for fontconfig-sys"
**Solution:** Install fontconfig development libraries
```bash
sudo dnf install fontconfig-devel     # Fedora/RHEL
sudo apt install libfontconfig1-dev  # Ubuntu/Debian
```

### Linux: "failed to run custom build command for gtk-sys"
**Solution:** Install GTK3 development libraries
```bash
sudo dnf install gtk3-devel     # Fedora/RHEL
sudo apt install libgtk-3-dev  # Ubuntu/Debian
```

### Windows: "link.exe not found"
**Solution:** Install Visual Studio Build Tools with C++ support, or use MinGW-w64

### macOS: "xcrun: error: invalid active developer path"
**Solution:** Install Xcode Command Line Tools
```bash
xcode-select --install
```

### All Platforms: Slow compilation
**Solution:** Enable parallel compilation and use faster linker
```bash
# Add to ~/.cargo/config.toml (Linux/macOS) or %USERPROFILE%\.cargo\config.toml (Windows)
[build]
jobs = 8  # Adjust to your CPU core count

# Linux only - use mold linker for faster linking
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

## Binary Size Optimization

To reduce binary size:
```bash
# Strip debug symbols (Linux/macOS)
strip target/release/Jobseeker

# Windows: already stripped in release mode
```

Add to `Cargo.toml` for smaller binaries:
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
strip = true        # Strip symbols (Rust 1.59+)
```

## Platform-Specific Notes

### Linux
- **Desktop file:** Create `~/.local/share/applications/jobseeker.desktop` for application menu integration
- **Icon:** Place application icon in `~/.local/share/icons/`
- **Database:** Stored in project directory as `jobseeker.db`

### macOS
- **App Bundle:** Consider using `cargo-bundle` to create `.app` bundle
- **Code Signing:** May be required for distribution

### Windows
- **Installer:** Consider using WiX or Inno Setup for installer creation
- **Antivirus:** Some AV software may flag the binary - this is a false positive

## Dependencies

Key dependencies and their purposes:
- **iced**: GUI framework (cross-platform)
- **sqlx**: SQLite database access
- **tokio**: Async runtime
- **reqwest**: HTTP client for API calls
- **serde/serde_json**: Serialization
- **pulldown-cmark**: Markdown parsing
- **docx-rs**: Word document generation
- **rfd**: Native file dialogs

All dependencies use Rust-native implementations or rustls for TLS, making the build process more reliable across platforms.

## CI/CD

The project includes GitHub Actions workflow (`.github/workflows/rust.yml`) that automatically:
- Builds the project on Linux (Ubuntu for CI compatibility, but Fedora is preferred for development), macOS, and Windows
- Runs tests
- Runs clippy linting

**Note:** While CI uses Ubuntu for GitHub Actions compatibility, the primary development environment is **Fedora Linux**.

## Getting Help

If you encounter build issues:
1. Check this document for your specific error
2. Ensure Rust is up to date: `rustup update`
3. Clean build artifacts: `cargo clean` then rebuild
4. Check GitHub Issues for similar problems
5. Create a new issue with your build error and platform details

## License

See LICENSE file in the repository.