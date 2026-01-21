# Roadmap — Jobseeker

Det här dokumentet samlar en genomgång av nuvarande funktionalitet, problem, och konkreta åtgärder för att få ordning på appen. Fokus ligger på att återställa och säkra funktionalitet för prioriterade geografiska områden (P1, P2, P3) enligt krav: det ska finnas tre separata prio-fält (ett per prio-nummer). Allt innehåll refererar till aktuell kodbas (Slint / Rust).

Notera: enligt din instruktion prioriterar vi inte `Utkast` / editor / import / export (de bitarna lämnas för tillfället).

---

## Snabb sammanfattning
- Status: Appen fungerar delvis — vissa saker fungerar nu som tidigare var trasiga, men andra funktioner som tidigare fungerade har brutits.
- Huvudproblem (hög prioritet): Inställningarna visar en enda "Geografi"-ruta med endast ett redigerbart fält (`locations_p1`), men systemet och söklogiken förstår tre prioriterade zoner (`locations_p1`, `locations_p2`, `locations_p3`). Alltså:
  - Vad som ska finnas: 3 prio-fält, ett per prio-nummer.
  - Vad som inte ska finnas: Ett enda fält (en fristående "Geografi"-ruta).
- Viktigt: sökning på `Prio 1` ska köras vid appstart (det finns redan kod som gör detta).

---

## Relevanta kodreferenser (snabbsökning)
- `AppSettings` (model):
```Jobseeker/src/models.rs#L109-117
pub struct AppSettings {
    pub keywords: String,
    pub blacklist_keywords: String,
    pub locations_p1: String,
    pub locations_p2: String,
    pub locations_p3: String,
    pub my_profile: String,
    pub ollama_url: String,
}
```

- Default (visar att `locations_p2` och `locations_p3` finns):
```Jobseeker/src/models.rs#L134-144
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            keywords: "...".to_string(),
            blacklist_keywords: "...".to_string(),
            locations_p1: "1283, ...".to_string(),
            locations_p2: "1280, 1281".to_string(),
            locations_p3: "".to_string(),
            ...
        }
    }
}
```

- Settings-UI visar endast ett fält (GEOGRAFI + `loc-p1`):
```Jobseeker/ui/main.slint#L330-360
Text { text: "GEOGRAFI"; color: #4a90e2; ... }
loc-p1 := LineEdit { text: root.settings.locations_p1; placeholder-text: "Prio 1 (Kommuner)..."; }
...
Button {
    text: "Spara Inställningar";
    clicked => {
        root.settings.keywords = keywords-input.text;
        root.settings.blacklist_keywords = blacklist-input.text;
        root.settings.locations_p1 = loc-p1.text;
        root.settings.my_profile = profile-input.text;
        root.save-settings(root.settings);
    }
}
```

- Söklogik använder prio-parameter och kan selektera p1/p2/p3:
```Jobseeker/src/lib.rs#L600-605
let (raw_query, locations_str) = match (free_query, prio) {
    (Some(q), _) => (q, String::new()),
    (None, Some(p)) => {
        let locs = match p {
            1 => &settings.locations_p1,
            2 => &settings.locations_p2,
            3 => &settings.locations_p3,
            _ => &settings.locations_p1,
        };
        (settings.keywords.clone(), locs.clone())
    },
    _ => (String::new(), String::new()),
};
```

- Vid uppstart läses inställningar och en initial `Prio 1`-sökning körs:
```Jobseeker/src/lib.rs#L192-222
// Load settings initially, trigger P1 search and load current month from DB
...
ui.set_settings(AppSettings {
    keywords: ...,
    locations_p1: normalize_locations(&settings_for_ui.locations_p1).into(),
    locations_p2: normalize_locations(&settings_for_ui.locations_p2).into(),
    locations_p3: normalize_locations(&settings_for_ui.locations_p3).into(),
    ...
});
...
perform_search(..., Some(1), None, settings_for_callback.clone()).await;
```

- Inbox UI har knappar för P1/P2/P3 (knapparna finns i UI):
```Jobseeker/ui/main.slint#L170-180
Button { text: "P1"; ... clicked => { root.search-prio(1); } }
Button { text: "P2"; ... clicked => { root.search-prio(2); } }
Button { text: "P3"; ... clicked => { root.search-prio(3); } }
```

---

## Problemdefinition (konkret)
1. Modell & sök-logik stödjer tre prio-områden (P1/P2/P3).
2. Settings-UI visar i praktiken bara ett redigerbart fält (`loc-p1`) trots att `AppSettings` har `locations_p2` och `locations_p3`.
3. Save-knappen i settings-UI skriver bara tillbaka `locations_p1` (användaren kan inte ändra p2/p3 via UI).
4. Slutresultat: användare kan inte konfigurera P2/P3 via GUI, även om P2/P3 används av söklogik om de fanns i DB / tidigare sparade inställningar.

---

## Förslag: Konkreta åtgärder (hög prioritet)
1. UI: Lägg till input-fält för P2 och P3 i Inställningar
   - Var: `Jobseeker/ui/main.slint` — under nuvarande GEOGRAFI-sektion.
   - Förslag: byt rubrik till t.ex. "PRIORITERADE OMRÅDEN" och lägg till:
```Jobseeker/ui/main.slint#L336-356
Text { text: "PRIORITERADE OMRÅDEN"; color: #4a90e2; font-weight: 700; font-size: 11px; }
loc-p1 := LineEdit { text: root.settings.locations_p1; placeholder-text: "Prio 1 (Kommuner)..."; }
loc-p2 := LineEdit { text: root.settings.locations_p2; placeholder-text: "Prio 2 (Kommuner)..."; }
loc-p3 := LineEdit { text: root.settings.locations_p3; placeholder-text: "Prio 3 (Kommuner)..."; }
```

2. UI: Uppdatera Save-knappen så att p2/p3 också sparas
```Jobseeker/ui/main.slint#L352-360
clicked => {
    root.settings.keywords = keywords-input.text;
    root.settings.blacklist_keywords = blacklist-input.text;
    root.settings.locations_p1 = loc-p1.text;
    root.settings.locations_p2 = loc-p2.text;   // NY
    root.settings.locations_p3 = loc-p3.text;   // NY
    root.settings.my_profile = profile-input.text;
    root.save-settings(root.settings);
}
```

3. Enhetstest: spara/läs inställningar (roundtrip)
   - Lägg till en test som sparar `AppSettings` (med p1/p2/p3 satta) via `Db::save_settings` och verifierar `Db::load_settings` returnerar dem.
   - Exempel-test (skelett):
```/dev/null/tests/settings_roundtrip.rs#L1-40
#[test]
fn settings_roundtrip() {
    // 1) Skapa temporär db-path
    // 2) Initiera Db::new(db_path)
    // 3) Spara AppSettings med locations_p1/p2/p3
    // 4) Läs tillbaka och assert_eq!
}
```

4. Integrationstest: verifiera att `perform_search(..., Some(1), ...)` använder `locations_p1` och att knappar P1/P2/P3 triggar korrekt prio
   - Mocka `JobSearchClient::search` eller verifiera att `parse_locations` får rätt string.
   - Lägg test för `JobSearchClient::parse_locations` som testar tolkning av kommaseparerade koder och namn.

5. UI-smoke: Snabb manuell kontroll vid PR
   - Efter PR: starta app, gå till Inställningar, sätt P1/P2/P3, spara, starta om och kontrollera att fälten kvarstår samt att P1-sökning körs vid start.

---

## Test- & processförslag (minska regressionsrisk)
- Tvinga minst en test som täcker settings roundtrip i CI.
- Branch policy: Inga direkta commits till main utan PR och review (särskilt UI/inställningsrelaterade ändringar).
- Lägg till en enkel linter / formatter-check i CI (cargo fmt + clippy).
- Sätt upp en "smoketest" som kan köras i CI (eller manuellt) som läser/sparar settings och verifierar att p1/p2/p3 håller.
- Dokumentera i README/SLINT_CONVERSION att "Geografi" i inställningar ska ersättas av "Prioriterade områden (P1/P2/P3)".

---

## Acceptanskriterier (för att markera ärendet klart)
- Tre synliga input-fält finns i Inställningar (Prio 1, Prio 2, Prio 3).
- Spara-knappen uppdaterar `locations_p1`, `locations_p2`, `locations_p3` i inställningsobjektet och i DB (`Db::save_settings`).
- Efter restart av appen syns de sparade värdena i UI.
- P1 körs automatiskt vid appstart med `locations_p1`.
- Knapparna P1/P2/P3 i Inbox triggar sökning med respektive locations.
- Minst ett unit/integration-test som verifierar settings-roundtrip och prio-val finns i testsviten och körs i CI.

---

## Lågprioriterade / efterföljande åtgärder
- Förbättra UI-text (hjälptext) så att det är tydligt att fälten accepterar kommunnamn eller kod.
- Utöka loggningen runt `parse_locations` för att snabbare upptäcka när AI/andra förändringar tömmer eller felaktigt modifierar inställningar.
- Sätt upp ett test som kör normalize_locations() med olika inputs och validerar resultat (förhindra inkompatibilitet).
- Eventuellt add en enkel migration/validator när settings laddas för att upptäcka "suspicious" värden som AI ändrat.

---

## Kort om historia & hur vi går vidare
- `Prio`-sökning och location-parsing implementerades tidigare (commit: feat: implement priority search...)
- Problemet nu är en regressions-UI där endast ett fält visas och kan ändras.
- Nästa naturliga steg: jag kan skapa en liten PR (ändringar i `ui/main.slint` + tester i `tests/` eller `src/`) som:
  - lägger till fälten för P2/P3
  - uppdaterar save-handlern
  - lägger till ett test som verifierar roundtrip
  - uppdaterar en rad i `SLINT_CONVERSION.md`/README för att stämma överens med det nya (rubrik + hjälptext)

---

STATUS UPDATE
-------------
Jag har implementerat förändringen i branch `fix/settings-prio-restore` och lagt till tester. Det som ingår:

- Commit `5a07cd7`: återintroducerade `loc-p2` och `loc-p3` i `ui/main.slint` och uppdaterade save-handler så att `locations_p2` och `locations_p3` sparas.
- Commit `2f66044`: lade till tester för `normalize_locations`.
- Integrationstest `tests/settings_roundtrip.rs` verifierar att `Db::save_settings` / `Db::load_settings` bevarar `locations_p1/2/3`.

Jag har kört `cargo test` lokalt och alla tester passerade. Vill du att jag pushar branchen till origin och öppnar en PR nu (så körs CI och vi får en pipeline/verifiering)? Om du föredrar att jag väntar med PR tills du har kollat koden först så säger du det — annars pushar jag och öppnar PR.