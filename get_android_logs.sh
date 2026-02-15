#!/bin/bash
# Script för att hämta loggar från Android/Waydroid för felsökning av Jobseeker

set -e

LOG_DIR="android_logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SESSION_DIR="$LOG_DIR/session_$TIMESTAMP"

echo "🔍 Jobseeker Android Log Extractor"
echo "===================================="
echo ""

# Skapa log-katalog
mkdir -p "$SESSION_DIR"

# Kontrollera om en enhet är ansluten
if ! adb devices | grep -q "device$"; then
    echo "❌ Ingen Android-enhet hittad. Kontrollera att Waydroid körs eller enheten är ansluten."
    echo "   Tips: 'waydroid session start' om du använder Waydroid"
    exit 1
fi

echo "✅ Enhet ansluten"
echo ""

# Hämta logcat (live loggar)
echo "📱 Hämtar logcat-loggar..."
adb logcat -d > "$SESSION_DIR/logcat.txt" 2>/dev/null || true

# Filtrera logcat för Jobseeker
echo "🔍 Filtrerar Jobseeker-loggar..."
grep -i "jobseeker\|gnawsoftware\|android_main" "$SESSION_DIR/logcat.txt" > "$SESSION_DIR/logcat_filtered.txt" 2>/dev/null || true

# Hämta fil-loggar från appens data-katalog
echo "📂 Hämtar fil-loggar från enheten..."
adb shell "run-as com.gnawsoftware.jobseeker cat /data/data/com.gnawsoftware.jobseeker/files/startup_test.txt" > "$SESSION_DIR/startup_test.txt" 2>/dev/null || echo "Kunde inte läsa startup_test.txt (behöver appen vara startad?)" > "$SESSION_DIR/startup_test.txt"

adb shell "run-as com.gnawsoftware.jobseeker cat /data/data/com.gnawsoftware.jobseeker/files/crash.log" > "$SESSION_DIR/crash.log" 2>/dev/null || echo "Ingen crash.log hittades (inga krascher sedan senast)" > "$SESSION_DIR/crash.log"

adb shell "run-as com.gnawsoftware.jobseeker cat /data/data/com.gnawsoftware.jobseeker/files/jobseeker.log" > "$SESSION_DIR/jobseeker.log" 2>/dev/null || echo "Ingen jobseeker.log hittades" > "$SESSION_DIR/jobseeker_error.txt"

# Lista alla filer i appens data-katalog
echo "📋 Listar filer i appens data-katalog..."
adb shell "run-as com.gnawsoftware.jobseeker ls -la /data/data/com.gnawsoftware.jobseeker/files/" > "$SESSION_DIR/files_list.txt" 2>/dev/null || echo "Kunde inte lista filer" > "$SESSION_DIR/files_list.txt"

# Hämta databasen om den finns
echo "💾 Hämtar databas..."
adb shell "run-as com.gnawsoftware.jobseeker cat /data/data/com.gnawsoftware.jobseeker/files/jobseeker.redb" > "$SESSION_DIR/jobseeker.redb" 2>/dev/null || echo "Kunde inte läsa databasen" > "$SESSION_DIR/db_error.txt"

echo ""
echo "✅ Loggar hämtade till: $SESSION_DIR"
echo ""
echo "📊 Sammanfattning:"
echo "   - logcat.txt: Fullständiga Android-loggar (logcat)"
echo "   - logcat_filtered.txt: Filtrerade Jobseeker-loggar"
echo "   - startup_test.txt: Startupp-test-loggar (vilken steg nåddes)"
echo "   - crash.log: Crash-loggar (om appen kraschat)"
echo "   - jobseeker.log: Detaljerad tracing-logg från appen"
echo "   - files_list.txt: Lista över filer i appens katalog"
echo "   - jobseeker.redb: Databas (om kunde läsas)"
echo ""
echo "💡 Tips för att läsa loggarna:"
echo "   1. Titta i startup_test.txt - vilken steg nådde appen?"
echo "   2. Leta i crash.log efter panic/felet (finns om appen kraschat)"
echo "   3. Använd jobseeker.log för detaljerad tracing av vad som hände"
echo "   4. Använd logcat_filtered.txt för Android-system-loggar"
echo ""
echo "🔍 Snabbanalys:"
if grep -q "Step 16" "$SESSION_DIR/startup_test.txt" 2>/dev/null; then
    echo "   ✅ Appen startade fullständigt ( nådde Step 16 )"
    echo "      Kolla jobseeker.log för vad som hände efter start"
elif grep -q "Step 15" "$SESSION_DIR/startup_test.txt" 2>/dev/null; then
    echo "   ⚠️  Appen nådde UI-run-loop (Step 15) men可能在 kraschat"
    echo "      Kolla crash.log och jobseeker.log"
else
    echo "   ❌ Appen kraschade tidigt - kolla vilken steg i startup_test.txt"
fi
echo ""
echo "🔴 För att se loggar live (när appen startar):"
echo "   adb logcat | grep -i 'jobseeker\\|gnawsoftware\\|android_main'"
