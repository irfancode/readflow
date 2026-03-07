# ReadFlow Installation Script for Windows
# Supports: Windows 10/11
# Run as: powershell -ExecutionPolicy Bypass -File install-windows.ps1

param(
    [string]$InstallPath = "$env:LOCALAPPDATA\ReadFlow",
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Green { param($Message) Write-Host $Message -ForegroundColor Green }
function Write-Yellow { param($Message) Write-Host $Message -ForegroundColor Yellow }
function Write-Red { param($Message) Write-Host $Message -ForegroundColor Red }

Write-Green "Installing ReadFlow v$Version for Windows..."

# Check if Rust is installed
function Test-RustInstalled {
    try {
        $null = Get-Command rustc -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

# Install Rust if not present
function Install-Rust {
    Write-Yellow "Rust not found. Installing Rust..."
    
    $rustupInit = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri https://win.rustup.rs -OutFile $rustupInit -UseBasicParsing
    
    Start-Process -FilePath $rustupInit -ArgumentList "-y", "--default-toolchain", "stable" -Wait -NoNewWindow
    
    # Refresh environment
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    Remove-Item $rustupInit -Force -ErrorAction SilentlyContinue
}

# Build from source
function Build-ReadFlow {
    Write-Yellow "Building ReadFlow from source..."
    
    # Check for source directory
    if (-not (Test-Path "readflow")) {
        Write-Red "Please ensure you have the readflow source code in the current directory"
        exit 1
    }
    
    Set-Location readflow
    
    # Build release
    cargo build --release
    
    # Create installation directory
    New-Item -ItemType Directory -Force -Path $InstallPath | Out-Null
    
    # Copy binary
    Copy-Item "target\release\readflow.exe" -Destination "$InstallPath\"
    
    Write-Green "Installed to $InstallPath\readflow.exe"
}

# Create Start Menu shortcut
function New-Shortcut {
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\ReadFlow.lnk")
    $Shortcut.TargetPath = "$InstallPath\readflow.exe"
    $Shortcut.WorkingDirectory = $InstallPath
    $Shortcut.Description = "A modern TUI browser"
    $Shortcut.Save()
    
    Write-Green "Start Menu shortcut created"
}

# Add to PATH
function Add-ToPath {
    $currentPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$InstallPath*") {
        [System.Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallPath", "User")
        Write-Green "Added to PATH"
    }
}

# Create configuration
function New-Config {
    $configDir = "$env:APPDATA\readflow"
    New-Item -ItemType Directory -Force -Path $configDir | Out-Null
    
    if (-not (Test-Path "$configDir\config.toml")) {
        @"
# ReadFlow Configuration
theme = "dark"
default_url = ""
enable_cookies = true
"@ | Out-File -FilePath "$configDir\config.toml" -Encoding utf8
        
        Write-Green "Configuration created at $configDir"
    }
}

# Main installation
function Main {
    if (-not (Test-RustInstalled)) {
        Install-Rust
    }
    
    Build-ReadFlow
    Add-ToPath
    New-Config
    New-Shortcut
    
    Write-Green "Installation complete!"
    Write-Host "Run 'readflow' to start"
}

Main
