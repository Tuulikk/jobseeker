# Jobseeker 🕵️‍♂️ - Gnaga sig till drömjobbet

![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)
![Build Status](https://github.com/Gnaw-Software/Jobseeker/actions/workflows/rust.yml/badge.svg)

Jobseeker är ett kraftfullt och integritetsfokuserat verktyg för att automatisera och strukturera ditt jobbsökande. Det är byggt enligt **"Gnag"-filosofin**: att gnaga sig igenom tråkiga, repetitiva uppgifter (som att leta annonser och kopiera texter) för att frigöra tid till det som faktiskt betyder något.

![Jobseeker Screenshot](screenshots/First.png)

## Varför Jobseeker?

Att söka jobb kan vara ett heltidsarbete i sig. Jobseeker agerar som en kognitiv förlängning som hjälper dig att:
- **Hitta rätt:** Filtrera bort bruset och fokusera på annonser som faktiskt matchar din profil.
- **Spara tid:** Automatisera insamling av data från Arbetsförmedlingen.
- **Skapa kvalitet:** Skriv dina ansökningar i en miljö fokuserad på skrivande, med verktyg för att exportera proffsiga dokument.

## Nyckelfunktioner

- **🤖 AI-Rankning (Ollama):** Kör en lokal AI (t.ex. Llama 3) som betygsätter annonser (1-10) mot din profil. Ingen data lämnar din dator.
- **📄 Proffsiga Ansökningar:** Inbyggd editor med stöd för att exportera dina personliga brev direkt till **PDF** och **Word (.docx)**.
- **🔍 Smart Sökning:** Sök i flera geografiska områden samtidigt med prioriteringsnivåer (P1, P2, P3).
- **🚫 Avancerad Svartlistning:** Slipp se annonser från specifika företag eller med nyckelord du inte är intresserad av.
- **📋 Rapporteringshjälp:** Snabbknappar för att kopiera all info du behöver för din aktivitetsrapport till Arbetsförmedlingen.
- **🔒 Privat av design:** All data (annonser, utkast, inställningar) sparas lokalt i en supersnabb **RedB**-databas (skriven helt i Rust) och JSON-filer. Ingen extern databasmotor krävs.

## Kom igång

### Förutsättningar

1. **Rust:** Installeras via [rustup.rs](https://rustup.rs/).
2. **Ollama:** För AI-rankning, kör [Ollama](https://ollama.com/) lokalt.
   ```bash
   ollama pull llama3
   ```
3. **Systembibliotek:**
   - **Ubuntu/Debian:** `sudo apt install libsoup-3.0-dev libgtk-4-dev libadwaita-1-dev`
   - **Fedora:** `sudo dnf install libsoup3-devel gtk4-devel libadwaita-devel`
   - **Windows/MacOS:** Inga extra systembibliotek krävs vanligtvis för att bygga.

### Installation & Körning

```bash
git clone https://github.com/Gnaw-Software/Jobseeker.git
cd Jobseeker
cargo run --release
```

## Licens

Detta projekt är licensierat under **Mozilla Public License 2.0 (MPL-2.0)** – en licens som främjar öppen källkod men tillåter flexibilitet. Se [LICENSE](LICENSE) för detaljer.

---
*"Allting är relativt – men att slippa klippa och klistra är absolut bra."*
