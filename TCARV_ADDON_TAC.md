# Metodik: TCARV-TAC (Tool Architecture & Core) - Tilläggsmodul

**Aktiveras vid:** Utveckling av infrastrukturverktyg, CLI-applikationer, bibliotek, MCP-servrar och "headless" system.

Detta är en specialisering av TCARV 1.0. Medan grundmetodiken fokuserar på logisk verifiering och mänskligt värde, fokuserar TAC på **Systemets robusthet, determinism och integrerbarhet**.

## 1. Hypotesfasen (Kontraktet som Lag)

För ett verktyg är "Sanningen" inte en berättelse, utan ett **Kontrakt**.

*   **Handling:** Innan kod skrivs, definiera det publika gränssnittet.
    *   För CLI: Skriv `--help` texten och usage-exempel.
    *   För MCP: Definiera JSON-schemat för verktyget.
    *   För Lib: Skriv funktionssignaturen och doc-comments.
*   **Krav:** Kontraktet måste vara "Input/Output-komplett". Om du ger X ska du alltid få Y (eller Error Z). Inga dolda tillstånd.
*   **Syfte:** Agenter och scripts hatar överraskningar. Specifikationen måste vara rigid så att integrationer inte går sönder.

## 2. Kärn-byggande (Den Agnostiska Kärnan)

Det vanligaste felet i verktyg är att blanda logik med presentation (t.ex. `println!` mitt i en beräkningsfunktion).

*   **Core-principen:** Kärnan (`Core`) får ALDRIG veta *hur* den körs.
    *   Den vet inte om den är ett CLI-kommando.
    *   Den vet inte om den är en MCP-server.
    *   Den vet inte om den är en WASM-modul i en webbläsare.
*   **Renhet:** Kärnfunktioner tar in data och returnerar data (eller `Result`). De skriver aldrig till STDOUT/STDERR direkt och de läser aldrig `env::args` direkt.
*   **Testbarhet:** Du ska kunna testa 100% av logiken genom unit-tester i `lib.rs` utan att någonsin starta binären.

## 3. Skal-integration (The Shells)

Verktyget ("Appen") är bara ett tunt skal runt Kärnan. Vi tillåter flera skal för samma kärna.

*   **CLI-skalet:** Ansvarar *enbart* för att parsa text-argument, anropa Kärnan, och formatera svaret till terminal-output.
*   **MCP-skalet:** Ansvarar *enbart* för att parsa JSON, anropa Kärnan, och returnera JSON.
*   **Regel:** Om du fixar en bugg i logiken, ska du bara behöva ändra i Kärnan. Båda skalen ska automatiskt dra nytta av fixen.

## 4. Verifiering (Deterministisk Stabilitet)

I TCARV 1.0 verifierar vi mot "känslan". I TCARV-TAC verifierar vi mot "determinism".

*   **Idempotens:** Om verktyget körs två gånger med samma input, ska resultatet vara identiskt (eller ofarligt).
*   **Exit Codes:** Ett verktyg kommunicerar framgång/fel via statuskoder, inte bara text. Detta är avgörande för CI/CD och scripts.
*   **Failsafe:** Vid osäkerhet (t.ex. parsning misslyckas), rör ingenting. Det är bättre att krascha säkert än att korrupta en fil.

---

## Agent-Instruktioner för TCARV-TAC

🚫 **Agenten FÅR INTE:**
*   Lägga affärslogik direkt i `main.rs` eller `cli.rs`.
*   Använda `print!` eller `console.log` djupt nere i funktioner (använd logging/tracing eller returnera strängar).
*   Göra antaganden om användarens miljö (t.ex. att "editor" är installerad) i Kärnan.

✅ **Agenten SKA:**
*   **Börja med Interfacet:** "Om jag kör kommandot så här, vad exakt ska komma ut?"
*   **Refaktorera mot Core:** Om du ser logik i CLI-lagret, föreslå en flytt till `src/core/`.
*   **Tänka "Headless":** Föreställ dig alltid att din kod ska köras av en annan dator, inte en människa.

---

## Retroaktiv TAC (Legacy Mode)

För befintliga verktyg (som GnawTreeWriter):

1.  **Identifiera Läckage:** Hitta var `main.rs` gör för mycket (t.ex. läser filer, loopar över logik).
2.  **Extrahera:** Flytta logiken till `src/core/mod.rs` eller liknande.
3.  **Parameterisera:** Byt ut hårdkodade `println!` mot returvärden.
4.  **Verifiera:** Skapa ett nytt testfall som anropar den nya funktionen direkt, utan att gå via CLI.
