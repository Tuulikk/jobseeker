# agents.md

Syfte
-----
Detta dokument beskriver riktlinjer och säkerhetsregler för automatiska agenter (scripts, AI-agenter eller andra automatiserade processer) som får interagera med Jobseeker-databasen. Målet är att skydda personlig data, undvika oavsiktliga ändringar i produktionsdata och skapa en tydlig arbetsprocess för experiment och automatisering.

Grundprinciper
---------------
- Produktion (din "live"-databas) får aldrig ändras automatiskt utan explicit, uttryckligt medgivande från dig. All förändring måste föregås av:
  1. Backup (timestamped copy)
  2. En tydlig "dry-run" eller verifierbar testkörning
  3. Godkännande från dig
- Allt automatiserat arbete ska vara reversibelt: skapar alltid backup, och loggar alla ändringar.
- Experiment och tester sker i en separat databas (test-miljö) -- aldrig direkt i produktionsdatabasen.

Plats och namngivning
---------------------
- Produktionsdatabas (per användare): (standard) `~/.local/share/jobseeker/jobseeker.db`
- Lokal/portable DB (om körd i projektkatalog): `./jobseeker.db` (används ej för produktion)
- Test-/experiment-DB (rekommenderat): `~/jobseeker-test.db` eller valfri sökväg satt via miljövariabel.
- Exportkatalog: `~/.local/share/jobseeker/exports/`
- Backups: samma katalog som DB (exempel: `jobseeker.db.bak.<timestamp>` eller `jobseeker.db.mergepre.bak.<timestamp>`)

Konfiguration / testkörning
--------------------------
- För att köra appen mot en test-DB:
  - Unix (bash/zsh): `export JOBSEEKER_DB_PATH="$HOME/jobseeker-test.db" && ./target/release/Jobseeker`
  - Alternativt skapa ett skript `scripts/run-with-test-db.sh` som sätter `JOBSEEKER_DB_PATH` och kör appen.
- `JOBSEEKER_DB_PATH` är det primära sättet att dirigera en agent eller utvecklingskörning till en icke-produktionsdatabas.

Agentpolicy (vad agenter får/inte får göra)
-------------------------------------------
1. Inga automatiska ändringar i produktion utan:
   - Backup (kopiera filen, spara med timestamp).
   - En human-review eller "explicit consent" flagg (t.ex. `--yes` eller användarbekräftelse).
   - Ett audit-loggmeddelande (se loggkravet nedan).
2. Alla automatiska förändringar ska:
   - Köras först i test-DB.
   - Inkludera en verifieringsfas (t.ex. räkna poster, jämföra ID-set).
   - Skriva ut en export (CSV/JSON) och presentera den för användaren före swap.
3. Agenten får läsa produktion (t.ex. skapa export) men:
   - Måste använda read-only transaktion om möjligt.
   - Ska aldrig skriva till produktion utan backup + explicit bekräftelse.
4. Experiment:
   - Använd alltid `JOBSEEKER_DB_PATH` för att styra experiment-Databasen.
   - Sätt upp testdata och kör inga batch-ändringar i produktion.

Audit & logging
---------------
- Alla ändringsoperationer (patches, merges, rensningar) måste loggas i en audit-logg:
  - Fil: `~/.local/share/jobseeker/agent-audit.log`
  - Format (förslag): `YYYY-MM-DD HH:MM:SS | <agent-name> | <action> | <target-db> | <ids> | <backup-file>`
- Backuper sparas med tidsstämpel i samma katalog som DB:n (`<db>.bak.<timestamp>`).
- Exporter (dagliga/manuel): sparas under `exports/` med filnamn `applied-YYYYMMDD-HHMMSS.csv`. Behåll `latest.csv` som snabbreferens.

Migration & merge-policy
------------------------
- Migration från SQLite till Redb är slutförd och de automatiska migreringsverktygen för detta har tagits bort från repot. Om du fortfarande har en äldre SQLite-databas och behöver migrera den, kontakta projektets underhållare eller använd ett bevarat migreringsskript från projektets historik efter manuell granskning.
- Merge-policy (fortsatt):
  1. Kör inspektion (räkna poster, verifiera kritiska ID-set).
  2. Skapa backup av original innan någon swap.
  3. Verifiera och jämför resultatet noggrant innan produktionen uppdateras.
- Aldrig automatisk swap utan explicit användarkonfirmering (backup och verifiering krävs alltid).

Daglig export (policy och teknisk lösning)
------------------------------------------
- Daglig export genereras och sparas i `~/.local/share/jobseeker/exports/`.
- Exportens syfte: snapshot för rapportering och spårbarhet.
- Exportskript ska:
  - Samla poster med `status=4` (applied) eller `applied_at` satt,
  - Generera en CSV `applied-YYYYMMDD-HHMMSS.csv`,
  - Uppdatera `latest.csv`,
  - Istället för att alltid skriva, jämför filinnehåll mot senaste och skriv endast om innehållet ändrats (för att undvika dubbletter med bara tidsstämpel).
- Vi kan installera en systemd user-timer eller cron-jobb för den här exporten (jag kan hjälpa sätta upp det).

Åtgärd vid misstänkt korruption eller felaktig data
---------------------------------------------------
1. Stoppa GUI/appen (minskar låsproblem).
2. Skapa backup av aktuell DB (`jobseeker.db.bak.<ts>`).
3. Kör export från aktuell DB och från eventuella andra DB-källor (t.ex. `~/jobseeker.db`, `jobseeker.db.sqlite.bak.*`).
4. Jämför ID-set och tidsstämplar (särskilt viktiga månader).
5. Om en källa är korrekt (du bekräftar), merge → uppdatera produktion (med backup).
6. Dokumentera allt i audit-loggen.

Policy för AI-agenter
---------------------
- AI får *inte* skriva till produktion utan ett explicit steg från användaren (textuell bekräftelse räcker ej — kräver en bekräftelse-kommando/flag).
- AI får föreslå ändringar, skapa CSV/rapporter och föreslå merges, men inte automatiskt utföra dem.
- Varje automatiskt förslag från AI ska innehålla:
  - Exakt diff (före/efter) för de poster som påverkas
  - En backup-sökväg
  - En tydlig roll-back-kommando
- AI som används i experimentsyfte ska alltid användas mot test-DB.

Praktiska exempel (snabbkommandon)
----------------------------------
- Köra app mot test-DB:
  - `JOBSEEKER_DB_PATH=$HOME/jobseeker-test.db ./target/release/Jobseeker`
- Manuellt skapa backup:
  - `cp ~/.local/share/jobseeker/jobseeker.db ~/.local/share/jobseeker/jobseeker.db.bak.$(date +%s)`
- Exportera (manuellt):
  - `~/.local/bin/jobseeker-daily-export` (om installerad)
- Återställ från backup:
  - `mv jobseeker.db jobseeker.db.corrupt && mv jobseeker.db.bak.<ts> jobseeker.db`

Samarbete & förändringsprocess
-------------------------------
- Alla förändringar i policy eller automatisering dokumenteras i `agents.md`.
- Om du vill lägga till en ny automatisk agent eller ge en agent behörighet att ändra data: skriv en PR/issue som beskriver:
  - Vad agenten ska göra,
  - Backup- och rollback-plan,
  - Testplan och hur man verifierar resultaten,
  - Var audit-loggen sparas för ändringen.

Slutord
-------
- Huvudregeln: aldrig göra automatiska ändringar i produktionsdata utan backup och uttryckligt godkännande.
- Jag hjälper gärna till att sätta upp testmiljö, schemaläggning och dokumentation (inkl. `agents.md`) och att göra en sista kontroll tillsammans innan vi låser automatiken.
