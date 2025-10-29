# Release Process

This document outlines the process for creating a new release of polly.

## Creating a Release

### 1. Update Version

Update the version in `Cargo.toml`:

```toml
[package]
version = "X.Y.Z"
```

Update the version in `PKGBUILD-polly`:

```bash
pkgver=X.Y.Z
```

### 2. Update Changelog

Create or update CHANGELOG.md with the changes in this release.

### 3. Commit Changes

```bash
git add Cargo.toml PKGBUILD-polly CHANGELOG.md
git commit -m "Bump version to vX.Y.Z"
git push
```

### 4. Create and Push Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

### 5. Automated Build

The GitHub Actions workflow will automatically:
- Build binaries for x86_64-unknown-linux-gnu
- Build binaries for x86_64-unknown-linux-musl
- Build binaries for aarch64-unknown-linux-gnu
- Create a GitHub release with all binaries attached

### 6. Update AUR Package

After the release is created:

1. Update `PKGBUILD-polly` with the new checksum:

```bash
# Download the release tarball
wget https://github.com/manthanabc/polkit-agent/archive/refs/tags/vX.Y.Z.tar.gz

# Generate checksum
sha256sum vX.Y.Z.tar.gz
```

2. Update the `sha256sums` in `PKGBUILD-polly`

3. Generate new .SRCINFO:

```bash
makepkg --printsrcinfo -p PKGBUILD-polly > .SRCINFO
```

4. Update AUR repository:

```bash
cd /path/to/aur/polly
cp /path/to/polkit-agent/PKGBUILD-polly PKGBUILD
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to vX.Y.Z"
git push
```

## Manual Build

To manually build release binaries without creating a tag:

```bash
# Build for current platform
cargo build --release

# Build for specific target
cargo build --release --target x86_64-unknown-linux-gnu

# Build with cross for other platforms
cross build --release --target aarch64-unknown-linux-gnu
```

## Testing Before Release

Before creating a release, ensure:

1. All tests pass:
```bash
cargo test
```

2. The binary builds successfully:
```bash
cargo build --release
```

3. The PKGBUILD works:
```bash
makepkg -p PKGBUILD-polly
```

4. The systemd service is valid:
```bash
systemd-analyze verify polly.service
```
