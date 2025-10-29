# AUR Installation Guide

This document provides instructions for installing polly from the Arch User Repository (AUR).

## Available Packages

### polly
The stable release version of polly, tracking tagged releases.

```bash
# Using an AUR helper (e.g., yay, paru)
yay -S polly

# Or manually
git clone https://aur.archlinux.org/polly.git
cd polly
makepkg -si
```

### polly-git
The development version of polly, tracking the latest git commits.

```bash
# Using an AUR helper (e.g., yay, paru)
yay -S polly-git

# Or manually
git clone https://aur.archlinux.org/polly-git.git
cd polly-git
makepkg -si
```

## Manual Installation from PKGBUILD

If you want to build directly from this repository:

```bash
# For stable version
makepkg -p PKGBUILD-polly -si

# For git version
makepkg -p PKGBUILD-polly-git -si
```

## Dependencies

### Runtime Dependencies
- glib2
- polkit

### Build Dependencies
- rust
- cargo
- git (for polly-git)

## Running polly

After installation, you can run polly as your Polkit authentication agent.

### Option 1: Using systemd (recommended)

Enable and start the systemd user service:

```bash
systemctl --user enable --now polly.service
```

To disable autostart:
```bash
systemctl --user disable --now polly.service
```

### Option 2: Manual autostart

Add the following to your compositor's autostart configuration:

```bash
polly &
```

**Hyprland** (`~/.config/hypr/hyprland.conf`):
```
exec-once = polly
```

**Sway** (`~/.config/sway/config`):
```
exec polly
```

**River** (`~/.config/river/init`):
```
polly &
```

## Submitting to AUR

For maintainers wanting to submit these packages to AUR:

1. Create the AUR repository:
```bash
# For stable version
git clone ssh://aur@aur.archlinux.org/polly.git
cd polly
cp /path/to/polkit-agent/PKGBUILD-polly PKGBUILD

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Initial commit: polly v0.1.0"
git push
```

2. For polly-git, repeat the above steps with PKGBUILD-polly-git.

## Updating .SRCINFO

When updating the PKGBUILD, always regenerate .SRCINFO:

```bash
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to version X.Y.Z"
git push
```
