# Fedora Quick Build Guide 🎩

**The superior Linux distribution deserves a superior guide.**

This is the streamlined build guide for Fedora users. Because you're using Fedora, you obviously know what you're doing.

## One-Command Setup

```bash
# Install all dependencies
sudo dnf install -y gcc gcc-c++ pkg-config fontconfig-devel gtk3-devel openssl-devel cmake git

# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Build & Run

```bash
# Clone the repo
git clone https://github.com/Gnaw-Software/Jobseeker.git
cd Jobseeker

# Build release version (optimized)
cargo build --release

# Run it!
./target/release/Jobseeker
```

## That's It

Seriously, that's it. Because Fedora just works™.

## Optional: Desktop Integration

Create a desktop launcher:

```bash
# Create desktop file
cat > ~/.local/share/applications/jobseeker.desktop << 'EOF'
[Desktop Entry]
Name=Jobseeker
Comment=Modern job search application
Exec=/path/to/Jobseeker/target/release/Jobseeker
Icon=applications-internet
Terminal=false
Type=Application
Categories=Office;Network;
EOF

# Update desktop database
update-desktop-database ~/.local/share/applications/
```

Don't forget to replace `/path/to/Jobseeker` with your actual path.

## Development Tips

```bash
# Fast development builds
cargo run

# Run tests
cargo test

# Linting
cargo clippy

# Format code
cargo fmt

# Watch for changes and rebuild (install cargo-watch first)
cargo install cargo-watch
cargo watch -x run
```

## Troubleshooting

### "Command not found: cargo"
You forgot to source the Rust environment:
```bash
source $HOME/.cargo/env
```

### Build is slow
Enable parallel compilation and use mold linker:
```bash
# Install mold
sudo dnf install mold

# Add to ~/.cargo/config.toml
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[build]
jobs = 8  # Adjust to your CPU cores

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF
```

## Why Fedora?

- **Latest packages:** Always up-to-date Rust, GTK, etc.
- **SELinux:** Proper security by default
- **DNF:** Package management that doesn't make you cry
- **Bleeding edge:** You get new features first
- **Red Hat backing:** Enterprise quality without the enterprise pain

## Contributing

If you find a bug or want to contribute, remember: we develop on Fedora, so your patches better work on Fedora first. 😉

---

**Fedora users:** You're using the best distro. Now go build some amazing software.

**Ubuntu users:** See `BUILD.md` for your more... verbose instructions.