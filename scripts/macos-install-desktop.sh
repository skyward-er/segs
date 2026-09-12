#!/usr/bin/env bash

set -euo pipefail

readonly APP_ID="eu.skywarder.segs2"
readonly APP_NAME="SEGS 2"
readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
readonly APPLICATIONS_DIR="${HOME}/Applications"
readonly APP_DIR="${APPLICATIONS_DIR}/${APP_NAME}.app"
readonly INSTALL_MARKER="Contents/.${APP_ID}-desktop-integration"
readonly PLIST_SOURCE="${PROJECT_ROOT}/crates/segs/assets/macos/Info.plist"
readonly ICON_SOURCE="${PROJECT_ROOT}/crates/segs-assets/assets/icons/SEGS-1024x1024.png"

# Print command usage
usage() {
    printf 'Usage: %s install [binary-path]\n' "$(basename -- "$0")"
    printf '       %s uninstall\n\n' "$(basename -- "$0")"
    printf 'Commands:\n'
    printf '  install    Install the SEGS 2 application wrapper\n'
    printf '  uninstall  Remove the SEGS 2 application wrapper\n'
    printf '\nIf binary-path is omitted, the script searches PATH and Cargo directories.\n'
}

# Resolve the installed binary from an override or common Cargo locations
resolve_binary() {
    local requested_path="${1:-}"
    local discovered_path=""

    if [[ -n "${requested_path}" ]]; then
        discovered_path="${requested_path}"
    elif discovered_path="$(command -v segs2 2>/dev/null)"; then
        :
    elif [[ -n "${CARGO_INSTALL_ROOT:-}" && -x "${CARGO_INSTALL_ROOT}/bin/segs2" ]]; then
        discovered_path="${CARGO_INSTALL_ROOT}/bin/segs2"
    elif [[ -n "${CARGO_HOME:-}" && -x "${CARGO_HOME}/bin/segs2" ]]; then
        discovered_path="${CARGO_HOME}/bin/segs2"
    elif [[ -x "${HOME}/.cargo/bin/segs2" ]]; then
        discovered_path="${HOME}/.cargo/bin/segs2"
    fi

    if [[ -z "${discovered_path}" || ! -f "${discovered_path}" || ! -x "${discovered_path}" ]]; then
        printf 'Could not find an executable segs2 binary\n' >&2
        printf 'Install it with Cargo or pass its path to the install command\n' >&2
        return 1
    fi

    if [[ "$(basename -- "${discovered_path}")" != "segs2" ]]; then
        printf 'Expected a binary named segs2, got: %s\n' "${discovered_path}" >&2
        return 1
    fi

    local binary_dir
    binary_dir="$(cd -- "$(dirname -- "${discovered_path}")" && pwd -P)"
    printf '%s/%s\n' "${binary_dir}" "$(basename -- "${discovered_path}")"
}

# Generate the application icon with standard macOS icon sizes
generate_icon() {
    local iconset_dir="$1"
    local output_path="$2"

    mkdir -p "${iconset_dir}"
    sips -z 16 16 "${ICON_SOURCE}" --out "${iconset_dir}/icon_16x16.png" >/dev/null
    sips -z 32 32 "${ICON_SOURCE}" --out "${iconset_dir}/icon_16x16@2x.png" >/dev/null
    sips -z 32 32 "${ICON_SOURCE}" --out "${iconset_dir}/icon_32x32.png" >/dev/null
    sips -z 64 64 "${ICON_SOURCE}" --out "${iconset_dir}/icon_32x32@2x.png" >/dev/null
    sips -z 128 128 "${ICON_SOURCE}" --out "${iconset_dir}/icon_128x128.png" >/dev/null
    sips -z 256 256 "${ICON_SOURCE}" --out "${iconset_dir}/icon_128x128@2x.png" >/dev/null
    sips -z 256 256 "${ICON_SOURCE}" --out "${iconset_dir}/icon_256x256.png" >/dev/null
    sips -z 512 512 "${ICON_SOURCE}" --out "${iconset_dir}/icon_256x256@2x.png" >/dev/null
    sips -z 512 512 "${ICON_SOURCE}" --out "${iconset_dir}/icon_512x512.png" >/dev/null
    cp "${ICON_SOURCE}" "${iconset_dir}/icon_512x512@2x.png"
    iconutil -c icns "${iconset_dir}" -o "${output_path}"
}

# Install the per-user application wrapper
install_desktop() {
    if [[ "$(uname -s)" != "Darwin" ]]; then
        printf 'This script can only install desktop integration on macOS\n' >&2
        return 1
    fi

    local binary_path
    binary_path="$(resolve_binary "${1:-}")"

    # Protect an unrelated application with the same display name
    if [[ -e "${APP_DIR}" && ! -f "${APP_DIR}/${INSTALL_MARKER}" ]]; then
        printf 'Refusing to replace an application not owned by SEGS 2: %s\n' "${APP_DIR}" >&2
        return 1
    fi

    # Assemble the wrapper and icon in a temporary directory
    mkdir -p "${APPLICATIONS_DIR}"
    local temporary_dir
    temporary_dir="$(mktemp -d "${APPLICATIONS_DIR}/.segs2.XXXXXX")"
    local temporary_app="${temporary_dir}/${APP_NAME}.app"
    local iconset_dir="${temporary_dir}/segs2.iconset"
    mkdir -p "${temporary_app}/Contents/MacOS" "${temporary_app}/Contents/Resources"
    cp "${PLIST_SOURCE}" "${temporary_app}/Contents/Info.plist"
    ln -s "${binary_path}" "${temporary_app}/Contents/MacOS/segs2"
    generate_icon "${iconset_dir}" "${temporary_app}/Contents/Resources/segs2.icns"
    touch "${temporary_app}/${INSTALL_MARKER}"

    # Replace an earlier SEGS 2 wrapper only after assembly succeeds
    rm -rf "${APP_DIR}"
    mv "${temporary_app}" "${APP_DIR}"
    rm -rf "${temporary_dir}"
    touch "${APP_DIR}"
    printf 'Installed SEGS 2 application wrapper for the current user\n'
}

# Remove only the application wrapper owned by SEGS 2
uninstall_desktop() {
    if [[ ! -e "${APP_DIR}" ]]; then
        printf 'SEGS 2 application wrapper is not installed\n'
        return
    fi

    if [[ ! -f "${APP_DIR}/${INSTALL_MARKER}" ]]; then
        printf 'Refusing to remove an application not owned by SEGS 2: %s\n' "${APP_DIR}" >&2
        return 1
    fi

    rm -rf "${APP_DIR}"
    printf 'Uninstalled SEGS 2 application wrapper for the current user\n'
}

# Dispatch the requested desktop integration operation
if [[ $# -lt 1 ]]; then
    usage
    exit 2
fi

case "$1" in
    install)
        if [[ $# -gt 2 ]]; then
            usage >&2
            exit 2
        fi
        install_desktop "${2:-}"
        ;;
    uninstall)
        if [[ $# -ne 1 ]]; then
            usage >&2
            exit 2
        fi
        uninstall_desktop
        ;;
    -h | --help | help)
        if [[ $# -ne 1 ]]; then
            usage >&2
            exit 2
        fi
        usage
        ;;
    *)
        usage >&2
        exit 2
        ;;
esac
