# Maintainer: Brian Holdsworth <brian@focus42llc.com>
pkgname=idun-defaults
pkgver=1.3
pkgrel=0
pkgdesc="Idun default configuration files"
url="https://github.com/idun-project/idun-defaults"
arch="aarch64"
license="GPL3"
depends="fd procps socat seatd cage rio syncterm yazi"
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
8c5eab9de7acac614033cc0454cd07cebf3850952955fead88f1fc9042ea0de0a7617a460e6500fefa8a8a4cf491a818fac659f30ce0d21ab2ff05520cc8b22a  idun-defaults-1.3.tar.gz
"
