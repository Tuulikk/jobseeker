# Android Development Guide 🤖

Snabbguide för att utveckla och felsöka Jobseeker på Android/Waydroid.

## Förutsättningar

- Rust toolchain med Android targets
- Java (OpenJDK 17+)
- ADB (Android Debug Bridge) - följer med Android SDK eller Waydroid
- Waydroid eller fysisk Android-enhet

## Snabbstart

```bash
# 1. Bygg APK
./dev_android.sh build

# 2. Installera på enhet/Waydroid
./dev_android.sh install

# 3. Starta appen
./dev_android.sh start

# 4. Hämta loggar (om appen kraschar)
./dev_android.sh logs
```

## Kommandon

| Kommando | Beskrivning |
|----------|-------------|
| `./dev_android.sh build` | Bygger APK |
| `./dev_android.sh install` | Installerar på enhet |
| `./dev_android.sh start` | Startar appen |
| `./dev_android.sh restart` | Startar om appen |
| `./dev_android.sh stop` | Stoppar appen |
| `./dev_android.sh logs` | Hämtar loggar till `android_logs/` |
| `./dev_android.sh watch` | Visar live-loggar |
| `./dev_android.sh test` | Kör fullständig testcykel |
| `./dev_android.sh clean` | Rensar byggfiler |
| `./dev_android.sh release` | Bygger optimerad release-version |

## Felsökning

### Appen kraschar vid start

1. **Bygg och installera:**
   ```bash
   ./dev_android.sh build
   ./dev_android.sh install
   ```

2. **Starta logg-monitoring:**
   ```bash
   ./dev_android.sh watch
   ```

3. **Starta appen i Waydroid/Android**

4. **Om appen kraschar, hämta loggar:**
   ```bash
   ./dev_android.sh logs
   ```

5. **Titta i loggarna:**
   - `android_logs/session_*/startup_test.txt` - Vilken steg nådde appen?
   - `android_logs/session_*/crash.log` - Crash-loggar
   - `android_logs/session_*/jobseeker.log` - Detaljerad tracing
   - `android_logs/session_*/logcat_filtered.txt` - Filtrerade system-loggar

### Ingen enhet hittad

- Om du använder Waydroid: `waydroid session start`
- Kontrollera: `adb devices` (ska visa din enhet)

### Byggfel

```bash
# Rensa och försök igen
./dev_android.sh clean
./dev_android.sh build
```

## Loggning

Appen loggar automatiskt till:
- **Logcat:** Android-systemets loggning
- **Fil:** `/data/data/com.gnawsoftware.jobseeker/files/jobseeker.log`
- **Crash:** `/data/data/com.gnawsoftware.jobseeker/files/crash.log`
- **Startup:** `/data/data/com.gnawsoftware.jobseeker/files/startup_test.txt`

## Arkitektur

### Android-specifik kod

- **Entry point:** `src/lib.rs` -> `android_main()`
- **UI Framework:** Slint med Android backend
- **Loggning:** `android_logger` + `tracing` med fil-output
- **Database:** Redb (samma som desktop)

### Viktiga filer

- `src/lib.rs` - Huvudkod inklusive `android_main()`
- `android/AndroidManifest.xml` - Android manifest
- `build.rs` - Bygg-script

## Waydroid-specifikt

För att utveckla med Waydroid:

```bash
# Starta Waydroid session
waydroid session start

# Visa Waydroid fönster
waydroid show-full-ui

# I en annan terminal, bygg och installera
./dev_android.sh build
./dev_android.sh install
```

## Performance

Bygg för release vid testing av prestanda:

```bash
./dev_android.sh release
./dev_android.sh install
./dev_android.sh start
```

Release-versionen är mycket snabbare än debug-versionen.

## Troubleshooting

### Appen startar inte alls

1. Kontrollera startup_test.txt - vilken step nåddes?
2. Leta i jobseeker.log efter felet
3. Använd logcat_filtered.txt för Android-specifika fel

### Appen krascherar efter start

1. crash.log visar panic-meddelandet
2. jobseeker.log visar vad som hände precis innan kraschen
3. Använd stacktrace från crash.log för att lokalisera felet

### Svart skärm

Ofta UI-relaterat. Kolla:
- Är Slint korrekt initierat? (Step 12 i startup_test.txt)
- Är UI setup klar? (Step 13 i startup_test.txt)

## Script-referens

- `dev_android.sh` - Master-script för allt
- `build_android.sh` - Bygger APK
- `get_android_logs.sh` - Hämtar loggar från enhet
- `watch_live_logs.sh` - Live logg-monitoring
