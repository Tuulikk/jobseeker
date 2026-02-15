#!/bin/bash
# Master-script för Android-utveckling: bygg, installera, testa och hämta loggar

set -e

ACTION="${1:-help}"

case "$ACTION" in
    build)
        echo "🔨 Bygger Android APK..."
        ./build_android.sh
        ;;
    
    install)
        echo "📱 Installerar APK på enhet..."
        APK_PATH="dist/jobseeker-android.apk"
        if [ ! -f "$APK_PATH" ]; then
            echo "❌ APK saknas. Kör './dev_android.sh build' först"
            exit 1
        fi
        
        # Avinstallera gammal version om den finns
        adb uninstall com.gnawsoftware.jobseeker 2>/dev/null || true
        
        # Installera ny version
        adb install "$APK_PATH"
        echo "✅ Installerad!"
        ;;
    
    start)
        echo "🚀 Startar Jobseeker..."
        adb shell am start -n com.gnawsoftware.jobseeker/android.app.NativeActivity
        echo "✅ Startad!"
        ;;
    
    restart)
        echo "🔄 Startar om Jobseeker..."
        adb shell am force-stop com.gnawsoftware.jobseeker
        sleep 1
        adb shell am start -n com.gnawsoftware.jobseeker/android.app.NativeActivity
        echo "✅ Omstartad!"
        ;;
    
    stop)
        echo "🛑 Stoppar Jobseeker..."
        adb shell am force-stop com.gnawsoftware.jobseeker
        echo "✅ Stoppad!"
        ;;
    
    logs)
        echo "📋 Hämtar loggar..."
        ./get_android_logs.sh
        ;;
    
    watch)
        echo "👀 Visar live-loggar (Ctrl+C för att avsluta)..."
        ./watch_live_logs.sh
        ;;
    
    test)
        echo "🧪 Kör fullständigt test: bygga -> installera -> starta -> loggar"
        echo ""
        ./build_android.sh
        echo ""
        ./dev_android.sh install
        echo ""
        ./dev_android.sh start
        echo ""
        echo "⏳ Väntar 5 sekunder..."
        sleep 5
        echo ""
        ./dev_android.sh logs
        ;;
    
    clean)
        echo "🧹 Rensar byggfiler..."
        cargo clean
        rm -rf dist
        rm -rf android_logs
        echo "✅ Rensat!"
        ;;
    
    release)
        echo "🚀 Bygger och installerar release-version..."
        echo "   Detta tar längre tid men ger bättre prestanda"
        cargo apk build --release --lib
        
        mkdir -p dist
        find target -name "*.apk" -exec cp {} dist/jobseeker-android-release.apk \;
        
        echo "✅ Release-APK: dist/jobseeker-android-release.apk"
        ./dev_android.sh install
        ;;
    
    help|*)
        echo "🤖 Jobseeker Android Development Tool"
        echo "======================================"
        echo ""
        echo "Användning: ./dev_android.sh <kommando>"
        echo ""
        echo "Kommandon:"
        echo "  build     - Bygger APK"
        echo "  install   - Installerar APK på ansluten enhet"
        echo "  start     - Startar appen"
        echo "  restart   - Startar om appen"
        echo "  stop      - Stoppar appen"
        echo "  logs      - Hämtar och sparar loggar till android_logs/"
        echo "  watch     - Visar live-loggar medan appen körs"
        echo "  test      - Kör fullständig testcykel (build+install+start+logs)"
        echo "  clean     - Rensar byggfiler"
        echo "  release   - Bygger release-version (optimerad)"
        echo ""
        echo "Exempel:"
        echo "  ./dev_android.sh build          # Bygg APK"
        echo "  ./dev_android.sh install        # Installera på Waydroid/enhet"
        echo "  ./dev_android.sh logs           # Hämta loggar efter krasch"
        echo "  ./dev_android.sh test           # Kör fullständig testcykel"
        echo ""
        echo "Tips för felsökning:"
        echo "  1. Bygg: ./dev_android.sh build"
        echo "  2. Installera: ./dev_android.sh install"
        echo "  3. Starta logg-monitor: ./dev_android.sh watch"
        echo "  4. Starta appen i Waydroid"
        echo "  5. Om appen kraschar: ./dev_android.sh logs"
        ;;
esac
