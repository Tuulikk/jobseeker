# Roadmap för 1.0 (MVP+)

Detta dokument listar vad som redan finns implementerat i projektet, och vad som bör göras för att nå en stabil 1.0-release (MVP+). Målet för 1.0 är ett användbart verktyg där man kan:
- skapa och redigera ansökningar,
- spara och hantera utkast,
- exportera ansökningar (Word/HTML/PDF-flöde),
- generera handläggarrapporter (DOCX/PDF),
- ha grundläggande UX, testning och release-paketering.

Funktioner som möjliggör att lägga till fler "aktörer" än Arbetsförmedlingen och flerspråkighet är bra att planera men kan behöva en större refaktorering och därför görs primärt efter 1.0.

---

## ✅ Vad som redan är gjort (status today)
- Editor & UI
  - Markdown-baserad editor med verktygsfält (fetstil, kursiv, rubriker, listor, infoga företag, klistra in profil). Implementerat i `src/main.rs` och `src/rich_editor.rs`.
  - **Stabil Markdown-förhandsvisning** med `rich_editor::markdown::to_iced` – ger formaterad förhandsvisning (rubriker, stycken, listor) direkt i appen via Iced-widgets.
  - Enhetstester för Markdown-rendering tilläggda i `rich_editor::tests`.
  - Mallgenerator: `rich_editor::markdown::create_template(company, position, profile)`.

- Utkast & persistens
  - Autolagring av utkast vid ändring (`Message::EditorContentChanged` ➜ `Db::save_application_draft`) i `src/db.rs`.
  - Ladda sparade utkast (`Db::get_application_draft`, `Db::get_all_drafts`).

- Import/Export
  - Import av textfiler (`Message::ImportFile`).
  - Export till HTML (användbart för att skriva ut till PDF via webbläsare) med `rich_editor::markdown::to_html`.
  - Export till DOCX via `rich_editor::export::markdown_to_docx`.
  - Ett grundläggande `markdown_to_pdf` finns som skapar HTML (på sikt behöver vi antingen bundla `wkhtmltopdf` eller använda headless-chrome).

- Jobbsök & metadata
  - Inhämtning av jobbannonser via `JobSearchClient` (`src/api.rs`).
  - Spara jobbannonser i DB med metadata i `src/db.rs` (`save_job_ad`), status/rating, och filtrering.

- Enkel AI-integration
  - `AiRanker::rate_job` i `src/ai.rs` används för att ge en matchningspoäng (implementerat).
  - "Förbättra text" är ännu placeholder (`EditorAiImprove` är inte färdig).

- Verktyg
  - Kopiera annons till urklipp, öppna länk i webbläsare, mailto-delningsfunktion finns i `src/main.rs`.
  - Inställningar är sparade i `settings.json` via `AppSettings` i `src/models.rs`.

---

## 🎯 MVP+ (Vad som bör vara klart inför 1.0)
Prioriterade funktioner som ger ett användbart 1.0 (MVP+).

2. Stabil export & handläggarrapporter (DOCX/PDF)
   - Varför: Handläggare ska kunna ta emot professionella rapporter med ansökningar.
   - Acceptance:
     - UI för att välja flera ansökningar och generera en "Handläggarrapport".
     - Rapport kan exporteras till DOCX och PDF.
     - Rapporten innehåller: metadata (datum, sökande, jobbmeta), ansökningstexter och en sidhuvud/sidfot.
   - Estimat: 1-2 veckor
   - Berörda filer: `src/main.rs`, nytt `src/reports.rs` (eller liknande), `src/rich_editor.rs::export::markdown_to_docx` (utökas för header/footer), samt PDF-flöde.

3. Sidhuvud & sidfot i export
   - Varför: Professionella dokument kräver header/footer (t.ex. klient- och handläggarinfo).
   - Acceptance: Möjlighet sätta globalt eller per-ansökan sidhuvud/sidfot i inställningar eller i rapport-generatorn; inkluderas i DOCX och HTML exports.
   - Estimat: 2-4 dagar (DOCX: kontrollera `docx-rs` API för headers/footers).

4. Förbättra PDF-export
   - Varför: Direkt PDF-export från appen (inte bara via "skriv ut från webbläsare").
   - Acceptance: Integrerat konverteringssteg (ex. köra `wkhtmltopdf` om tillgängligt, eller använda headless Chrome) med god felhantering.
   - Estimat: 3-7 dagar beroende på lösning och distribution.

5. UX-polish & feedback
   - Varför: Bra UX minskar fel och support.
   - Acceptance: Progressindikatorer vid långa operationer (export, sök), success/error-notifieringar, möjligheten att byta namn på utkast.
   - Estimat: 2-4 dagar.

6. Testning & CI
   - Varför: Stabilitet och snabb återkoppling.
   - Acceptance: Enhetstester för markdown -> HTML, DB (integrationstest mot temporärt DB), exportfunktioner; GitHub Actions som kör build + tester.
   - Estimat: 2-5 dagar.
   - **Status:** Enhetstester för Markdown-rendering på plats (`rich_editor::tests`).

7. Release & packaging
   - Varför: Användare behöver enkla installers/binaries.
   - Acceptance: Cross-platform build pipeline (Windows/macOS/Linux) och publicerad 1.0 release med ändringslogg.
   - Estimat: 3-7 dagar (beroende på signering/os-specifika krav).

---

## ♻️ Refaktor/arkitektur som bör planeras efter 1.0
Dessa är större förändringar som rimligtvis kan vara post-1.0 eftersom de kräver schema-migration och designarbete.

- Multi-aktörer & flerspråkighet
  - Förslag: Byt `AppSettings` till strukturerad konfiguration med `actors: [{id, name, templates, contact, language}]`.
  - Konsekvens: Kräver migrering av `settings.json` och uppdaterad UI för att hantera aktörer och språkval.
  - Prioritering: Efter 1.0 (större refactor).

- Templating-system
  - Förslag: Inför `tera` eller `handlebars` för att hantera mallar (ansökan + sidhuvud/sidfot/rapport), så att placeholders (företag, roll, datum, sökande) kan fyllas dynamiskt.
  - Konsekvens: Bättre kontroll över lokalisering och per-aktör-mallar.

- Förbättrad AI-integration
  - Förslag: Implementera `EditorAiImprove` med säker, återanvändbar chat-kommunikation, och möjlighet att granska/sammanfatta AI-förslag innan de appliceras i dokumentet.

- DB-migration & versionshantering
  - Varför: Schemaändringar (t.ex. per-ansökan metadata) behöver migrationssteg.

---

## Tekniska risker och val att ta ställning till
- Val av PDF-verktyg: bundla `wkhtmltopdf`, kräver distributions-överväganden, eller använd headless chrome vilket är tungt men mer flexibelt.
- DOCX-API-begränsningar: `docx-rs` kan kräva extra arbete för avancerade headers/footers eller sidnumrering.
- Multi-aktör stöd kräver tydlig datamodell; om det införs tidigt kan många UI-flöden förenklas.

---

## Tidslinje & milstolpar (förslag)
- Sprint 1 (1–2 veckor): "Ny ansökan"-UI, rename utkast, autosave stabilitet, små UX-förbättringar.
- Sprint 2 (1–2 veckor): Rapport-generator MVP (DOCX), header/footer grund, tests for export.
- Sprint 3 (1–2 veckor): PDF-export/packaging, CI + tests, polish.
- Release: 1.0 (MVP+) med release notes som beskriver begränsningar (t.ex. "språk: svenska", "aktörer: grundläggande").

---

## Konkreta nästa PR:er (kort lista)
1. Lägg till "Ny ansökan"-knapp + `Message::NewApplication` + UI-test.
2. Stöd för att byta namn på utkast (headline) och visa det i Drafts-listan.
3. Utöka `export::markdown_to_docx` med parametrar `header`/`footer`.
4. Implementera `reports` modul + UI för att välja ansökningar och generera rapport (DOCX).
5. Lägg till enhetstester för `rich_editor::markdown::to_html` och `export::markdown_to_docx`.
6. CI: workflow för build + test + cross-build.

---

## Öppna frågor / beslutsområden att diskutera
- Ska vi bunta PDF-konverterare i applikationen eller bidra med tydliga instruktioner för användaren att installera externt verktyg?
- Vilket mallspråk/templatesystem känns rätt (simpelt string-interpolation vs. `tera`/`handlebars`)?
- Hur mycket flerspråkstöd krävs i 1.0 (endast UI vs. även templates/rapporter)?

---

Om du vill kan jag: 
- skapa en första konkret TODO-PR-plan (de enskilda issues med filer och förslag på ändringar), 
- eller skriva en konkret implementation för t.ex. `Ny ansökan` och enkla tests för exportsteg.

Vill du att jag börjar med en kort PR-specifikation för "Ny ansökan"-funktionen?