# Maintainer: Brian Holdsworth <brian@focus42llc.com>
pkgname=idun-defaults
pkgver=1.0
pkgrel=1
pkgdesc="Idun default configuration files"
url="https://github.com/idun-project/idun-defaults"
arch="aarch64"
license="GPL3"
depends="fd procps socat"
source="$pkgname-$pkgver.tar.gz"
builddir="$srcdir"
options="!check"
install="$pkgname.post-install $pkgname.post-upgrade"

build() {
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

	install -Dm755 "$builddir/wifi" \
		"$pkgdir/usr/bin/wifi"

	# Config files
	install -Dm644 "$builddir/idunrc.toml" \
		"$pkgdir/usr/share/idun/idunrc.toml"

	install -Dm644 "$builddir/bashrc" \
		"$pkgdir/usr/share/idun/bashrc"

	install -Dm644 "$builddir/newshell" \
		"$pkgdir/usr/share/idun/newshell"

	install -Dm644 "$builddir/Idun_c64u_run_first.cfg" \
		"$pkgdir/usr/share/idun/Idun_c64u_run_first.cfg"
}
sha512sums="
be93bf181e2eacfd22e62b2bf66c359d7c0c3918fa8e4a78577f319afcd8ba796bbda8a81e58446fdefb5d82ab2dd7421b76ad629cfa41a254e0e0c33f675d8d  idun-defaults-1.0.tar.gz
"
