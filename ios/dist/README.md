# amp - iOS Installation Guide

**Version:** 1.0.0
**Platform:** iOS 16.0+
**Architecture:** arm64 (iPhone 6s and later)

## What is amp?

amp correlates street addresses with parking restriction zones in Malmo, Sweden. It works fully offline - no internet connection required after installation.

## Installation (Sideloading)

iOS does not allow installing apps directly from `.ipa` files without code signing. Choose one of the methods below.

---

### Method 1: AltStore / SideStore (Recommended - No Mac required after setup)

1. **Install AltStore** on your iPhone:
   - Download AltServer on your Mac/PC from [altstore.io](https://altstore.io)
   - Connect your iPhone via USB
   - Install AltStore to your iPhone through AltServer
   - On your iPhone, trust the AltStore developer profile in **Settings > General > VPN & Device Management**

2. **Install amp.ipa via AltStore:**
   - Transfer `amp.ipa` to your iPhone (AirDrop, iCloud Drive, email, etc.)
   - Open the file and choose **"Open in AltStore"**
   - AltStore will sign and install the app
   - The app will appear on your home screen

3. **Renewal:** AltStore apps expire after 7 days. AltStore refreshes them automatically if your iPhone and Mac/PC are on the same Wi-Fi network. Keep AltServer running on your computer.

---

### Method 2: Xcode (Requires Mac)

1. **Unzip** `amp.ipa` (rename to `.zip` and extract, or use `unzip amp.ipa`)
2. You'll get a `Payload/amp.app` folder
3. **Open Xcode** > Window > Devices and Simulators
4. Connect your iPhone via USB
5. Select your device in the left sidebar
6. Drag `amp.app` from `Payload/` onto the **Installed Apps** section
7. Xcode will ask you to select a signing team - use your Apple ID
8. Trust the developer profile on your iPhone: **Settings > General > VPN & Device Management**

---

### Method 3: Apple Configurator 2 (Requires Mac)

1. Install **Apple Configurator 2** from the Mac App Store
2. Connect your iPhone via USB
3. Select your device
4. Go to **Add > Apps** and select `amp.ipa`
5. Trust the developer profile on your device

---

### Method 4: TrollStore (Jailbroken/TrollStore-compatible devices)

If your device supports TrollStore (check [ios.cfw.guide](https://ios.cfw.guide)):

1. Transfer `amp.ipa` to your iPhone
2. Open the file in TrollStore
3. TrollStore will install it permanently (no 7-day expiration)

---

## After Installation

- **First launch:** The app will request notification permissions (for parking restriction alerts)
- **GPS:** When using the GPS button, the app will request location permission
- **Import/Export:** The app's Documents folder is visible in the **Files** app (On My iPhone > amp). Place `.parquet` files in the `import` subfolder to import data.
- **Offline:** The app works entirely offline. All parking data is bundled in the app.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Untrusted Developer" | Settings > General > VPN & Device Management > Trust |
| App won't install | Make sure your device runs iOS 16.0 or later |
| App expires after 7 days | Normal with free signing. Use AltStore auto-refresh or TrollStore |
| Blank screen on launch | Force-close and reopen. If persistent, reinstall. |

## Technical Details

- **Bundle ID:** `se.malmo.skaggbyran.amp`
- **Binary:** arm64 Mach-O (unsigned)
- **UI Framework:** Dioxus (Rust) via WKWebView
- **Size:** ~7.5 MB (compressed)
