# Maintainer: Brian Holdsworth <brian@focus42llc.com>
# Define the kiosk apps and their dependencies
_base_deps="cage
syncterm
"
_xterm_deps="alacritty"
_thor_deps="yazi"

pkgname=idun-defaults
pkgver=1.3
pkgrel=0
pkgdesc="Idun default configuration files"
url="https://github.com/idun-project/idun-defaults"
arch="aarch64"
license="GPL3"
depends="fd
fzf
procps
socat
$_base_deps
$_xterm_deps
$_thor_deps
"
source="$pkgname-$pkgver.tar.gz"
builddir="$srcdir"
options="!check"
install="$pkgname.post-install $pkgname.post-upgrade"

build() {
	export RUSTFLAGS="-C target-feature=-crt-static -C link-arg=-dynamic-linker=/lib/ld-musl-aarch64.so.1"
	cd "$builddir"
	
	cd ffetch
	cargo build --release
	cd ..
	
	cd idunsh
	cargo build --release
}

package() {
	# Binaries
	install -Dm755 "$builddir/target/release/ffetch" \
		"$pkgdir/usr/bin/ffetch"

	install -Dm755 "$builddir/target/release/idunsh" \
		"$pkgdir/usr/bin/idunsh"
	# Kiosk scripts/assets
	install -Dm755 "$builddir/kiosk" \
		"$pkgdir/usr/bin/kiosk"
	# Config files
	install -Dm644 "$builddir/idunrc.toml" \
		"$pkgdir/usr/share/idun/idunrc.toml"

	install -Dm644 "$builddir/bashrc" \
		"$pkgdir/usr/share/idun/bashrc"

	install -Dm644 "$builddir/newshell" \
		"$pkgdir/usr/share/idun/newshell"

	install -Dm644 "$builddir/Idun_c64u_run_first.cfg" \
		"$pkgdir/usr/share/idun/Idun_c64u_run_first.cfg"

	install -Dm644 "$builddir/kiosk.lst" \
		"$pkgdir/usr/share/idun/kiosk.lst"
}
sha512sums="
b46c71af8c811d11b61dda7b0ee1cd93a7109292c9648ee9174e4a67c3f44fd66e5b769fb05cc88dedbac31fd96d0ca5031cbd3c6d6afbcfcd5534ba46eea935  idun-defaults-1.3.tar.gz
"
