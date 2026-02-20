#!/bin/bash
# Script för att bygga och installera Jobseeker i WayDroid i ett svep

set -e

echo "🤖 Jobseeker - Bygg & Installera"
echo "=================================="
echo ""

# 1. Bygg APK
echo "1️⃣  Bygger APK..."
./build_android.sh

echo ""
echo "2️⃣  Installerar på WayDroid/enhet..."

# Kontrollera att adb hittar en enhet
if ! adb devices | grep -q "device$"; then
    echo "❌ Ingen Android-enhet hittad!"
    echo "   Se till att WayDroid körs: waydroid session start"
    exit 1
fi

# Avinstallera gammal version
echo "   🗑️  Tar bort gammal version..."
adb uninstall com.gnawsoftware.jobseeker 2>/dev/null || true

# Installera ny version
echo "   📦 Installerar ny version..."
adb install -r dist/jobseeker-android.apk

echo ""
echo "3️⃣  Startar appen..."
adb shell am start -n com.gnawsoftware.jobseeker/android.app.NativeActivity

echo ""
echo "✅ Klar! Jobseeker är nu installerad och startad i WayDroid"
echo ""
echo "💡 För att se loggar: ./watch_live_logs.sh"
