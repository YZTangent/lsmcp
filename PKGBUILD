# Maintainer: LSMCP Contributors <lsmcp@example.com>
pkgname=lsmcp
pkgver=0.1.0
pkgrel=1
pkgdesc="Language Server Manager for Model Context Protocol"
arch=('x86_64' 'aarch64')
url="https://github.com/YZTangent/lsmcp"
license=('MIT' 'Apache-2.0')
depends=()
makedepends=('rust' 'cargo')
optdepends=(
    'nodejs-lts-iron: for TypeScript/JavaScript LSP auto-installation'
    'python: for Python LSP auto-installation'
    'go: for Go LSP auto-installation'
)
source=("$pkgname-$pkgver.tar.gz::https://github.com/YZTangent/lsmcp/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked --all-features
}

check() {
    cd "$pkgname-$pkgver"
    cargo test --release --locked
}

package() {
    cd "$pkgname-$pkgver"

    # Install binary
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"

    # Install licenses
    install -Dm644 LICENSE-MIT "$pkgdir/usr/share/licenses/$pkgname/LICENSE-MIT"
    install -Dm644 LICENSE-APACHE "$pkgdir/usr/share/licenses/$pkgname/LICENSE-APACHE"

    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 docs/ARCHITECTURE.md "$pkgdir/usr/share/doc/$pkgname/ARCHITECTURE.md"
}
