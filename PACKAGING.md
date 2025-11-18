# Packaging Guide for LSMCP

This document provides instructions for package maintainers on building and distributing LSMCP.

## Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)
- Git

## Package Files

### Nix

**File**: `flake.nix`

The Nix flake provides a fully reproducible build:

```bash
# Build
nix build

# Run without installing
nix run

# Enter development shell
nix develop
```

The flake includes:
- `packages.default`: The main lsmcp package
- `apps.default`: Direct execution wrapper
- `devShells.default`: Development environment with Rust toolchain

### Fedora/RHEL (RPM)

**File**: `lsmcp.spec`

Build the RPM package:

```bash
# Create source tarball
git archive --format=tar.gz --prefix=lsmcp-0.1.0/ -o lsmcp-0.1.0.tar.gz HEAD

# Build RPM
rpmbuild -ba lsmcp.spec
```

### Arch Linux (AUR)

**File**: `PKGBUILD`

Build the package:

```bash
# Update checksums
updpkgsums

# Build
makepkg -si

# Test
makepkg -si --noconfirm
```

For AUR submission:
1. Create a git repository in AUR
2. Add PKGBUILD and .SRCINFO
3. Generate .SRCINFO: `makepkg --printsrcinfo > .SRCINFO`
4. Commit and push

### Debian/Ubuntu (DEB)

**Directory**: `debian/`

Build the package:

```bash
# Install build dependencies
sudo apt install debhelper cargo rustc

# Build
dpkg-buildpackage -us -uc -b

# The .deb file will be in the parent directory
```

## Version Updates

When releasing a new version:

1. Update `Cargo.toml` version
2. Update version in packaging files:
   - `flake.nix`: `version = "x.y.z"`
   - `lsmcp.spec`: `Version: x.y.z`
   - `PKGBUILD`: `pkgver=x.y.z`
   - `debian/changelog`: Add new entry with `dch -v x.y.z-1`

3. Update checksums:
   - PKGBUILD: Run `updpkgsums`
   - Others: Update after creating release tarball

4. Tag the release:
   ```bash
   git tag -a vx.y.z -m "Release version x.y.z"
   git push origin vx.y.z
   ```

## Distribution

### Nix

The flake can be used directly from GitHub:
```nix
{
  inputs.lsmcp.url = "github:YZTangent/lsmcp";
}
```

### Copr (Fedora)

1. Create account at https://copr.fedorainfracloud.org/
2. Create new project "lsmcp"
3. Upload source RPM or configure auto-rebuild from git

### AUR

1. Create an account on https://aur.archlinux.org/
2. Clone the lsmcp AUR repository
3. Update PKGBUILD and .SRCINFO
4. Push changes

### PPA (Ubuntu)

1. Create Launchpad account
2. Set up PPA: https://launchpad.net/~username/+archive/ubuntu/ppa
3. Upload source package:
   ```bash
   debuild -S
   dput ppa:username/ppa ../lsmcp_0.1.0-1_source.changes
   ```

## Testing

Before releasing packages:

1. **Build test**: Ensure package builds successfully
   ```bash
   cargo build --release --locked
   cargo test --release
   ```

2. **Installation test**: Install package and verify binary works
   ```bash
   lsmcp --version
   lsmcp --help
   ```

3. **Functional test**: Test with a real project
   ```bash
   lsmcp --workspace /path/to/test-project
   ```

## Checksums and Signatures

For official releases:

1. Generate checksums:
   ```bash
   sha256sum lsmcp-x.y.z.tar.gz > lsmcp-x.y.z.tar.gz.sha256
   ```

2. Sign the release (optional but recommended):
   ```bash
   gpg --detach-sign --armor lsmcp-x.y.z.tar.gz
   ```

## Contact

For packaging questions or issues:
- Open an issue: https://github.com/YZTangent/lsmcp/issues
- Discussion: https://github.com/YZTangent/lsmcp/discussions
