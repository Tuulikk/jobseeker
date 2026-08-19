# Roadmap — Jobseeker 2026

Det här dokumentet sammanfattar visionen och framstegen för Jobseeker. Vi har gått från en instabil prototyp till en robust, privat och "superior" applikation för jobbsökande.

## ✅ Slutförda Milstolpar (V0.2.x) — kodverifierade 2026 (motor2-audit)

### Arkitektur & Stabilitet
- **Slint-konvertering:** ✅ Hela UI:t i Rust + Slint (build.rs:24, lib.rs setup_ui, android_main).
- **RedB-databas:** ✅ En fil, sex tabeller (db.rs:7-12, redb 2.4.0).
- **Trådsäkerhet:** ✅ Tokio-runtime + Slint event-loop-brygga.
- **Individuell sökning:** ✅ Term-för-term i perform_search (lib.rs:1353-1413), limit 100.

### Funktioner
- **Statistik & Napp-tracking:** ✅ Statusräknare + top-10 nyckelord (refresh_stats).
- **Export-system:** ✅ Urklipp, e-post (mailto) och fil (.txt). ⚠️ PDF-export
  är Linux-only (hårdkodad DejaVuSans-sökväg, exporter.rs:6) — flyttas till V0.3.0.
- **Automatisk Synk:** ✅ Mapp-baserad merge (JSON-mellanhand) efter sökning/status;
  konfliktlösning = senaste updated_at.
- **DPI-stöd:** ✅ (samplingsverifierat, 14px-bas).

### Kända dokumentationsfel
- AGENTS.md: "quotes are automatically applied" stämmer INTE med koden
  (lib.rs:1367 tar bort citattecken; api.rs skickar q rå). Utred innan
  JobProvider-refaktorn.

---

## 🧹 Städning krävs FÖRE V0.3.0 (blockers, motor2-audit 2026-08-19)

1. [ ] Ta bort döda filer (verifierat ej refererade):
      `src/slint_main.rs`, `src/android_content_resolver.rs`,
      `src/*.rs.tmp` (7 st), `ui/*.slint.tmp` (4 st),
      `test_lib`, `test_string`, `dist/Jobseeker-0.2.12.apk` (föråldrad).
2. [ ] Kör `cargo run --bin test_query_logic` + `test_api_mini` före och
      efter städningen (AGENTS.md: Testing as Source of Truth).
3. [ ] Deduplisera HTML-städning/JobEntry-mappning (lib.rs:1371 vs 883) och
      `has_mouse_wheel`-blocken (lib.rs:1451/1475) — halverar risken i
      kommande UI-arbete.
4. [ ] Besluta quoting-doktrin (se Kända dokumentationsfel) och uppdatera
      AGENTS.md så den matchar koden.

---

## 🚀 Nästa Steg (V0.3.0+) — oförändrade mål, tillagda förutsättningar

### 🌍 Global Expansion & Modularitet
- **JobProvider Trait:** Utgå från api.rs (JobSearchClient, base_url api.rs:29,
  MUNICIPALITIES api.rs:12-20). Behåll "en query per kommun + term-för-term"
  som JobTech-provider-specifikt (AGENTS.md Optimization Trap #1) — lägg
  strategin I providern, inte i lib.rs där den ligger nu (lib.rs:1394).
- **PDF-export portabilitet** (ny): bunta font som asset, ta bort hårdkodad
  sökväg — krav för CI/CD-byggen av Windows.
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