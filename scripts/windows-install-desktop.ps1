[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string] $Command,

    [Parameter()]
    [string] $BinaryPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$AppName = 'SEGS 2'
$BinaryName = 'segs2.exe'
$ShortcutName = 'SEGS 2.lnk'

# Print command usage
function Show-Usage {
    Write-Output 'Usage: windows-install-desktop.ps1 install [-BinaryPath <path>]'
    Write-Output '       windows-install-desktop.ps1 uninstall'
    Write-Output ''
    Write-Output 'Commands:'
    Write-Output '  install    Install the SEGS 2 Start Menu shortcut'
    Write-Output '  uninstall  Remove the SEGS 2 Start Menu shortcut'
    Write-Output ''
    Write-Output 'If BinaryPath is omitted, the script searches PATH and Cargo directories.'
}

# Resolve the installed binary from an override or common Cargo locations
function Resolve-SegsBinary {
    param(
        [string] $RequestedPath
    )

    $DiscoveredPath = $null
    if ($RequestedPath) {
        $DiscoveredPath = $RequestedPath
    } else {
        $CommandInfo = Get-Command $BinaryName -CommandType Application -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($CommandInfo) {
            $DiscoveredPath = $CommandInfo.Source
        }
    }

    # Search Cargo installation roots when the binary is not on PATH
    if (-not $DiscoveredPath) {
        $Candidates = @()
        if ($env:CARGO_INSTALL_ROOT) {
            $Candidates += Join-Path $env:CARGO_INSTALL_ROOT "bin\$BinaryName"
        }
        if ($env:CARGO_HOME) {
            $Candidates += Join-Path $env:CARGO_HOME "bin\$BinaryName"
        }
        $Candidates += Join-Path $HOME ".cargo\bin\$BinaryName"
        $DiscoveredPath = $Candidates |
            Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
            Select-Object -First 1
    }

    if (-not $DiscoveredPath -or -not (Test-Path -LiteralPath $DiscoveredPath -PathType Leaf)) {
        throw 'Could not find segs2.exe. Install it with Cargo or pass -BinaryPath.'
    }

    $ResolvedPath = (Resolve-Path -LiteralPath $DiscoveredPath).Path
    if ([System.IO.Path]::GetFileName($ResolvedPath) -ine $BinaryName) {
        throw "Expected a binary named $BinaryName, got: $ResolvedPath"
    }

    return $ResolvedPath
}

# Open a shortcut through the Windows Script Host COM API
function Open-Shortcut {
    param(
        [string] $Path
    )

    $Shell = New-Object -ComObject WScript.Shell
    return @($Shell, $Shell.CreateShortcut($Path))
}

# Release shortcut COM objects created by the script
function Close-Shortcut {
    param(
        [object[]] $Objects
    )

    foreach ($Object in $Objects) {
        if ($null -ne $Object -and [System.Runtime.InteropServices.Marshal]::IsComObject($Object)) {
            [void] [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
        }
    }
}

# Install the per-user Start Menu shortcut
function Install-Desktop {
    param(
        [string] $RequestedPath
    )

    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        throw 'This script can only install desktop integration on Windows.'
    }

    $ResolvedBinary = Resolve-SegsBinary $RequestedPath
    $ProgramsDirectory = [Environment]::GetFolderPath([Environment+SpecialFolder]::Programs)
    $ShortcutPath = Join-Path $ProgramsDirectory $ShortcutName

    # Protect an unrelated shortcut with the same display name
    if (Test-Path -LiteralPath $ShortcutPath -PathType Leaf) {
        $ExistingObjects = Open-Shortcut $ShortcutPath
        try {
            $ExistingTarget = $ExistingObjects[1].TargetPath
            if ([System.IO.Path]::GetFileName($ExistingTarget) -ine $BinaryName) {
                throw "Refusing to replace a shortcut not owned by $AppName`: $ShortcutPath"
            }
        } finally {
            Close-Shortcut $ExistingObjects
        }
    }

    # Create the shortcut with the icon embedded in the executable
    $ShortcutObjects = Open-Shortcut $ShortcutPath
    try {
        $Shortcut = $ShortcutObjects[1]
        $Shortcut.TargetPath = $ResolvedBinary
        $Shortcut.WorkingDirectory = [System.IO.Path]::GetDirectoryName($ResolvedBinary)
        $Shortcut.IconLocation = "$ResolvedBinary,0"
        $Shortcut.Description = 'Skyward Enhanced Ground Software'
        $Shortcut.Save()
    } finally {
        Close-Shortcut $ShortcutObjects
    }

    Write-Output 'Installed SEGS 2 Start Menu shortcut for the current user'
}

# Remove only the Start Menu shortcut owned by SEGS 2
function Uninstall-Desktop {
    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        throw 'This script can only uninstall desktop integration on Windows.'
    }

    $ProgramsDirectory = [Environment]::GetFolderPath([Environment+SpecialFolder]::Programs)
    $ShortcutPath = Join-Path $ProgramsDirectory $ShortcutName
    if (-not (Test-Path -LiteralPath $ShortcutPath -PathType Leaf)) {
        Write-Output 'SEGS 2 Start Menu shortcut is not installed'
        return
    }

    # Protect an unrelated shortcut with the same display name
    $ShortcutObjects = Open-Shortcut $ShortcutPath
    try {
        $TargetPath = $ShortcutObjects[1].TargetPath
        if ([System.IO.Path]::GetFileName($TargetPath) -ine $BinaryName) {
            throw "Refusing to remove a shortcut not owned by $AppName`: $ShortcutPath"
        }
    } finally {
        Close-Shortcut $ShortcutObjects
    }

    Remove-Item -LiteralPath $ShortcutPath
    Write-Output 'Uninstalled SEGS 2 Start Menu shortcut for the current user'
}

# Dispatch the requested desktop integration operation
if (-not $Command) {
    Show-Usage
    exit 2
}

switch ($Command.ToLowerInvariant()) {
    'install' {
        Install-Desktop $BinaryPath
    }
    'uninstall' {
        if ($BinaryPath) {
            Show-Usage
            exit 2
        }
        Uninstall-Desktop
    }
    { $_ -in @('help', '-h', '--help') } {
        if ($BinaryPath) {
            Show-Usage
            exit 2
        }
        Show-Usage
    }
    default {
        Show-Usage
        exit 2
    }
}
