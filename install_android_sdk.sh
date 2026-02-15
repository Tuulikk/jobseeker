#!/bin/bash
# Installera Android SDK till user directory (ingen root krävs)

set -e

INSTALL_DIR="$HOME/AndroidSDK"
CMDTOOLS_VERSION="11076708"
CMDTOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-${CMDTOOLS_VERSION}_latest.zip"

echo "📦 Installerar Android SDK till: $INSTALL_DIR"
echo ""

# Skapa katalogstruktur
mkdir -p "$INSTALL_DIR/cmdline-tools"
mkdir -p "$INSTALL_DIR/tmp"

# Ladda ner commandlinetools
if [ ! -f "$INSTALL_DIR/tmp/cmdline-tools.zip" ]; then
    echo "⬇️  Laddar ner Android Command Line Tools..."
    wget -O "$INSTALL_DIR/tmp/cmdline-tools.zip" "$CMDTOOLS_URL"
fi

# Packa upp
if [ ! -d "$INSTALL_DIR/cmdline-tools/latest" ]; then
    echo "📂 Packar upp..."
    unzip -q "$INSTALL_DIR/tmp/cmdline-tools.zip" -d "$INSTALL_DIR/tmp/"
    mv "$INSTALL_DIR/tmp/cmdline-tools" "$INSTALL_DIR/cmdline-tools/latest"
fi

# Acceptera licenser
echo "📜 Accepterar licenser..."
yes | "$INSTALL_DIR/cmdline-tools/latest/bin/sdkmanager" --licenses --sdk_root="$INSTALL_DIR" 2>/dev/null || true

# Installera nödvändiga paket
echo "📦 Installerar NDK och platform-tools..."
"$INSTALL_DIR/cmdline-tools/latest/bin/sdkmanager" \
    "ndk;25.2.9519653" \
    "platforms;android-30" \
    "build-tools;30.0.3" \
    "platform-tools" \
    --sdk_root="$INSTALL_DIR"

echo ""
echo "✅ Android SDK installerad!"
echo ""
echo "Lägg till detta i din ~/.bashrc eller ~/.zshrc:"
echo ""
echo "export ANDROID_HOME=\"$INSTALL_DIR\""
echo "export ANDROID_SDK_ROOT=\"$INSTALL_DIR\""
echo "export PATH=\"\$PATH:\$ANDROID_HOME/cmdline-tools/latest/bin:\$ANDROID_HOME/platform-tools\""
echo ""
echo "Kör sedan: source ~/.bashrc  (eller source ~/.zshrc)"
