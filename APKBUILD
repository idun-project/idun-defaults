# Maintainer: Brian Holdsworth <brian@focus42llc.com>
pkgname=idun-defaults
pkgver=1.0
pkgrel=0
pkgdesc="Idun default configuration files"
url="https://github.com/idun-project/idun-defaults"
arch="aarch64"
license="GPL3"
depends="fd procps"
source="
$pkgname-$pkgver.tar.gz
$pkgname.post-install
"
builddir="$srcdir"
options="!check"
install="$pkgname.post-install"

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
2c00b6fd46a52e3834ed4c492ba2816ddcf5e9fd865c697a5db22674846ba33e373e80f52401c18575f94e70626a66b9f1e9a8174247f9f1b892b31689d79ecc  idun-defaults-1.0.tar.gz
"
