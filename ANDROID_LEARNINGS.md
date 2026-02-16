# Android Networking & Permissions Learnings

Detta dokument beskriver de kritiska lärdomar som drogs när nätverksåtkomst implementerades för Jobseeker i WayDroid/Android-miljö.

## 1. Problemet: SecurityException & Permission Denied
Trots att `INTERNET`-behörighet fanns i `AndroidManifest.xml`, vägrade appen att öppna sockets. Logcat visade `java.lang.SecurityException: Permission denied (missing INTERNET permission?)`.

### Orsak A: `cargo-apk` ignorerade Manifestet
`cargo-apk` genererar ofta ett eget manifest och kan ignorera manuella ändringar i `android/AndroidManifest.xml` om de inte matchar verktygets interna logik.
**Lösning:** Deklarera rättigheter uttryckligen i `Cargo.toml` med den strukturerade syntaxen:
```toml
[[package.metadata.android.uses_permission]]
name = "android.permission.INTERNET"
[[package.metadata.android.uses_permission]]
name = "android.permission.ACCESS_NETWORK_STATE"
```

### Orsak B: GID (Group ID) uppdaterades inte
Android (och WayDroid) tilldelar Linux-grupper (som `inet` / `3003`) endast vid **första installationen**. En vanlig `adb install -r` (reinstall) räcker ofta inte för att uppdatera dessa grupper om de ändrats.
**Lösning:** Avinstallera appen helt innan en ny version med nya rättigheter installeras:
`adb uninstall com.gnawsoftware.jobseeker && adb install dist/jobseeker-android.apk`

## 2. Teknikval: Rust `reqwest` vs Java JNI
Vi försökte länge med en JNI-brygga (`HttpURLConnection` i Java).
*   **JNI-problem:** Java-nätverksklasser kräver ofta en "Android Looper" eller att tråden är skapad på ett specifikt sätt av Android-systemet. Detta krockade med Rusts `tokio`-runtime.
*   **Rust-fördelen:** Genom att använda `reqwest` med `rustls-tls` pratar koden direkt med Linux-kärnan. Så länge appen har rätt GID (3003), fungerar detta utan att behöva krångla med JNI-begränsningar.

## 3. Miljöspecifikt (WayDroid på Fedora)
WayDroid kräver ibland extra handpåläggning på värdmaskinen för att nätverksbryggan ska fungera:
*   **IP Forwarding:** `sudo sysctl -w net.ipv4.ip_forward=1`
*   **Firewall (firewalld):** 
    `sudo firewall-cmd --zone=trusted --add-interface=waydroid0 --permanent`
    `sudo firewall-cmd --zone=FedoraWorkstation --add-masquerade --permanent`
*   **DNS:** `sudo waydroid prop set persist.waydroid.dns 1.1.1.1`

## Sammanfattning för framtiden
1.  Lita inte på att `cargo-apk` läser manifestet korrekt – använd `Cargo.toml` för permissions.
2.  Verifiera alltid med `adb shell dumpsys package <pkg> | grep permission` att rättigheten faktiskt beviljats.
3.  Vid permission-strul: **Avinstallera alltid** innan ny installation.
4.  Använd ren Rust-nätverkskod (`reqwest`) där det är möjligt, det är mer robust än JNI i hybrid-appar.
