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
install="$pkgname.install"

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
0dcf9f03a4f373fe62c9858eaecb4a6bf508e1ff09f84a15454d57520937f0708ced4e3adc19a955d0473df011331d52278f0c63fb07cce1c927d959106e1567  idun-defaults-1.0.tar.gz
c3944d35bf76daeb6422b066d872df921a67b7cc01210054d1e63eaded24eac2a7cf7e087fd675ad3f657d6ce68a05d9c721ccef9b2730c0ed869f3b5f418173  idun-defaults.post-install
"
