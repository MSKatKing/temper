#!/usr/bin/env bash
set -euo pipefail

package_id="$(cargo pkgid -p temper)"
version="${package_id##*@}"

if [[ -z "${version}" || "${version}" == "null" ]]; then
    echo "Failed to read the temper package version from cargo metadata." >&2
    exit 1
fi

tag="v${version}"

if git rev-parse --verify --quiet "refs/tags/${tag}" >/dev/null; then
    echo "Release tag ${tag} already exists. Bump Cargo.toml before releasing." >&2
    exit 1
fi

latest_tag="$(git tag --list 'v*' --sort=-v:refname | head -n 1)"

if [[ -n "${latest_tag}" && "${latest_tag}" == "${tag}" ]]; then
    echo "Cargo version ${version} matches the latest release ${latest_tag}." >&2
    exit 1
fi

echo "${version}"
