# Jobseeker 🕵️‍♂️ - Gnaga sig till drömjobbet

![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)
![Build Status](https://github.com/Tuulikk/jobseeker/actions/workflows/build.yml/badge.svg)

Jobseeker är ett verktyg byggt enligt **"Gnag"-filosofin**: att gnaga sig igenom tråkiga, repetitiva uppgifter för att spara energi till det som faktiskt betyder något. Denna version är helt ombyggd i **Slint** för snabbhet och stabilitet.

> [!IMPORTANT]
> **Projektstatus:** Detta är V0.2 (Beta). Appen är nu stabil för daglig användning, men funktioner kan fortfarande tillkomma eller finjusteras. All data sparas privat i en lokal RedB-databas.

![Jobseeker Screenshot](screenshots/First.png)

## Varför Jobseeker?

Jobseeker är din kognitiva förlängning för att dominera jobbsökandet:
- **📦 Allt-i-ett Inkorg:** Samla annonser från JobTech (Arbetsförmedlingen) i en offline-inkorg. Inget mer klickande på sega webbsidor.
- **🔍 Smart Sökning:** Sök på dussintals nyckelord och geografiska zoner samtidigt. Appen aggregerar och deduplicerar allt åt dig.
- **📊 Statistik & Kontroll:** Se exakt hur många jobb du sökt denna månad, vilka sökord som ger napp och exportera rapporter med ett klick.
- **🔄 Automatisk Synk:** Stöd för kontinuerlig backup till valfri mapp (Dropbox, Syncthing, eller delade mappar på Android).
- **🔒 Privacy First:** Ingen molntjänst, ingen spårning. Din data bor hos dig.

## Funktioner

- **⚡ Blixtsnabbt UI:** Byggt i Rust + Slint. Startar direkt och flyter mjukt.
- **⭐ Prioritering:** Dela upp dina sökningar i P1 (Högst prio), P2 och P3 zoner.
- **📋 Export:** Generera aktivitetsrapporter till Urklipp, E-post eller fil på sekunder.
- **🚫 Svartlistning:** Filtrera automatiskt bort annonser du inte vill se.
- **🤖 AI-Klar:** Förberedd för integration med lokal AI (Ollama) för ranking av annonser.

## Kom igång

### Förutsättningar

1. **Rust:** Installeras via [rustup.rs](https://rustup.rs/).
2. **Systembibliotek:**
   - **Ubuntu/Debian:** `sudo apt install libsoup-3.0-dev libgtk-4-dev libadwaita-1-dev libxkbcommon-dev libfontconfig1-dev`
   - **Fedora:** `sudo dnf install libsoup3-devel gtk4-devel libadwaita-devel libxkbcommon-devel fontconfig-devel`
   - **Windows/MacOS:** Inga extra systembibliotek krävs.

### Installation & Körning

```bash
git clone https://github.com/Tuulikk/jobseeker.git
cd Jobseeker
cargo run --release
```

## Licens

Detta projekt är licensierat under **Mozilla Public License 2.0 (MPL-2.0)**. Se [LICENSE](LICENSE) för detaljer.

---
*"Allting är relativt – men att slippa klippa och klistra är absolut bra."*
