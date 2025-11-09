# Rusty Audio - PWA Static Assets

This directory contains static assets for the Progressive Web App deployment of Rusty Audio.

## Directory Structure

```
www/
├── index.html          # Main HTML entry point with WASM loader
├── manifest.json       # PWA manifest (app metadata, icons)
├── sw.js              # Service worker (offline support, caching)
├── icon-*.png         # PWA icons (various sizes) - TO BE GENERATED
├── favicon-*.png      # Browser favicons - TO BE GENERATED
└── README.md          # This file
```

## Required Icons

Generate the following icon sizes for full PWA support:

### Required Sizes
- `icon-72.png` - 72x72px (iOS)
- `icon-96.png` - 96x96px (Android, desktop shortcuts)
- `icon-128.png` - 128x128px (Chrome Web Store)
- `icon-144.png` - 144x144px (Windows tiles)
- `icon-152.png` - 152x152px (iOS)
- `icon-192.png` - 192x192px (Android home screen) **REQUIRED**
- `icon-384.png` - 384x384px (Android splash)
- `icon-512.png` - 512x512px (Android high-res) **REQUIRED**

### Favicon Sizes
- `favicon-16x16.png` - 16x16px
- `favicon-32x32.png` - 32x32px

### Maskable Icons (Optional)

For better Android integration, create maskable versions with safe zone:
- Use 80% of canvas for icon content
- 20% padding around edges for system masking

## Generating Icons

### Method 1: Online Tool (Recommended)

Use [RealFaviconGenerator](https://realfavicongenerator.net/):

1. Upload a 512x512px master icon
2. Select "Progressive Web App" option
3. Download generated icon pack
4. Copy all files to this directory

### Method 2: ImageMagick (Command Line)

```bash
# Create a 512x512 master icon first (master.png)
# Then generate all sizes:

for size in 72 96 128 144 152 192 384 512; do
  convert master.png -resize ${size}x${size} icon-${size}.png
done

# Generate favicons
convert master.png -resize 16x16 favicon-16x16.png
convert master.png -resize 32x32 favicon-32x32.png
```

### Method 3: Inkscape (Vector Source)

If you have an SVG source:

```bash
for size in 72 96 128 144 152 192 384 512; do
  inkscape -w $size -h $size logo.svg -o icon-${size}.png
done
```

## Design Guidelines

### Icon Design Best Practices

1. **Simple and Recognizable**
   - Use bold, simple shapes
   - Avoid fine details (won't show at small sizes)
   - High contrast between foreground and background

2. **Color Scheme**
   - Match app theme: #4ecdc4 (teal) and #1a1a1a (dark)
   - Consider dark mode and light mode contexts
   - Use transparency carefully (some platforms don't support)

3. **Maskable Icon Safe Zone**
   - Keep important content in center 80% of canvas
   - Test with mask previews: [maskable.app](https://maskable.app/)

4. **Format**
   - PNG format (best compatibility)
   - 24-bit RGB + alpha channel
   - Optimize with `pngquant` or `optipng` after generation

### Example Master Icon Concept

```
🎵 Music note symbol
+ Retro car stereo aesthetic
+ Teal/dark theme colors
+ Circular or rounded square background
```

## Screenshots (Optional)

Add screenshots for better Play Store / App Store listings:

- `screenshot-wide.png` - 1280x720px (desktop view)
- `screenshot-mobile.png` - 750x1334px (mobile view)

## Customization

### Editing manifest.json

```json
{
  "name": "Your Custom Name",
  "short_name": "Short Name",
  "theme_color": "#your-color",
  "background_color": "#your-bg-color"
}
```

### Editing Service Worker (sw.js)

Customize caching behavior:

```javascript
// Change cache version
const CACHE_NAME = 'rusty-audio-v1.1.0';

// Add more files to cache
const STATIC_ASSETS = [
  // ... add your files
];

// Adjust cache size limits
const MAX_AUDIO_CACHE_SIZE = 200 * 1024 * 1024; // 200MB
```

## Testing PWA Features

### Local Testing

1. Build the WASM project:
   ```bash
   cd ..
   ./scripts/build-wasm.sh
   ```

2. Serve from `dist/` directory:
   ```bash
   cd dist
   python3 -m http.server 8080
   ```

3. Open DevTools (F12) → Application tab:
   - ✅ Manifest: Check all icons load
   - ✅ Service Worker: Verify registration
   - ✅ Cache Storage: Confirm files cached

### PWA Installability

Requirements:
- ✅ Served over HTTPS (or localhost)
- ✅ Valid `manifest.json`
- ✅ Icons: 192x192 and 512x512 minimum
- ✅ Service worker registered
- ✅ `start_url` loads successfully

### Browser DevTools Checks

**Chrome/Edge:**
```
DevTools → Application → Manifest
- Check "Add to Home Screen" works
- Verify icon display

DevTools → Lighthouse
- Run PWA audit
- Aim for score > 90
```

**Firefox:**
```
DevTools → Application → Manifest
- Check manifest parsing
- Verify service worker
```

**Safari:**
```
Develop → Service Workers
- Check registration
- Note: Limited PWA support on iOS
```

## Deployment

After generating icons, the build script automatically copies all assets to `dist/`:

```bash
./scripts/build-wasm.sh
```

Output structure:
```
dist/
├── index.html
├── manifest.json
├── sw.js
├── rusty_audio.js
├── rusty_audio_bg.wasm
└── icon-*.png  (copied from www/)
```

## Troubleshooting

### Icons not showing

1. Check file names match manifest.json
2. Verify PNG format and dimensions
3. Check browser console for 404 errors
4. Clear browser cache and service worker

### Service worker not registering

1. Must use HTTPS (localhost exempt)
2. Check `sw.js` is in correct location
3. Look for JavaScript errors in console
4. Verify MIME type: `application/javascript`

### PWA not installable

1. Run Lighthouse audit for detailed errors
2. Check all icons present (especially 192 and 512)
3. Verify manifest.json is valid JSON
4. Ensure `start_url` is accessible

## Resources

- [Web.dev PWA Guide](https://web.dev/progressive-web-apps/)
- [PWA Builder](https://www.pwabuilder.com/)
- [Icon Generator](https://realfavicongenerator.net/)
- [Maskable Icon Editor](https://maskable.app/editor)
- [Lighthouse CI](https://github.com/GoogleChrome/lighthouse-ci)
