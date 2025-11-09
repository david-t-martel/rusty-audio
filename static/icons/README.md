# PWA Icons

## Required Icons

This directory needs two PNG icons for Progressive Web App (PWA) functionality:

- `icon-192.png` - 192x192 pixels
- `icon-512.png` - 512x512 pixels

## Generating Icons

### Option 1: Using icon.svg (recommended)
An SVG template (`icon.svg`) has been provided. Convert it to PNG using:

**Online tools:**
- https://svgtopng.com/
- https://cloudconvert.com/svg-to-png

**Command line (if ImageMagick installed):**
```powershell
magick convert -density 300 -background none icon.svg -resize 192x192 icon-192.png
magick convert -density 300 -background none icon.svg -resize 512x512 icon-512.png
```

**PowerShell with Inkscape:**
```powershell
inkscape icon.svg --export-filename=icon-192.png --export-width=192 --export-height=192
inkscape icon.svg --export-filename=icon-512.png --export-width=512 --export-height=512
```

### Option 2: Use a Placeholder
For quick testing, create solid color squares:

**Using PowerShell:**
```powershell
# This creates simple colored squares as placeholders
Add-Type -AssemblyName System.Drawing

# 192x192
$bmp192 = New-Object System.Drawing.Bitmap(192, 192)
$graphics192 = [System.Drawing.Graphics]::FromImage($bmp192)
$graphics192.Clear([System.Drawing.Color]::FromArgb(26, 26, 26))
$bmp192.Save("$PWD\icon-192.png", [System.Drawing.Imaging.ImageFormat]::Png)
$graphics192.Dispose()
$bmp192.Dispose()

# 512x512
$bmp512 = New-Object System.Drawing.Bitmap(512, 512)
$graphics512 = [System.Drawing.Graphics]::FromImage($bmp512)
$graphics512.Clear([System.Drawing.Color]::FromArgb(26, 26, 26))
$bmp512.Save("$PWD\icon-512.png", [System.Drawing.Imaging.ImageFormat]::Png)
$graphics512.Dispose()
$bmp512.Dispose()
```

### Option 3: Use PWA Asset Generator
```bash
npm install -g pwa-asset-generator
pwa-asset-generator icon.svg ./
```

## Current Status

⚠️ **Icons not yet generated** - The application will build but PWA installation may not work correctly until icons are present.

## Design Guidelines

The icon should:
- Be recognizable at small sizes (192px)
- Work on both light and dark backgrounds
- Follow Material Design icon guidelines
- Represent audio/music theme
- Use the application's brand colors (#4a9eff blue on dark background)
