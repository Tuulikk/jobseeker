# Iced ↔ Slint — audit och åtgärdsförslag

Syfte
------
Gå igenom git‑historiken och källkoden (Iced‑era + Slint‑era) för att dokumentera hur `Prio`/geografi hanterades tidigare, vad som ändrats och föreslå en säker plan för återställning utan att införa nya regressionsfel.

Kort sammanfattning
-------------------
- Iced-versionen hade tre separata fält för prioriterade områden (Prio 1, Prio 2, Prio 3) i Inställningar och auto-sparade ändringar vid input‑händelser.
- Tidig Slint‑konversion (commit som implementerade prioritetssökning) behöll de tre fälten i `ui/main.slint`.
- I en senare uppdatering (senaste baseline) försvann/ersattes fälten av en mer generisk "GEOGRAFI" sektion med endast ett synligt fält för `locations_p1`.
- Backend (modell + söklogik) stödjer fortfarande tre priofält (`locations_p1/2/3`) — problemet är alltså främst en UI‑regression och brist på test som verifierar att UI sparar alla tre fält.

Historiska noteringar (bevis)
------------------------------
Iced (hur det såg ut tidigare)
```Jobseeker/src/main.rs#L1217-1231
column![
    text("Geografiska områden").size(18).color(Color::from_rgb(0.3, 0.6, 0.8)),
    text("Du kan nu skriva kommunnamn (t.ex. Helsingborg, Malmö) eller koder").size(14)...
    column![
        text("Område 1 (Högsta prioritet)").size(14),
        text_input("Kommuner eller koder", &self.settings.locations_p1).on_input(Message::SettingsLocP1Changed).padding(10),
    ],
    column![
        text("Område 2").size(14),
        text_input("Kommuner eller koder", &self.settings.locations_p2).on_input(Message::SettingsLocP2Changed).padding(10),
    ],
    column![
        text("Område 3").size(14),
        text_input("Kommuner eller koder", &self.settings.locations_p3).on_input(Message::SettingsLocP3Changed).padding(10),
    ],
]
```
- Iced höll separata event/meddelanden (`SettingsLocP1Changed`, `SettingsLocP2Changed`, `SettingsLocP3Changed`) och körde `SaveSettings` som sparade `self.settings` i DB. (Se `Message::SettingsLocP*` och `Message::SaveSettings` i Iced‑koden.)

Tidigare Slint-implementation (efter första konversionen)
```Jobseeker/ui/main.slint#L616-628
VerticalLayout {
    Text { text: "Område Prio 1:"; }
    loc-p1-input := LineEdit { text: root.settings.locations_p1; placeholder-text: "t.ex. stockholm (eller kod)"; }
}
VerticalLayout {
    Text { text: "Område Prio 2:"; }
    loc-p2-input := LineEdit { text: root.settings.locations_p2; }
}
VerticalLayout {
    Text { text: "Område Prio 3:"; }
    loc-p3-input := LineEdit { text: root.settings.locations_p3; }
}
```
och save-handler (samma commit)
```Jobseeker/ui/main.slint#L714-716
Button {
    text: "Spara inställningar";
    clicked => {
        root.settings.locations_p1 = loc-p1-input.text;
        root.settings.locations_p2 = loc-p2-input.text;
        root.settings.locations_p3 = loc-p3-input.text;
        root.save-settings(root.settings);
    }
}
```

Nuvarande Slint (nuvarande problem)
```Jobseeker/ui/main.slint#L330-360
Text { text: "GEOGRAFI"; ... }
loc-p1 := LineEdit { text: root.settings.locations_p1; placeholder-text: "Prio 1 (Kommuner)..."; }
...
// Save-knapp (koden sätter endast locations_p1):
root.settings.locations_p1 = loc-p1.text;
root.save-settings(root.settings);
```

Backend / Modell (visar att supporting code finns)
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

Söklogik använder prio 1/2/3 korrekt:
```Jobseeker/src/lib.rs#L600-610
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

Analys (varför regressionen är farlig)
---------------------------------------
- UI saknar de fält som användaren måste kunna redigera — även om backend stödjer p2/p3 så kan användaren inte ange dem via UI längre.
- Ingen automatisk testning fångade regressionsändringen → ändringen slank igenom och skapade en tyst funktionsbortfall.
- Iced‑versionen auto-sparade vid input, Slint-UI använde en explicit spara‑knapp (skillnaden i UX är relevant: auto-save minskar risken att värden glöms kvar).

Rekommenderad, säker åtgärdsplan
--------------------------------
1. Återställ UI‑fält för `locations_p2` och `locations_p3` i `ui/main.slint` (återskapa det som fanns i commit som introducerade prio‑stöd). Detta är en lokal, låg-risk ändring om den görs isolerat.
   - Fil: `ui/main.slint`.
   - Åtgärd: Lägg tillbaka `loc-p2-input` och `loc-p3-input`, uppdatera save‑knappen så att den sätter `root.settings.locations_p2` och `root.settings.locations_p3` innan `root.save-settings(...)`.

2. Lägg till tester (prioritet hög):
   - Unit test för `normalize_locations` / `beautify_locations` som garanterar att numeriska koder -> namn samt namn lämnas orörda.
   - Integration/test för `Db::save_settings` + `Db::load_settings` roundtrip som verifierar att `locations_p1/2/3` bevaras.
   - (Valfritt) Smoke/integration test som kör `perform_search(..., Some(prio), ...)` och kontrollerar att `parse_locations` får rätt data.

3. CI:
   - Kör `cargo test` i CI och blockera merges om tests misslyckas.
   - Lägg till ett litet "settings roundtrip" test i CI för att tidigt fånga regression i settings.

4. Docs:
   - Uppdatera `SLINT_CONVERSION.md` och `README.md` för att beskriva att det finns tre prio‑fält och att Prio 1 laddas på start.
   - Skriv en kort processbeskrivning i `CONTRIBUTING.md`: UI/inställningsändringar måste uppdatera dokumentation + tester.

5. Process:
   - Gör ändringarna i en separat branch (`fix/settings-prio-restore`) och öppna en liten PR.
   - Inkludera tester först (rörelsen 'test-first' minskar risken för regressionsfixes).
   - Be om minst en reviewer (kodägare) innan merge.

Föreslagen steg-för-steg (tidsuppskattning)
------------------------------------------
- Skapa branch + tests: 1–2 h
- Implementera UI‑ändring (slint): 0.5–1 h
- Kör lokala tester + korrigera: 0.5–1 h
- PR + review + merge: beroende på review (0.5–2 h)

Riskbegränsning / extra säkerhet
-------------------------------
- Inför en test som kontrollerar att `settings.json` serialiserar och deserialiserar `locations_p1/2/3`.
- Lägg till en enkel validator vid load som loggar/varnar om `locations_p2` och `locations_p3` saknas eller är tomma efter en ändring (kan vara en debug-varning tills ny UI finns).
- Kräv att ändringar i `ui/main.slint` måste ha med ett test eller en enkel verifieringssteg i PR‑beskrivningen.

Bilaga — förslag på konkret UI-patch (skiss)
--------------------------------------------
- Återställ blocket med tre inputs (samma struktur som tidigare Slint/Iced). Exempel (prototyp):
```/dev/null/proposed_main.slint#L1-20
Text { text: "Prioriterade områden"; color: #4a90e2; }
loc-p1 := LineEdit { text: root.settings.locations_p1; placeholder-text: "Prio 1 (Kommuner)..."; }
loc-p2 := LineEdit { text: root.settings.locations_p2; placeholder-text: "Prio 2 (Kommuner)..."; }
loc-p3 := LineEdit { text: root.settings.locations_p3; placeholder-text: "Prio 3 (Kommuner)..."; }
```
- Uppdatera save‑handler så att alla tre fälten sparas:
```/dev/null/proposed_main.slint#L1-6
root.settings.locations_p1 = loc-p1.text;
root.settings.locations_p2 = loc-p2.text; // NEW
root.settings.locations_p3 = loc-p3.text; // NEW
root.save-settings(root.settings);
```
(Notera: ovan är en prototyp/skiss; referera till tidigare Slint‑commit om du vill återanvända exakt implementation.)

Avslutning / nästa steg
-----------------------
Vill du att jag:
- (A) implementerar förändringen + lägger till tester och öppnar en PR, eller
- (B) bara förbereder en patch/diff för din granskning, eller
- (C) att jag först skriver tester (utan UI‑ändring) för att skapa en säker bas att jobba mot?

Säg vilket alternativ du föredrar så går jag vidare med det. Jag föreslår att vi börjar med att lägga till tester och sedan göra UI‑ändringen i en liten PR (test → kod → review → merge) så minimerar vi risken för nya regrassioner.