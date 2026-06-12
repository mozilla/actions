#!/bin/bash
# Fetch the SHA256 checksum for an NSS or NSPR release tarball from the
# Mozilla FTP and update nss/nss-versions.env.
#
# Usage: update-sha256.sh <nss|nspr> <version>
# Called by Renovate postUpgradeTasks after bumping NSS_VERSION / NSPR_VERSION.
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <nss|nspr> <version>" >&2
    exit 1
fi

PKG="$1"
VERSION="$2"

case "$PKG" in
    nss)
        TAG="${VERSION//./_}"
        URL="https://ftp.mozilla.org/pub/security/nss/releases/NSS_${TAG}_RTM/src/SHA256SUMS"
        FILENAME="nss-${VERSION}.tar.gz"
        VAR="NSS_SHA256"
        ;;
    nspr)
        URL="https://ftp.mozilla.org/pub/nspr/releases/v${VERSION}/src/SHA256SUMS"
        FILENAME="nspr-${VERSION}.tar.gz"
        VAR="NSPR_SHA256"
        ;;
    *)
        echo "Unknown package: $PKG" >&2
        exit 1
        ;;
esac

SHA=$(curl -fsSL "$URL" | awk -v f="$FILENAME" '{sub(/^\*/, "", $2)} $2 == f {print $1}')

if ! [[ "$SHA" =~ ^[0-9a-f]{64}$ ]]; then
    echo "Invalid or missing SHA256 for $FILENAME from $URL${SHA:+: $SHA}" >&2
    exit 1
fi

sed -i "s|^${VAR}=.*|${VAR}=${SHA}|" nss/nss-versions.env
echo "${VAR}=${SHA}"
