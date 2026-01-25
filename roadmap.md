# Roadmap — Jobseeker 2026

Det här dokumentet sammanfattar visionen och framstegen för Jobseeker. Vi har gått från en instabil prototyp till en robust, privat och "superior" applikation för jobbsökande.

## ✅ Slutförda Milstolpar (V0.2.x)

### Arkitektur & Stabilitet
- **Slint-konvertering:** Hela gränssnittet i Rust + Slint för cross-platform stöd.
- **RedB-databas:** Blixtsnabb lokal lagring i en enda fil.
- **Trådsäkerhet:** Tokio-baserad motor som kör sökningar och statistik asynkront.
- **Individuell sökning:** Garanterad 100% träffsäkerhet genom att gnaga igenom sökord ett och ett.

### Funktioner
- **Statistik & Napp-tracking:** Se exakt vilka ord och områden som ger resultat.
- **Export-system:** Formaterade rapporter till Urklipp (Linux-fixad), E-post och fil.
- **Automatisk Synk:** Kontinuerlig backup till Dropbox/Syncthing-mappar.
- **DPI-stöd:** Uppskalat UI (12px+) för bättre läsbarhet på alla skärmar.

---

## 🚀 Nästa Steg (V0.3.0+)

### 🌍 Global Expansion & Modularitet
- **JobProvider Trait:** Refaktorera API-koden till en modulär arkitektur för att enkelt kunna lägga till nya källor.
- **Fler Källor:** Implementera stöd för t.ex. Adzuna (globalt), USAJOBS eller specifika bransch-API:er.
- **API-Key Management:** Hantering av personliga nycklar för externa tjänster i Inställningar.

### 📱 Android Polering
- **JNI-integration:** Säkerställa att systemfunktioner som "Öppna i webbläsare" och "Kopiera" fungerar 100% via Androids egna systemanrop.
- **UI-anpassning:** Finjustera touch-ytor och mobil-layout för en "native" känsla.

### 📡 Utökad Synk & Moln
- **SFTP-synk:** Inbyggt stöd för att synka direkt mot en egen server.
- **Konflikthantering:** Smartare hantering om databasen ändrats på flera enheter samtidigt.

### 🤖 AI-förbättringar (GnawSense)
- **Krav-analys:** AI-varningssystem om en annons kräver något du saknar (t.ex. körkort).
- **Motivationsbrev:** Automatiskt generera utkast baserat på din profil och annonsens text.

---

## 🛠 Underhåll & Release
- **CI/CD:** Full automatisering via GitHub Actions (Windows, Linux, Android APK).
- **Semantic Versioning:** Följa strikt versionshantering för alla releaser.
- **Dokumentation:** Hålla `Overview.md` och `README.md` uppdaterade.