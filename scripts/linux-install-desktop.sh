#!/usr/bin/env bash

set -euo pipefail

readonly APP_ID="eu.skywarder.segs2"
readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
readonly DATA_ROOT="${XDG_DATA_HOME:-${HOME}/.local/share}"
readonly APPLICATIONS_DIR="${DATA_ROOT}/applications"
readonly ICON_THEME_DIR="${DATA_ROOT}/icons/hicolor"
readonly DESKTOP_SOURCE="${PROJECT_ROOT}/crates/segs/assets/linux/${APP_ID}.desktop"
readonly ICON_SOURCE="${PROJECT_ROOT}/crates/segs/assets/linux/${APP_ID}.png"

# Print command usage
usage() {
    printf 'Usage: %s <command>\n\n' "$(basename -- "$0")"
    printf 'Commands:\n'
    printf '  install    Install the SEGS 2 desktop entry and icon\n'
    printf '  uninstall  Remove the SEGS 2 desktop entry and icon\n'
}

# Refresh desktop integration caches when their tools are available
refresh_caches() {
    if [[ -d "${APPLICATIONS_DIR}" ]] && command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "${APPLICATIONS_DIR}"
    fi

    if [[ -d "${ICON_THEME_DIR}" ]] && command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache --force --ignore-theme-index "${ICON_THEME_DIR}"
    fi
}

# Install the per-user desktop entry and icon
install_desktop() {
    install -Dm644 "${DESKTOP_SOURCE}" "${APPLICATIONS_DIR}/${APP_ID}.desktop"
    install -Dm644 "${ICON_SOURCE}" "${ICON_THEME_DIR}/512x512/apps/${APP_ID}.png"

    refresh_caches
    printf 'Installed SEGS 2 desktop integration for the current user\n'
}

# Remove only the per-user desktop entry and icon
uninstall_desktop() {
    rm -f "${APPLICATIONS_DIR}/${APP_ID}.desktop"
    rm -f "${ICON_THEME_DIR}/512x512/apps/${APP_ID}.png"
    rm -f "${ICON_THEME_DIR}/1024x1024/apps/${APP_ID}.png"

    refresh_caches
    printf 'Uninstalled SEGS 2 desktop integration for the current user\n'
}

# Dispatch the requested desktop integration operation
if [[ $# -ne 1 ]]; then
    usage
    exit 2
fi

case "$1" in
    install)
        install_desktop
        ;;
    uninstall)
        uninstall_desktop
        ;;
    -h | --help | help)
        usage
        ;;
    *)
        usage >&2
        exit 2
        ;;
esac
