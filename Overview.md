# Jobseeker - Översiktsdokument (V0.2)

## Syfte

Jobseeker är en privat, kognitiv förlängning för jobbsökande. Det automatiserar det repetitiva arbetet med att söka, bevaka och rapportera jobbaktiviteter enligt "Gnaw"-filosofin. Appen är byggd för att vara offline-first och lagrar all data lokalt för maximal integritet.

---

## Arkitektur

### Teknikstack
- **UI:** Slint (Native Rust GUI-ramverk)
- **Backend:** Rust med Tokio async runtime
- **Databas:** RedB (Blixtsnabb key-value store, lagrad i en enda `.redb`-fil)
- **API:** JobTech JobSearch API (Individuella sökningar per nyckelord för 100% stabilitet)
- **Synk:** Kontinuerlig filbaserad backup (Dropbox/Syncthing-vänlig)
- **AI:** Ollama-integration (experimentell ranking)

### Filstruktur
```
src/
├── main.rs          - Entry point
├── lib.rs           - Core logik, UI-setup, Clipboard & Export-hantering
├── models.rs        - Datastrukturer & Inställningar
├── api.rs           - API-klient (Hanterar 100-limit och sökords-sanitering)
├── db.rs            - RedB databas wrapper (Trådsäker access)
└── ai.rs            - AI-rankning (Ollama)

ui/
└── main.slint       - UI-definition (Responsive Split-view & Statistik)
```

---

## Core Data & Söklogik

### Sökstrategi (Den "Gnagande" metoden)
🛑 **Viktigt:** Vi söker på varje nyckelord **individuellt**. 
- Varför? JobTech API:s koncept-extrahering är instabil vid komplexa OR-frågor.
- Resultat: Genom att köra separata anrop och deduplicera i Rust garanterar vi att inga jobb missas.
- Limit: API:et har en hård gräns på 100 träffar per anrop som vi respekterar strikt.

### Prioritetszoner (P1, P2, P3)
Användaren definierar tre geografiska zoner. 
- P1 laddas automatiskt vid start.
- Prio-knapparna i UI triggar omedelbara API-sökningar för vald zon.
- Månadsnavigering (pilarna) växlar kontext till databasen för att visa historik.

---

## Funktioner & Moduler

### Statistik & Rapportering
- **Aktivitetsmätare:** Global räknare som visar framsteg mot månadens ansökningsmål.
- **Export:** Genererar formaterade rapporter till Urklipp (med Linux-persistens), E-post eller lokal textfil.
- **Napp-statistik:** Visar vilka sökord som faktiskt ger resultat i inkorgen.

### Automatisk Synk
- **Kontinuerlig Backup:** Varje gång något ändras (nytt jobb, ändrad status, sparade inställningar) triggas en synk.
- **Mål:** Databasen kopieras till en användardefinierad `sync_path`. Detta gör att externa tjänster (Dropbox/Syncthing) omedelbart ser ändringen.

### Databas (RedB)
- **JOB_ADS_TABLE:** Allt data om annonser, inklusive `search_keyword` för statistik.
- **SETTINGS_TABLE:** Lagrar användarens profil, sökord och synk-inställningar.

---

## Thread Safety & UI-mönster

Vi följer ett strikt mönster för att hålla UI:t responsivt:
1. **Event:** UI triggar en callback.
2. **Spawn:** Rust-koden fångar upp data från UI och kör `rt.spawn(async move { ... })`.
3. **Guard:** Vi håller **aldrig** Slint-handtag (`App`) över en `.await`.
4. **Update:** Resultatet skickas tillbaka via `slint::invoke_from_event_loop`.

---

## Status & Roadmap

### ✅ Klart i V0.2
- Stabil Slint-konvertering med responsive design.
- RedB-integration med timestamps för `applied_at`.
- Robust urklippshantering för Linux (via dedikerad tråd).
- Automatisk synk-logik för externa mappar.
- Global progress-bar och statistik-vy.

### 🚀 Kommande (V0.3+)
- SFTP-synk för egen-hostad integritet.
- Formaterad PDF-export med inbäddade typsnitt.
- GnawSense: Fördjupad AI-analys av kravprofiler direkt i UI.