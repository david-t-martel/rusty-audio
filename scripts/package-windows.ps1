# Windows Packaging Script for Rusty Audio
# Creates MSI installer and portable distributions
# Requires: WiX Toolset 3.11+ for MSI creation

param(
    [string]$Version = "0.1.0",
    [switch]$CreateMSI = $false,
    [switch]$CreatePortable = $true,
    [string]$Configuration = "release"
)

$ErrorActionPreference = "Stop"

# Configuration
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$DistDir = Join-Path $ProjectRoot "dist\windows"
$TargetDir = Join-Path $ProjectRoot "target\x86_64-pc-windows-msvc\release"
$BinaryName = "rusty-audio_native.exe"
$OutputName = "rusty-audio.exe"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Rusty Audio - Windows Packaging" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Version: $Version" -ForegroundColor Yellow
Write-Host "Configuration: $Configuration" -ForegroundColor Yellow
Write-Host ""

# Create distribution directory
Write-Host "[1/6] Creating distribution directory..." -ForegroundColor Green
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

# Build the release binary if not exists
$BinaryPath = Join-Path $TargetDir $BinaryName
if (-not (Test-Path $BinaryPath)) {
    Write-Host "[2/6] Building release binary..." -ForegroundColor Green
    $env:RUSTFLAGS = "-C target-cpu=native -C link-arg=/STACK:8388608"
    cargo build --release --features native-binary --target x86_64-pc-windows-msvc

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed!"
        exit 1
    }
} else {
    Write-Host "[2/6] Using existing binary: $BinaryPath" -ForegroundColor Yellow
}

# Verify binary
Write-Host "[3/6] Verifying binary..." -ForegroundColor Green
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found: $BinaryPath"
    exit 1
}

$BinarySize = (Get-Item $BinaryPath).Length / 1MB
Write-Host "  Binary size: $([math]::Round($BinarySize, 2)) MB" -ForegroundColor Cyan

# Copy binary to dist
$DistBinary = Join-Path $DistDir $OutputName
Copy-Item $BinaryPath $DistBinary -Force
Write-Host "  Copied to: $DistBinary" -ForegroundColor Cyan

# Create portable distribution
if ($CreatePortable) {
    Write-Host "[4/6] Creating portable distribution..." -ForegroundColor Green

    $PortableDir = Join-Path $DistDir "rusty-audio-portable-$Version"
    New-Item -ItemType Directory -Force -Path $PortableDir | Out-Null

    # Copy executable
    Copy-Item $DistBinary "$PortableDir\" -Force

    # Copy documentation
    $DocsToInclude = @("README.md", "LICENSE", "USER_MANUAL.md", "PERFORMANCE_GUIDE.md")
    foreach ($doc in $DocsToInclude) {
        $docPath = Join-Path $ProjectRoot $doc
        if (Test-Path $docPath) {
            Copy-Item $docPath "$PortableDir\" -Force
        }
    }

    # Create portable README
    $PortableReadme = @"
Rusty Audio - Portable Windows Distribution
============================================

Version: $Version
Built: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
Architecture: x86_64 (64-bit)

QUICK START
-----------
1. Double-click rusty-audio.exe to launch
2. Load audio files via File menu or drag-and-drop
3. Adjust EQ, effects, and volume as desired
4. Press F11 for fullscreen mode

FEATURES
--------
- 8-band parametric equalizer (60Hz to 7680Hz)
- Real-time spectrum analyzer (512-point FFT)
- Signal generator (sine, square, sawtooth, noise)
- Audio recording with monitoring
- Multiple themes (7 themes available)
- Professional audio processing pipeline

SYSTEM REQUIREMENTS
-------------------
- Windows 10/11 (64-bit)
- 4GB RAM minimum, 8GB recommended
- DirectX 12 or Vulkan support
- Audio output device
- 100MB disk space

AUDIO DRIVERS
-------------
This application supports:
- WASAPI (Windows Audio Session API) - Default
- ASIO (Audio Stream Input/Output) - Professional audio
- DirectSound - Legacy compatibility

For best latency, use ASIO drivers if available.

CONFIGURATION
-------------
Settings are stored in:
%APPDATA%\rusty-audio\config.toml

TROUBLESHOOTING
---------------
- No audio output: Check audio device selection in Settings
- High latency: Try ASIO drivers or reduce buffer size
- Graphics issues: Update GPU drivers
- Crashes: Check Windows Event Viewer for details

SUPPORT
-------
- GitHub: https://github.com/david-t-martel/rusty-audio
- Issues: https://github.com/david-t-martel/rusty-audio/issues
- Wiki: https://github.com/david-t-martel/rusty-audio/wiki

LICENSE
-------
See LICENSE file for licensing information.

"@

    $PortableReadme | Out-File -FilePath "$PortableDir\PORTABLE-README.txt" -Encoding UTF8

    # Create ZIP archive
    $ZipPath = Join-Path $DistDir "rusty-audio-portable-windows-x64-$Version.zip"
    Compress-Archive -Path $PortableDir -DestinationPath $ZipPath -Force

    Write-Host "  Portable package created: $ZipPath" -ForegroundColor Cyan
    Write-Host "  Size: $([math]::Round(((Get-Item $ZipPath).Length / 1MB), 2)) MB" -ForegroundColor Cyan
}

# Create MSI installer (requires WiX Toolset)
if ($CreateMSI) {
    Write-Host "[5/6] Creating MSI installer..." -ForegroundColor Green

    # Check if WiX is installed
    $WixCandle = Get-Command candle.exe -ErrorAction SilentlyContinue
    $WixLight = Get-Command light.exe -ErrorAction SilentlyContinue

    if (-not $WixCandle -or -not $WixLight) {
        Write-Warning "WiX Toolset not found. Skipping MSI creation."
        Write-Host "  Install WiX from: https://wixtoolset.org/releases/" -ForegroundColor Yellow
    } else {
        # Create WiX source file
        $WixSource = @"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*"
           Name="Rusty Audio"
           Language="1033"
           Version="$Version.0"
           Manufacturer="Rusty Audio Team"
           UpgradeCode="12345678-1234-1234-1234-123456789012">

    <Package InstallerVersion="200"
             Compressed="yes"
             InstallScope="perMachine"
             Description="Professional audio player built in Rust"
             Comments="Rusty Audio v$Version" />

    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <MediaTemplate EmbedCab="yes" />

    <Feature Id="ProductFeature" Title="Rusty Audio" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
      <ComponentRef Id="ApplicationShortcut" />
    </Feature>

    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder">
        <Directory Id="INSTALLFOLDER" Name="Rusty Audio" />
      </Directory>
      <Directory Id="ProgramMenuFolder">
        <Directory Id="ApplicationProgramsFolder" Name="Rusty Audio"/>
      </Directory>
    </Directory>

    <DirectoryRef Id="ApplicationProgramsFolder">
      <Component Id="ApplicationShortcut" Guid="*">
        <Shortcut Id="ApplicationStartMenuShortcut"
                  Name="Rusty Audio"
                  Description="Professional audio player"
                  Target="[INSTALLFOLDER]rusty-audio.exe"
                  WorkingDirectory="INSTALLFOLDER"/>
        <RemoveFolder Id="CleanUpShortCut" Directory="ApplicationProgramsFolder" On="uninstall"/>
        <RegistryValue Root="HKCU"
                       Key="Software\RustyAudio"
                       Name="installed"
                       Type="integer"
                       Value="1"
                       KeyPath="yes"/>
      </Component>
    </DirectoryRef>

    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="MainExecutable" Guid="*">
        <File Id="RustyAudioEXE"
              Source="$DistBinary"
              KeyPath="yes"
              Checksum="yes">
          <Shortcut Id="DesktopShortcut"
                    Directory="DesktopFolder"
                    Name="Rusty Audio"
                    Description="Professional audio player"
                    WorkingDirectory="INSTALLFOLDER"
                    Advertise="yes" />
        </File>
      </Component>
    </ComponentGroup>
  </Product>
</Wix>
"@

        $WixFile = Join-Path $DistDir "rusty-audio.wxs"
        $WixSource | Out-File -FilePath $WixFile -Encoding UTF8

        # Compile WiX source
        $WixObj = Join-Path $DistDir "rusty-audio.wixobj"
        & candle.exe -out $WixObj $WixFile

        # Link to create MSI
        $MsiPath = Join-Path $DistDir "rusty-audio-setup-$Version.msi"
        & light.exe -out $MsiPath $WixObj -ext WixUIExtension

        if (Test-Path $MsiPath) {
            Write-Host "  MSI installer created: $MsiPath" -ForegroundColor Cyan
            Write-Host "  Size: $([math]::Round(((Get-Item $MsiPath).Length / 1MB), 2)) MB" -ForegroundColor Cyan
        } else {
            Write-Warning "MSI creation failed"
        }

        # Clean up WiX artifacts
        Remove-Item $WixFile -Force -ErrorAction SilentlyContinue
        Remove-Item $WixObj -Force -ErrorAction SilentlyContinue
    }
} else {
    Write-Host "[5/6] Skipping MSI creation (use -CreateMSI to enable)" -ForegroundColor Yellow
}

# Generate checksums
Write-Host "[6/6] Generating checksums..." -ForegroundColor Green
$ChecksumFile = Join-Path $DistDir "SHA256SUMS.txt"
Remove-Item $ChecksumFile -ErrorAction SilentlyContinue

$FilesToHash = Get-ChildItem $DistDir -Include "*.exe", "*.zip", "*.msi" -Recurse | Where-Object { $_.DirectoryName -eq $DistDir }

foreach ($file in $FilesToHash) {
    $hash = (Get-FileHash $file.FullName -Algorithm SHA256).Hash
    "$hash  $($file.Name)" | Out-File -FilePath $ChecksumFile -Append -Encoding ASCII
}

Write-Host "  Checksums written to: $ChecksumFile" -ForegroundColor Cyan

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Packaging Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Output directory: $DistDir" -ForegroundColor Yellow
Write-Host ""
Write-Host "Files created:" -ForegroundColor Yellow
Get-ChildItem $DistDir -Include "*.exe", "*.zip", "*.msi" -Recurse | Where-Object { $_.DirectoryName -eq $DistDir } | ForEach-Object {
    $size = [math]::Round(($_.Length / 1MB), 2)
    Write-Host "  - $($_.Name) ($size MB)" -ForegroundColor Cyan
}
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Test the portable distribution" -ForegroundColor White
Write-Host "  2. Test the MSI installer (if created)" -ForegroundColor White
Write-Host "  3. Upload to GitHub Releases" -ForegroundColor White
Write-Host ""
