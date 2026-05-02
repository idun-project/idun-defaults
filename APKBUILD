# Maintainer: Brian Holdsworth <brian@focus42llc.com>
pkgname=idun-defaults
pkgver=1.0
pkgrel=2
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
6c650b7f2245acda15bcc7586ed1a06c8dbf41d09dbb9e493c548dbe01f3177892965a8a7bd28b44f6ad06fcceb6d468eb8f546f0df432d00f01dc38a8a02b1a  idun-defaults-1.0.tar.gz
"
