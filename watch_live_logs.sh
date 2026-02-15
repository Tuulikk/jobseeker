#!/bin/bash
# Script för att se Android-loggar live medan Jobseeker startar

echo "🔍 Jobseeker Live Log Monitor"
echo "================================"
echo ""
echo "Starta Jobseeker nu (klicka på ikonen i Waydroid)"
echo ""
echo "Tryck Ctrl+C för att avsluta"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Filtrera och visa relevanta loggar i realtid
adb logcat -c  # Rensa buffer
adb logcat -v time | grep --line-buffered -i "jobseeker\|gnawsoftware\|android_main\|tokio\|slint" &
LOGCAT_PID=$!

# Alternativt: Visa ALLA loggar om ovanstående inte visar något
sleep 2
echo "💡 Om inget visas ovan, prova att ta bort filtret:"
echo "   adb logcat -v time | grep -i 'rust\|panic\|fatal'"
echo ""

wait $LOGCAT_PID
