#!/bin/bash
# Script to generate .SRCINFO files for AUR packages

set -e

echo "Generating .SRCINFO for polly..."
makepkg --printsrcinfo -p PKGBUILD-polly > PKGBUILD-polly.SRCINFO
echo "✓ Generated PKGBUILD-polly.SRCINFO"

echo "Generating .SRCINFO for polly-git..."
makepkg --printsrcinfo -p PKGBUILD-polly-git > PKGBUILD-polly-git.SRCINFO
echo "✓ Generated PKGBUILD-polly-git.SRCINFO"

echo ""
echo "Done! Copy the appropriate files to your AUR repository:"
echo "  For polly:     cp PKGBUILD-polly PKGBUILD && cp PKGBUILD-polly.SRCINFO .SRCINFO"
echo "  For polly-git: cp PKGBUILD-polly-git PKGBUILD && cp PKGBUILD-polly-git.SRCINFO .SRCINFO"
