# Maintainer: Brian Holdsworth <brian@focus42llc.com>
# Define the kiosk apps and their dependencies
_base_deps="cage
syncterm
opencubicplayer"
_xterm_deps="alacritty
font-jetbrains-mono-nerd"
_thor_deps="yazi
yazi-cli
bat
mediainfo
ueberzugpp
poppler-utils
zathura
zathura-pdf-poppler
7zip"

pkgname=idun-defaults
pkgver=1.4
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
	# rootfs resizer
	install -Dm755 "$builddir/resize-fs" \
		"$pkgdir/usr/bin/resize-fs"

	install -Dm755 "$builddir/resizesd.rc" \
		"$pkgdir/etc/init.d/resizesd"
	# Kiosk
	install -Dm644 "$builddir/kiosk.lst" \
		"$pkgdir/usr/share/idun/kiosk.lst"

	install -Dm644 "$builddir/kiosk_conf.tar.zst" \
		"$pkgdir/usr/share/idun/kiosk_conf.tar.zst"
}
sha512sums="
5bb2ab5e46b5b24026a1995fab070f2e7993a6e3a58012c0afdcee3eb9a3cdbd916a939b4968f77966eb35a0cbd9f4144677868622f47a8e674a2335b6ec5988  idun-defaults-1.4.tar.gz
"
