# Maintainer: Brian Holdsworth <brian@focus42llc.com>
pkgname=idun-defaults
pkgver=1.0
pkgrel=0
pkgdesc="Idun default configuration files"
url="https://github.com/idun-project/idun-defaults"
arch="aarch64"
license="GPL3"
depends="fd procps"
source="$pkgname-$pkgver.tar.gz"
builddir="$srcdir"
options="!check"
install="$pkgname.post_install $pkgname.post_upgrade"

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
e58bcbaa16f2b09b23aaf5c85be41648fa080aa43b7c1b9cbea23b712e443f4452e2e3e1e7cbeb6e332c4ba2673d703ce09e8dbe1fe7dea3864e298c65a2bb9e  idun-defaults-1.0.tar.gz
c3944d35bf76daeb6422b066d872df921a67b7cc01210054d1e63eaded24eac2a7cf7e087fd675ad3f657d6ce68a05d9c721ccef9b2730c0ed869f3b5f418173  idun-defaults.install
"
