# Summary of Changes

This document summarizes the changes made to add AUR packaging and release workflow.

## New Files Added

### AUR Packaging Files
1. **PKGBUILD-polly** - PKGBUILD for stable releases
   - Downloads source from GitHub release tags
   - Builds and installs to `/usr/bin/polly`
   - Includes systemd service installation
   
2. **PKGBUILD-polly-git** - PKGBUILD for git development version
   - Clones from git repository
   - Tracks latest commits
   - Provides/conflicts with 'polly'

3. **polly.install** - Post-install script
   - Displays helpful message about enabling the service
   - Shows both systemd and manual startup options

4. **generate-srcinfo.sh** - Helper script
   - Generates .SRCINFO files for both packages
   - Makes AUR submission easier

### Release Automation
5. **.github/workflows/release.yml** - GitHub Actions workflow
   - Triggered by version tags (v*)
   - Builds for three architectures:
     - x86_64-unknown-linux-gnu (standard glibc)
     - x86_64-unknown-linux-musl (static musl)
     - aarch64-unknown-linux-gnu (ARM64)
   - Creates GitHub release with binaries
   - Uses cross-compilation for musl and ARM targets

6. **Cross.toml** - Cross-compilation configuration
   - Specifies build dependencies for cross targets
   - Ensures polkit and glib are available during build

### System Integration
7. **polly.service** - systemd user service
   - Enables auto-start with graphical session
   - Includes restart on failure
   - Part of graphical-session.target

### Documentation
8. **AUR_INSTALL.md** - Installation guide
   - Instructions for installing from AUR
   - How to enable/use the service
   - Guide for AUR maintainers

9. **RELEASE.md** - Release process documentation
   - Step-by-step release instructions
   - How to update AUR packages
   - Testing checklist

10. **PACKAGING.md** - Packaging overview
    - Describes all packaging files
    - Documents build targets
    - Installation methods
    - Maintenance procedures

## Modified Files

1. **Readme.md**
   - Added installation section
   - Links to AUR packages
   - Mentions pre-built binaries
   - Links to documentation

2. **.gitignore**
   - Added AUR build artifacts (*.tar.gz, *.pkg.tar.zst)
   - Added build directories (pkg/, src/)
   - Added editor files (.vscode/, .idea/, *.swp)
   - Added OS files (.DS_Store, Thumbs.db)

## How to Use

### For Users (Installing)

**Arch Linux:**
```bash
yay -S polly        # Stable
yay -S polly-git    # Development
```

**Other distributions:**
- Download binary from GitHub releases
- Build from source with cargo

### For Maintainers (Releasing)

1. Update version in Cargo.toml and PKGBUILD-polly
2. Commit and push changes
3. Create and push tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
4. GitHub Actions automatically builds and releases
5. Update AUR package with new checksums

### For AUR Maintainers (Submitting)

```bash
# Clone AUR repo
git clone ssh://aur@aur.archlinux.org/polly.git

# Copy PKGBUILD
cp PKGBUILD-polly polly/PKGBUILD

# Generate .SRCINFO
cd polly
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Initial upload: polly 0.1.0"
git push
```

## Architecture Support

Official binaries are built for:
- x86_64 (Intel/AMD 64-bit) - glibc and musl variants
- aarch64 (ARM 64-bit)

## Dependencies

**Runtime:**
- glib2
- polkit

**Build:**
- rust
- cargo
- libglib2.0-dev
- libpolkit-gobject-1-dev
- libpolkit-agent-1-dev
