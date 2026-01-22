# Jobseeker - Översiktsdokument

## Syfte

Jobseeker är ett verktyg för att strukturera och effektivisera jobbsökandet genom att centralisera annonser från Arbetsförmedlingen, automatisera sökning över flera geografiska områden, och erbjuda stöd för att skriva och exportera ansökningar till PDF/Word.

All data sparas lokalt – ingen molnlagring används.

---

## Arkitektur

### Teknikstack
- **UI:** Slint (Rust-baserat GUI-ramverk)
- **Backend:** Rust med Tokio async runtime
- **Databas:** RedB (embedded, key-value store)
- **API:** Arbetsförmedlingens JobSearch API (https://jobsearch.api.jobtechdev.se)
- **AI (experimentellt):** Ollama för lokal AI-rankning
- **Plattform:** Cross-platform (Linux, Windows, macOS, Android-stöd inbyggt)

### Filstruktur
```
src/
├── main.rs          - Entry point (delegerar till desktop_main)
├── lib.rs           - Core logik, UI-setup, callbacks
├── models.rs        - Datastrukturer (JobAd, AppSettings, AdStatus)
├── api.rs           - API-klient för JobSearch, kommun-kod mapping
├── db.rs            - RedB databas wrapper
└── ai.rs            - AI-rankning (Ollama)

ui/
└── main.slint       - UI-definition (Slint DSL)
```

---

## Core Data Structures

### AppSettings (src/models.rs:109-117)
```rust
pub struct AppSettings {
    pub keywords: String,              // Sökord för jobb
    pub blacklist_keywords: String,     // Nyckelord att filtrera bort
    pub locations_p1: String,          // Prio 1 kommuner
    pub locations_p2: String,          // Prio 2 kommuner
    pub locations_p3: String,          // Prio 3 kommuner
    pub my_profile: String,            // Profiltext för AI
    pub ollama_url: String,            // URL till lokal Ollama
}
```

**Varför tre prioritetsområden?**
Gör det möjligt att söka i olika regioner med olika prioritet t.ex.:
- P1: Hemregionen (Helsingborg, Ängelholm...)
- P2: Större städer i närheten (Malmö, Lund)
- P3: Reserv/övriga regioner

Appen kör automatiskt P1-sökning vid start.

### AdStatus (src/models.rs:4-12)
```rust
pub enum AdStatus {
    New = 0,
    Rejected = 1,
    Bookmarked = 2,
    ThumbsUp = 3,
    Applied = 4,
}
```

Status sparas i databasen och används för filtrering i UI. "Rejected" annonser visas inte i lista men sparas kvar.

---

## Sökflöde

### Priority Search (src/lib.rs:674-688)
När användaren klickar P1/P2/P3:
1. Hämta inställningar från DB
2. Välj `locations_p1/p2/p3` baserat på prio
3. Anropa `JobSearchClient::search()` med keywords + municipalities
4. Filtrera bort annonser som matchar `blacklist_keywords`
5. Visa resultat i UI

### Normalisering av platser (src/lib.rs:168-186)
`normalize_locations()` gör två saker:
1. Löser numeriska koder till kommunnamn (t.ex. "1283" → "Helsingborg")
2. Title-casar namn (t.ex. "malmö" → "Malmö")

Detta för att UI ska vara användarvänligt (visa namn) men API-anrop ska vara korrekta (skicka koder).

### Kommun-mapping (src/api.rs:11-42)
`MUNICIPALITIES` arrayen mappar kommunnamn till Arbetsförmedlingens kommun-ID (t.ex. "helsingborg" → "1283"). Används av:
- `parse_locations()` - konverterar namn → koder
- `get_municipality_name()` - konverterar koder → namn

---

## Databas (RedB)

### Tables (src/db.rs:7-9)
- `JOB_ADS_TABLE` - Sparade jobbannonser
- `APPLICATIONS_TABLE` - Sparade ansökningsutkast
- `SETTINGS_TABLE` - Användarinställningar

### Key methods
- `save_job_ad()` / `get_job_ad()` - CRUD för annonser
- `get_filtered_jobs(statuses, year, month)` - Hämta annonser med filter, sorterad efter `applied_at` eller `bookmarked_at` fallback till `internal_created_at`
- `save_settings()` / `load_settings()` - Roundtrip för inställningar
- `update_ad_status()` - Uppdatera status och sätter timestamps (`applied_at`, `bookmarked_at`)

### Varför RedB?
Embedded (inget separat DB-tjänst), typsäker via Rust, snabb för key-value patterns. Använd JSON-serialisering för komplexa objekt.

---

## Detaljerad Sök- och Dataflöde

### Konceptuellt Flöde

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              UI-LAYER (Slint)                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   P1 P2 P3   │  │ Fri sökning  │  │  Filter-ik   │  │  Månad < >   │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
└─────────┼──────────────────────┼──────────────────┼──────────────────┼──────────┘
          │                      │                  │                  │
          ▼                      ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              APP-LAYER (Rust)                                    │
│                                                                                 │
│  ui.on_search_prio()       ui.on_search_pressed()    (inget separat callback) │
│         │                           │                                         │
│         ▼                           ▼                                         │
│  rt.spawn(async move)       rt.spawn(async move)                               │
│         │                           │                                         │
│         ▼                           ▼                                         │
│  db.load_settings()         db.load_settings()                                │
│         │                           │                                         │
│         ▼                           ▼                                         │
│  perform_search(prio,..)    perform_search(free_query,..)                     │
│         │                           │                                         │
│         ▼                           ▼                                         │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                       perform_search()                                 │   │
│  │  1. Välj query/locations baserat på prio eller free search            │   │
│  │  2. parse_locations() → konvertera kommunnamn → koder                   │   │
│  │  3. api_client.search() → HTTP GET till JobSearch API                  │   │
│  │  4. Filtrera bort blacklist_keywords                                   │   │
│  │  5. Slå upp status i DB för varje annons                               │   │
│  │  6. Convert JobAd → JobEntry (Slint struct)                            │   │
│  │  7. ui.set_jobs() via invoke_from_event_loop()                         │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│  ui.on_month_offset()                                                        │
│         │                                                                     │
│         ▼                                                                     │
│  rt.spawn(async move)                                                        │
│         │                                                                     │
│         ▼                                                                     │
│  Beräkna ny månad (year/month math)                                         │
│         │                                                                     │
│         ▼                                                                     │
│  db.get_filtered_jobs(year, month)                                          │
│         │                                                                     │
│         ▼                                                                     │
│  Convert JobAd → JobEntry                                                    │
│         │                                                                     │
│         ▼                                                                     │
│  ui.set_jobs() via invoke_from_event_loop()                                   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            API-LAYER (JobSearch)                                │
│                                                                                 │
│  GET https://jobsearch.api.jobtechdev.se/search                                │
│  ?q=it+supporttekniker                                                         │
│  &limit=100                                                                     │
│  &municipality=1283                                                            │
│  &municipality=1277                                                            │
│  &municipality=1260                                                            │
│                                                                                 │
│  Response: JSON                                                                 │
│  {                                                                             │
│    "hits": [                                                                   │
│      {                                                                         │
│        "id": "001",                                                            │
│        "headline": "Supporttekniker",                                          │
│        "description": {"text": "<p>Vi söker...</p>"},                        │
│        "webpage_url": "https://arbetsformedlingen...",                        │
│        "publication_date": "2026-01-15T10:00:00",                            │
│        "employer": {"name": "Tech AB"},                                       │
│        "workplace_address": {"city": "Helsingborg", "municipality": "1283"}   │
│      },                                                                        │
│      ...                                                                       │
│    ]                                                                           │
│  }                                                                             │
└─────────────────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            DB-LAYER (RedB)                                      │
│                                                                                 │
│  JOB_ADS_TABLE          APPLICATIONS_TABLE         SETTINGS_TABLE              │
│  ┌─────────────┐        ┌─────────────┐          ┌─────────────┐              │
│  │ "001" → JSON│        │ "001" → ... │          │ "current"→ │              │
│  │ "002" → JSON│        │ "003" → ... │          │   JSON      │              │
│  └─────────────┘        └─────────────┘          └─────────────┘              │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### API-Request Format (Problem)

**Aktuellt implementation (src/api.rs:87-97):**
```rust
pub async fn search(&self, query: &str, municipalities: &[String], limit: u32) {
    let mut params = vec![
        ("q", query.to_string()),
        ("limit", limit.to_string()),
    ];

    for m in municipalities {
        if !m.is_empty() {
            params.push(("municipality", m.to_string()));
        }
    }
    // Resultat: ?q=...&limit=100&municipality=1283&municipality=1277&municipality=1260
}
```

**Problem:**
- Flertalet `municipality` params kan tolkas olika beroende på API-implementering
- API:et kan förvänta sig `&municipality=1283,1277,1260` (kommaseparerad)
- Eller en array-format: `&municipality[]=1283&municipality[]=1277`
- Nuvarande format (`&municipality=1283&municipality=1277`) är standard HTTP query format men bör verifieras mot API-dokumentationen

**Verifiering behövs:**
- Testa mot riktigt API med 1, 2, 3 kommuner
- Kontrollera logs för `Tolkade X st kommun-ID:n` (lib.rs:694)
- Lägg till explicit test som validerar API-response för multi-kommun request

### Konflikt: Prio-sökning vs Månad-navigering

**Aktuell beteende:**
| Åtgärd | Vad händer | Datakälla | State |
|--------|------------|-----------|-------|
| App startar | Laddar P1 + aktuell månad från DB | DB | OK |
| Klicka P1/P2/P3 | API-sökning, **ersätter allt** i `ui.set_jobs()` | API | ❌ Månad ignoreras |
| Klicka månad-pilar | DB-sökning, **ersätter allt** i `ui.set_jobs()` | DB | ❌ Prio ignoreras |
| Filtrera (Bokm./ThumbsUp) | Filtrar aktuell lista i UI | UI state | OK |

**Problemet:**
Det finns ingen "source of truth" för vad som visas. Varje åtgärd (`perform_search`, `get_filtered_jobs`) skriver över hela job-listan utan att hänsyfta till den andra kontexten.

**Önskat beteende:**
1. **Prio-knappar ska vara toggelbara** - visar vilken zon som är aktiv
2. **Aktiv Prio styr över månads-navigering** - när en prio är aktiv ska månads-pilarna **filtrera** API-resultatet för den månad, inte byta källa till DB
3. **När ingen prio är aktiv** - månads-pilarna navigerar i DB-data (sparade annonser)
4. **Separat "Uppdatera"-knapp** - gör ny API-sökning för aktiv prio

**Föreslagen state-modell:**
```
enum DataSource {
    Api { prio: u8, ads: Vec<JobAd> },
    Db { year: i32, month: u32, ads: Vec<JobAd> },
}
```

---

## UI-arkitektur (Slint)

### Main Window (ui/main.slint:379-501)
- **Tabbar:** Jobb (Inbox) | Inställningar
- **Inbox-pane:** Split-view med job-lista till vänster och detaljer till höger
- **Responsive:** På mobilen växlas mellan lista och detaljvy, på desktop visas båda sid-vid-sid

### Components
- `InboxPane` (ui/main.slint:128-239)
  - Toolbar: Fri sökning + P1/P2/P3-knappar + filter-ikoner
  - Month navigator: Bläddra mellan månader
  - Job-list: `JobListItem` per annons
- `JobDetailPane` (ui/main.slint:241-312)
  - Action buttons: Reject, Save, ThumbsUp, Apply, Open
  - Reporting help (om applied): Kopiera företag/kommun/titel
  - Scrollbar beskrivning
- `SettingsPage` (ui/main.slint:314-377)
  - Sökord, blacklist, prioriterade områden, AI-profil
  - System-logs viewer
  - Loggfil-path och senaste API-request för debug

---

## API-Integration

### JobSearch API Details

**Endpoint:** `https://jobsearch.api.jobtechdev.se/search`

**Request format (aktuell):**
```
GET /search?q=it+supporttekniker&limit=100&municipality=1283&municipality=1277
```

**Potentiell formatfel:**
API-dokumentation bör verifieras för:
- Är multi-municipality korrekt format?
- Bör kommuner vara kommaseparerad: `?municipality=1283,1277,1260`?
- Behövs array-syntax: `?municipality[]=1283&municipality[]=1277`?

**API Response Structure:**
```json
{
  "hits": [
    {
      "id": "24912345",
      "headline": "Supporttekniker till växande bolag",
      "description": {
        "text": "<p>Vi söker en person som är...</p>",
        "description": "Kort beskrivning..."
      },
      "webpage_url": "https://arbetsformedlingen.se/platsannons/...",
      "publication_date": "2026-01-15T10:00:00",
      "last_application_date": "2026-02-15T23:59:59",
      "employer": {
        "name": "Tech AB",
        "workplace": "Helsingborg"
      },
      "workplace_address": {
        "city": "Helsingborg",
        "municipality": "1283",
        "municipality_concept_id": "o_5hV8_X1_Spk"
      },
      "working_hours_type": {
        "label": "Heltid"
      },
      "occupation": {
        "label": "IT-supporttekniker"
      },
      "number_of_vacancies": 1
    }
  ],
  "total": {"value": 47}
}
```

### Error Handling Strategy

**API-fel (src/api.rs:111-116):**
- HTTP-status ≠ 200 → `anyhow::Error` med detaljer
- Body loggas för debugging
- UI visar felmeddelande via `status_msg`

**JSON parse-fel (src/api.rs:118-150):**
- Saknad `hits` array → `Context("No 'hits' array found")`
- Individa annonser som inte parsas → `eprintln()` men andra annonser fortsätter
- `working_hours_type` extraheras manuellt om deserialisering misslyckas

**DB-fel:**
- All returnerar `anyhow::Result<T>`
- UI visar fel via `status_msg` men ej panik

**Thread Safety Pattern:**
```
UI Callback → rt.spawn(async task) → API/DB await
                                              │
                                              ▼
                            invoke_from_event_loop → UI update
```

**Varning:** Aldrig hålla `ui` (strong ref) över `.await` → använd `ui_weak`.

### Rate Limiting & Retry

**Nuvarande:** Ingen explicit rate limiting eller retry logic.
**Risk:** API kan blockera vid för många requests.
**Måste:**
- Implementera backoff
- Respekta API rate limits (dokumentera max requests/min)
- Cache API-response kort tid för att undvika dubbla requests

---

## Async & Threading

### Tokio Runtime (src/lib.rs:817-834)
`Arc<Runtime>` delas mellan callbacks. Alla DB- och API-anrop görs async via `rt.spawn()`.

### Thread Safety
- UI-tråden (Slint) måste aldrig blockeras av `.await`
- `slint::invoke_from_event_loop()` används för att skicka data tillbaka till UI-tråden från async tasks

### Exempel: Priority search (src/lib.rs:381-398)
```rust
ui.on_search_prio(move |prio| {
    // (Körs på UI-tråden)
    rt_handle.spawn(async move {
        // (Körs på Tokio worker thread)
        let settings = db.load_settings().await.unwrap_or_default();
        perform_search(api_client, db, ui_weak, Some(prio), None, settings).await;
    });
});
```

---

## Status & Kända Problem

### Fungerar
- ✅ Filter (Alla, Bokmärkta, ThumbsUp, Sökta) - client-side filtering
- ✅ Status-toggle (Reject/Save/ThumbsUp/Apply) med timestamps
- ✅ Month navigation laddar från DB - men konflikt med prio-sökning
- ✅ Settings roundtrip (spara/läs p1/p2/p3)
- ✅ Svartlista-filtrering - client-side filtering av API-resultat
- ✅ Loggning till både fil och UI
- ✅ API request-response parsing (men format ej verifierat för multi-municipality)

### Kritiska Problem

**1. API Request Format (o verifierat)**
- Multi-municipality format: `?municipality=1283&municipality=1277` kan vara fel
- API kan förvänta sig kommaseparerad: `?municipality=1283,1277,1260`
- Behöver testas mot riktigt API med 2+ kommuner

**2. Prio-sökning vs Månad-navigering konflikt**
| Scenario | Aktuell beteende | Förväntat beteende |
|----------|------------------|---------------------|
| App start | Laddar P1 + månad | Laddar P1 + månad ✓ |
| Klicka P1 | API-sök, ersätter all data | API-sök, sätt P1 aktiv ✓ |
| Klicka P2 | API-sök, ersätter all data | API-sök, sätt P2 aktiv ✓ |
| Klicka månad < | DB-sök, ersätter all data | Filtrera aktivt API-resultat på månad |
| Klicka månad när ingen Prio aktiv | DB-sök | DB-sök ✓ |

**Lösning kräver:**
- State för `active_prio: Option<u8>`
- Separat "Uppdatera"-knapp för API-sökning
- Månad-pilar ska filtrera API-resultat när Prio aktiv, annars DB
- Prio-knappar ska vara toggelbara (visar aktiv state)

### Begränsningar / Ej implementerat
- ❌ Utkast-tab (Drafts) finns inte i aktivt UI (finns i `SLINT_CONVERSION.md` men ej kopplad)
- ❌ Ansökningseditor med PDF/Word-export finns ej i aktivt UI
- ❌ AI-rankning implementerad men ej integrerad i UI (button finns ej)
- ❌ Android copy-to-clipboard ej implementerat (JNI krävs)
- ❌ `rfd` (file dialogs) inaktiverat pga ashpd version conflict
- ❌ API rate limiting / retry logic saknas

### Regressionshistorik (från roadmap.md)
Tidigare har AI-agenter:
- Tömt `locations_p2` och `locations_p3` när de ändrat inställningar
- Döljt p2/p3-fält i UI
- Behållit backend-logiken som använder alla tre prio-zoner

Lösades genom:
- UI nu visar `loc-p2` och `loc-p3` explicit (ui/main.slint:340-342)
- Save-handler sparar alla tre fält (ui/main.slint:367-369)
- Tester för `normalize_locations` och `settings_roundtrip`

---

## Designbeslut

### Varför 3 prio-zoner?
- För att Arbetsförmedlingen kan kräva att man söker jobb utanför sitt lännsområde, kanske även riktigt långt ifrån. Då kanske ha krav på minst x antal jobb utanför sitt län. Med 3 prio områden har jag tänkt närmaste kommunerna i Prio 1, Längre bort i länet i Prio 2 och då Prio 3 som är i andra län. Som standard när man öppar appen så ska Prio1 ladda in i inboxen, samt datum pilarna styra över aktiv månad annonserna har lagts upp.

### Varför Slint istället för web/Electron?
- Native prestanda
- Cross-platform med single binary
- Rust integration (typsäkerhet, no GC)
- Minska beroenden (ingen Node.js/Chromium)

### Varför lokal AI (Ollama)?
- Ingen moln-kostnad
- Ingen data skickas ut från användarens maskin
- Full kontroll över modeller och prompter

### Varför RedB istället för SQLite?
- Enklare API för key-value
- Mindre overhead för read/write operations
- Kompatibel med WASM (future proofing)

---

## Tester

### Unit tests (src/lib.rs:188-218)
- `normalize_locations_resolves_codes_and_titlecases_names` - verifierar att koder löses och namn title-casas
- `normalize_locations_trims_and_ignores_empty_entries` - whitespace och tomma entries hanteras
- `parse_locations_resolves_malmo_lund` - kommunnamn → koder

### Integration test (tests/settings_roundtrip.rs)
- Sparar `AppSettings` med p1/p2/p3 via `Db::save_settings`
- Läser tillbaka via `Db::load_settings`
- Verifierar att alla fält är intakta

---

## Nästa steg

### Högre prioritet (kritiska problem)
1. **Verifiera API-format för multi-municipality**
   - Testa `?municipality=1283,1277,1260` (kommaseparerad)
   - Jämför med nuvarande `?municipality=1283&municipality=1277`
   - Dokumentera korrekt format i Overview.md

2. **Fixa Prio/Månad-konflikt**
   - Lägg till state för `active_prio: Option<u8>` i UI
   - Gör P1/P2/P3 knappar togglbara (visar aktiv state)
   - Lägg till separat "Uppdatera"-knapp
   - Månad-pilar ska filtrera API-resultat när Prio aktiv
   - Månad-pilar ska DB-söka när ingen Prio aktiv

3. **Implementera API rate limiting**
   - Backoff-strategi vid timeouts
   - Cache API-response kort tid (30-60 sek)

### Medel prioritet (enligt roadmap.md)
4. Återställa Utkast-funktionalitet
5. Implementera ansökningseditor (PDF/Word export)
6. Integrera AI-rankning i UI
7. Fixa `rfd` dependency issue för file dialogs

---

## Referenser

- README.md - Översikt och kom igång
- roadmap.md - Detaljerad problem- och lösningsbeskrivning
- SLINT_CONVERSION.md - Fullständig lista över UI-komponenter och planerade funktioner
- ui/main.slint - Aktuell UI-definition
