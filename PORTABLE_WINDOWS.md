# Portable Windows Build Guide 💾

Create a portable version of Jobseeker that runs directly from a USB drive - no installation needed!

## Why Portable?

- **No installation required** - Perfect for job fairs or library computers
- **USB drive ready** - Carry your job search on a stick
- **Demo-friendly** - Show off the app anywhere
- **Zero traces** - Database stays on USB, nothing on host computer

## Prerequisites

You'll need a Windows machine (or Wine on Linux) to build the Windows executable.

### On Windows

1. Install **Rust** from [rustup.rs](https://rustup.rs/)
2. Install **Visual Studio Build Tools** (or MinGW-w64)
   - Download from [visualstudio.microsoft.com](https://visualstudio.microsoft.com/downloads/)
   - Select "Desktop development with C++"

### Cross-compile from Linux (Fedora) - Recommended!

```bash
# Install cross-compilation tools
sudo dnf install -y mingw64-gcc mingw64-gcc-c++ mingw64-winpthreads-static

# Add Windows target to Rust
rustup target add x86_64-pc-windows-gnu

# Configure cargo for MinGW (optional but recommended)
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
EOF
```

**Note:** Cross-compiling from Fedora is actually easier than building on Windows! No Visual Studio needed.

## Building Portable Windows Version

### Method 1: Build on Windows

```powershell
# Clone the repo
git clone https://github.com/Gnaw-Software/Jobseeker.git
cd Jobseeker

# Build release version
cargo build --release --target x86_64-pc-windows-msvc

# The executable is at: target\x86_64-pc-windows-msvc\release\Jobseeker.exe
```

### Method 2: Cross-compile from Linux

```bash
# Clone the repo
git clone https://github.com/Gnaw-Software/Jobseeker.git
cd Jobseeker

# Build for Windows from Linux
cargo build --release --target x86_64-pc-windows-gnu

# The executable is at: target/x86_64-pc-windows-gnu/release/Jobseeker.exe
```

## Creating the Portable Package

### Automatic (recommended)

Create a build script `build-portable.sh`:

```bash
#!/bin/bash

# Build for Windows
echo "Building for Windows..."
cargo build --release --target x86_64-pc-windows-gnu

# Create portable directory
echo "Creating portable package..."
mkdir -p portable-windows
cd portable-windows

# Copy executable
cp ../target/x86_64-pc-windows-gnu/release/Jobseeker.exe .

# Create default settings
cat > settings.json << 'EOF'
{
  "keywords": "",
  "blacklist_keywords": "",
  "locations_p1": "",
  "locations_p2": "",
  "locations_p3": "",
  "my_profile": "",
  "ollama_url": "http://localhost:11434/v1"
}
EOF

# Create README
cat > README.txt << 'EOF'
JOBSEEKER PORTABLE EDITION
==========================

This is a portable version of Jobseeker that runs without installation.

USAGE:
------
1. Double-click Jobseeker.exe to run
2. All data is stored in this folder:
   - jobseeker.db (your job database)
   - settings.json (your preferences)
3. To use on another computer, copy the entire folder

FIRST TIME SETUP:
-----------------
1. Run Jobseeker.exe
2. Go to Settings tab
3. Configure your keywords and locations
4. Click "Spara inställningar"
5. Start searching!

SYSTEM REQUIREMENTS:
--------------------
- Windows 10 or later (64-bit)
- No installation needed
- No admin rights required
- ~100 MB free space for database

TIPS:
-----
- Keep this folder on a USB drive for portability
- Back up jobseeker.db regularly (it contains all your job data)
- The app creates a database automatically on first run

LICENSE:
--------
See LICENSE file in source repository
EOF

# Create launcher batch file (optional)
cat > start.bat << 'EOF'
@echo off
echo Starting Jobseeker...
Jobseeker.exe
EOF

echo "Portable package created in portable-windows/"
echo "Total size:"
du -sh .

cd ..

# Optional: Create ZIP archive
echo "Creating ZIP archive..."
zip -r jobseeker-portable-windows.zip portable-windows/
echo "Done! Package: jobseeker-portable-windows.zip"
```

Make it executable and run:
```bash
chmod +x build-portable.sh
./build-portable.sh
```

### Manual Steps

```bash
# After building, create portable folder
mkdir portable-windows
cd portable-windows

# Copy the executable
cp ../target/x86_64-pc-windows-gnu/release/Jobseeker.exe .

# Create empty settings.json (app will populate on first run)
echo '{"keywords":"","blacklist_keywords":"","locations_p1":"","locations_p2":"","locations_p3":"","my_profile":"","ollama_url":"http://localhost:11434/v1"}' > settings.json

# Create README.txt (see above for content)

# Optional: Add icon (if you have one)
# cp ../assets/icon.ico .
```

## Optimizing Size

To reduce the executable size:

```bash
# Add to Cargo.toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = true            # Link-time optimization  
codegen-units = 1     # Better optimization
strip = true          # Strip symbols
panic = "abort"       # Smaller panic handler

# Build with these optimizations
cargo build --release --target x86_64-pc-windows-gnu

# Further compress with UPX (optional)
upx --best --lzma target/x86_64-pc-windows-gnu/release/Jobseeker.exe
```

**Note:** UPX compression may trigger false positives in some antivirus software.

## USB Drive Setup

### Recommended Structure

```
USB Drive (E:)
├── Jobseeker/
│   ├── Jobseeker.exe          (main executable)
│   ├── jobseeker.db           (created on first run)
│   ├── settings.json          (your settings)
│   ├── README.txt             (instructions)
│   └── start.bat              (optional launcher)
```

### USB Drive Recommendations

- **Minimum:** 256 MB (for app + small database)
- **Recommended:** 2 GB+ (for larger job database)
- **Speed:** USB 3.0 for faster database access
- **Format:** NTFS or exFAT (for large files)

## Testing the Portable Version

```bash
# On a Windows VM or test machine:
1. Copy the portable-windows folder to Desktop
2. Double-click Jobseeker.exe
3. Configure settings
4. Search for jobs
5. Close app
6. Move folder to different location
7. Run again - all data should persist
```

## Security Considerations

- **No admin rights needed** - Runs in user space
- **Data privacy** - All data stays on USB, nothing on host PC
- **Antivirus** - Some AV may flag Rust executables as unknown
  - Solution: Submit to VirusTotal, get whitelisted
  - Or: Code sign the executable (requires certificate)

## Distribution

### For Personal Use
Just copy the folder to USB and use!

### For Sharing
Create a ZIP archive:
```bash
zip -r jobseeker-portable-v0.1-windows.zip portable-windows/
```

### For Release
Consider:
- Code signing certificate (prevents "Unknown publisher" warnings)
- Proper versioning
- Changelog
- GPG signature for verification

## Troubleshooting

### "Windows protected your PC" message
**Cause:** Executable not digitally signed  
**Solution:** Click "More info" → "Run anyway"  
Or: Get a code signing certificate (~$100-500/year)

### "Missing VCRUNTIME140.dll"
**Cause:** Missing Visual C++ Runtime  
**Solution:** Use `x86_64-pc-windows-gnu` target instead of `msvc`, or include redistributables

### App doesn't save settings
**Cause:** USB drive is write-protected  
**Solution:** Remove write protection from USB drive

### Database locked errors
**Cause:** USB drive too slow or disconnected  
**Solution:** Copy folder to local drive first, use from there

### Slow performance
**Cause:** USB 2.0 drive with large database  
**Solution:** Use USB 3.0 drive or copy to local SSD temporarily

## Advanced: Auto-Update Script

Create `update.bat` for easy updates:

```batch
@echo off
echo Jobseeker Portable Update Script
echo ================================
echo.
echo This will download the latest version.
echo Your database and settings will be preserved.
echo.
pause

REM Backup current files
if exist jobseeker.db (
    echo Backing up database...
    copy jobseeker.db jobseeker.db.backup
)
if exist settings.json (
    echo Backing up settings...
    copy settings.json settings.json.backup
)

REM Download latest version (requires curl or wget)
echo Downloading latest version...
curl -L -o Jobseeker-new.exe https://github.com/Gnaw-Software/Jobseeker/releases/latest/download/Jobseeker.exe

REM Replace executable
if exist Jobseeker-new.exe (
    del Jobseeker.exe
    ren Jobseeker-new.exe Jobseeker.exe
    echo Update complete!
) else (
    echo Download failed!
)

echo.
pause
```

## Building for Different Windows Versions

```bash
# Windows 10/11 (64-bit) - Default
cargo build --release --target x86_64-pc-windows-gnu

# Windows 10/11 (ARM64) - For Surface ARM devices
rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc

# Windows 7/8 (32-bit) - Legacy support
rustup target add i686-pc-windows-gnu
cargo build --release --target i686-pc-windows-gnu
```

## File Size Reference

Typical sizes for portable package:
- **Executable:** ~30-40 MB (release, stripped)
- **With UPX:** ~15-20 MB (compressed)
- **Empty database:** ~20 KB
- **Settings:** ~0.5 KB
- **Database with 1000 jobs:** ~5-10 MB

**Total portable package:** ~40-50 MB (fits easily on smallest USB drives)

## Final Checklist

Before distributing:
- [ ] Build in release mode
- [ ] Strip executable
- [ ] Test on clean Windows install
- [ ] Verify no missing DLLs
- [ ] Include README.txt
- [ ] Test from USB drive
- [ ] Backup database works
- [ ] Settings persist
- [ ] No admin rights needed
- [ ] Virus scan clean

---

## Quick Build from Fedora

Since you're on Fedora, here's the fastest way:

```bash
# One-time setup (if you haven't already)
sudo dnf install -y mingw64-gcc mingw64-gcc-c++ mingw64-winpthreads-static zip
rustup target add x86_64-pc-windows-gnu

# Build portable Windows version
./build-portable-windows.sh

# Done! Package is ready in:
# - portable-windows/ (folder to copy to USB)
# - jobseeker-portable-windows.zip (for distribution)
```

### Known Issue: SQLite Cross-Compilation

Cross-compiling SQLite from Linux to Windows can sometimes fail due to compiler issues. If you encounter errors about `sqlite3_unlock_notify` or MinGW compiler crashes:

**Solution 1: Build on Windows**
- Use a Windows machine or VM
- Install Rust and Visual Studio Build Tools
- Run: `cargo build --release --target x86_64-pc-windows-msvc`

**Solution 2: Use GitHub Actions**
- The project's CI automatically builds Windows binaries
- Download from GitHub Actions artifacts or Releases page

**Solution 3: Use bundled SQLite (experimental)**
- Add to `.cargo/config.toml`:
```toml
[env]
LIBSQLITE3_SYS_BUNDLED = "1"
```
- This forces SQLite to be bundled, avoiding system library issues
- May increase compilation time significantly

For production builds, we recommend using the CI-built binaries or building natively on Windows.

**Now you can bring your job search anywhere!** 🚀

Perfect for:
- Job fairs
- Library computers  
- Friend's computer
- Work computer (lunch break job hunting 😉)
- Interviews (show your organizational skills)
- Démonstration på Windows-maskiner (när de undrar varför du använder Fedora)