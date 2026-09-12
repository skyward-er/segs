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
    printf 'Usage: %s install [binary-path]\n' "$(basename -- "$0")"
    printf '       %s uninstall\n\n' "$(basename -- "$0")"
    printf 'Commands:\n'
    printf '  install    Install the SEGS 2 desktop entry and icon\n'
    printf '  uninstall  Remove the SEGS 2 desktop entry and icon\n'
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

# Escape an executable path for the desktop entry Exec field
escape_exec_path() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value//\"/\\\"}"
    value="${value//\$/\\\$}"
    value="${value//\`/\\\`}"
    printf '"%s"' "${value}"
}

# Escape an executable path for a desktop entry string field
escape_string() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value//$'\n'/\\n}"
    value="${value//$'\r'/\\r}"
    value="${value//$'\t'/\\t}"
    printf '%s' "${value}"
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
    local binary_path
    binary_path="$(resolve_binary "${1:-}")"
    local escaped_binary_path
    escaped_binary_path="$(escape_exec_path "${binary_path}")"
    local escaped_try_exec
    escaped_try_exec="$(escape_string "${binary_path}")"

    # Write a desktop entry with the resolved executable path
    mkdir -p "${APPLICATIONS_DIR}"
    while IFS= read -r line; do
        case "${line}" in
            Exec=*)
                printf 'Exec=%s\n' "${escaped_binary_path}"
                ;;
            TryExec=*)
                printf 'TryExec=%s\n' "${escaped_try_exec}"
                ;;
            *)
                printf '%s\n' "${line}"
                ;;
        esac
    done <"${DESKTOP_SOURCE}" >"${APPLICATIONS_DIR}/${APP_ID}.desktop"
    chmod 644 "${APPLICATIONS_DIR}/${APP_ID}.desktop"

    # Install the themed icon and refresh desktop caches
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
