pkgname=idun-defaults
pkgver=1.0
pkgrel=1
pkgdesc="Idun default configuration files"
arch=("aarch64" "armv7h")
url="https://github.com/idun-project/idun-defaults"
depends=(fd)
makedepends=(rustup)
provides=(idun-defaults)
source=(ffetch)
install="config.install"

build() {
  cd ../ffetch
  cargo zigbuild --release --target arm-unknown-linux-gnueabihf
}

package() {
  install -D -m 755 ../ffetch/target/arm-unknown-linux-gnueabihf/release/ffetch "${pkgdir}"/usr/bin/ffetch
  install -D -m 644 ../idunrc.toml "${pkgdir}"/etc/xdg/idun/idunrc.toml
  install -m 644 ../bashrc "${pkgdir}"/etc/xdg/idun/bashrc
  install -m 644 ../newshell "${pkgdir}"/etc/xdg/idun/newshell
}
