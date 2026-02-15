#!/bin/bash
# Script för att bygga Jobseeker för Android lokalt

set -e

echo "🤖 Jobseeker Android Builder"
echo "=============================="
echo ""

# Sätt miljövariabler för Android-bygge
export ANDROID_HOME="$HOME/AndroidSDK"
export ANDROID_NDK_ROOT="$HOME/AndroidSDK/ndk/25.2.9519653"
export ANDROID_PLATFORM="android-30"
export ANDROID_BUILD_TOOLS_VERSION="35.0.1"

# Hitta Java installation
if [ -f "/home/tuulikk/.antigravity/extensions/redhat.java-1.50.0-linux-x64/jre/21.0.9-linux-x86_64/bin/javac" ]; then
    export JAVA_HOME="/home/tuulikk/.antigravity/extensions/redhat.java-1.50.0-linux-x64/jre/21.0.9-linux-x86_64"
    export PATH="$JAVA_HOME/bin:$PATH"
    echo "✅ Använder Java: $JAVA_HOME"
fi

echo ""

# Kontrollera nödvändiga verktyg
echo "1️⃣  Kontrollerar beroenden..."

if ! command -v rustup &> /dev/null; then
    echo "❌ rustup saknas. Installera från https://rustup.rs/"
    exit 1
fi

if ! command -v java &> /dev/null; then
    echo "❌ java saknas. Installera OpenJDK 17 eller senare"
    exit 1
fi

echo "✅ Rust och Java hittade"
echo ""

# Installera Android targets om de saknas
echo "2️⃣  Kontrollerar Android Rust targets..."
if ! rustup target list | grep -q "aarch64-linux-android (installed)"; then
    echo "📦 Installerar aarch64-linux-android..."
    rustup target add aarch64-linux-android
fi

if ! rustup target list | grep -q "x86_64-linux-android (installed)"; then
    echo "📦 Installerar x86_64-linux-android..."
    rustup target add x86_64-linux-android
fi

echo "✅ Android targets installerade"
echo ""

# Installera cargo-apk om det saknas
echo "3️⃣  Kontrollerar cargo-apk..."
if ! command -v cargo-apk &> /dev/null; then
    echo "📦 Installerar cargo-apk..."
    cargo install cargo-apk
else
    echo "✅ cargo-apk installerat"
fi
echo ""

# Skapa dummy keystore om den saknas
echo "4️⃣  Kontrollerar keystore..."
if [ ! -f "dummy.keystore" ]; then
    echo "📦 Skapar dummy keystore..."
    keytool -genkey -v -keystore dummy.keystore -alias android \
        -keyalg RSA -keysize 2048 -validity 10000 \
        -storepass password -keypass password \
        -dname "CN=Android Debug,O=Android,C=US" 2>/dev/null || true
else
    echo "✅ Keystore finns"
fi
echo ""

# Bygga APK
echo "5️⃣  Bygger APK..."
echo "   Detta kan ta några minuter..."
echo ""

# Bygg utan default-features (ingen AI på Android tills vi löst ONNX)
cargo apk build --release --lib --no-default-features

echo ""
echo "✅ Bygge klart!"
echo ""

# Add DEX file to APK for JNI HTTP calls
echo "📦 Adding DEX classes to APK..."
APK_PATH="target/release/apk/Jobseeker.apk"
if [ -f "$APK_PATH" ] && [ -f "android/build/dex/classes.dex" ]; then
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Extract APK
    unzip -q "$OLDPWD/$APK_PATH"

    # Copy DEX file to root of APK (where Android expects it)
    cp "$OLDPWD/android/build/dex/classes.dex" ./classes.dex

    # Rebuild APK
    zip -q -r "$OLDPWD/$APK_PATH" classes.dex

    # Resign APK (the zip command breaks the signature)
    echo "🔏 Resigning APK..."
    $ANDROID_HOME/build-tools/35.0.1/apksigner sign --ks "$OLDPWD/dummy.keystore" \
        --ks-pass pass:password --key-pass pass:password \
        --out "$OLDPWD/$APK_PATH" "$OLDPWD/$APK_PATH"

    # Cleanup
    cd "$OLDPWD"
    rm -rf "$TEMP_DIR"

    echo "✅ DEX classes added to APK"
else
    echo "⚠️  Warning: Could not add DEX classes (APK or DEX file not found)"
fi
echo ""

# Hitta och visa APK-filen
APK_PATH="target/release/apk/Jobseeker.apk"
if [ -f "$APK_PATH" ]; then
    echo "📱 APK skapad: $APK_PATH"
    
    # Kopiera till tydlig plats
    mkdir -p dist
    cp "$APK_PATH" dist/jobseeker-android.apk
    echo "📱 Kopierad till: dist/jobseeker-android.apk"
    echo ""
    
    # Visa storlek
    SIZE=$(du -h "dist/jobseeker-android.apk" | cut -f1)
    echo "📊 Storlek: $SIZE"
    echo ""
    
    echo "💡 För att installera på enhet/Waydroid:"
    echo "   adb install dist/jobseeker-android.apk"
    echo ""
    echo "💡 För att installera och starta direkt:"
    echo "   adb install dist/jobseeker-android.apk && adb shell am start -n com.gnawsoftware.jobseeker/android.app.NativeActivity"
else
    echo "❌ Kunde inte hitta APK-filen"
    exit 1
fi
