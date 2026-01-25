# Roadmap — Jobseeker 2026

Det här dokumentet sammanfattar visionen och framstegen för Jobseeker. Vi har gått från en instabil prototyp till en robust, privat och "superior" applikation för jobbsökande.

## ✅ Slutförda Milstolpar (V0.2.x)

### Arkitektur & Stabilitet
- **Slint-konvertering:** Hela gränssnittet är nu byggt i Slint för maximal prestanda och cross-platform stöd.
- **RedB-databas:** Bytt till RedB för en blixtsnabb, offline-first upplevelse. All data sparas lokalt.
- **Trådsäkerhet:** Implementerat en robust Tokio-baserad motor som hanterar sökningar och statistik i bakgrunden utan att låsa UI:t.
- **Individuell sökning:** Optimerad söklogik som söker på varje nyckelord individuellt för att garantera 100% träffsäkerhet mot JobTech API.

### Gränssnitt (UI/UX)
- **Split-view:** Modern desktop-layout med lista till vänster och detaljer till höger.
- **DPI-skalning:** Anpassat typsnitt (12px+) och layouter för att fungera på högupplösta skärmar.
- **Global Statusrad:** Alltid synlig feedback för användaren vid sparning, kopiering och synk.

### Funktioner
- **Statistik-motor:** Detaljerad överblick över sökta, bokmärkta, intressanta och avvisade jobb per månad.
- **Napp-tracking:** Statistik över vilka sökord som faktiskt genererar flest annonser.
- **Export-system:** Rapportgenerering till Urklipp (med Linux-fix), E-post och lokal textfil (.txt).
- **Automatisk Synk:** Kontinuerlig backup av databasen till valfri mapp (Dropbox/Syncthing/Android-vänligt).

---

## 🚀 Nästa Steg (V0.3.0+)

### 📡 Utökad Synk & Moln
- **SFTP-synk:** Inbyggt stöd för att synka mot egen server för maximal integritet.
- **Konflikthantering:** Smartare hantering om databasen ändrats på flera enheter samtidigt.

### 🤖 AI-förbättringar (GnawSense)
- **Lokal Ranking:** Djupare integration med Ollama för att ranka annonser baserat på din profil.
- **Motivationsbrev:** Automatiskt generera utkast till ansökningar baserat på annonsens krav.

### 📄 Rapportering & PDF
- **PDF-generering:** Fullt stöd för formaterade PDF-rapporter med logotyp och snygg layout (kräver inbäddade typsnitt).
- **Excel/CSV-export:** För de som vill ha rådata för egna analyser.

---

## 🛠 Underhåll & Release
- **CI/CD:** Full automatisering via GitHub Actions (Windows, Linux, Android APK).
- **Release Tags:** Börja använda semantisk versionshantering och officiella releaser på GitHub.
- **Dokumentation:** Hålla `Overview.md` och `README.md` i synk med den tekniska verkligheten.
