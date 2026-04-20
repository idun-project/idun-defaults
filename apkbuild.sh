#!/bin/bash
set -e
# Extract pkgname/pkgver from APKBUILD
. ./APKBUILD
TARBALL="${pkgname}-${pkgver}.tar.gz"

echo ">>> Creating source tarball: $TARBALL"
git archive --format=tar.gz -o "$TARBALL" HEAD

echo ">>> Updating checksums"
abuild checksum

echo ">>> Building package"
abuild -r

echo ">>> Done"
