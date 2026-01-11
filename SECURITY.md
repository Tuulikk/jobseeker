# Säkerhetspolicy 🛡️

## Vårt åtagande

Jobseeker är byggt med integritet som högsta prioritet. Vi strävar efter att aldrig skicka din personliga data till molnet för analys eller lagring.

## Lokalt först

- **AI-analys:** Sker via Ollama på din lokala maskin. Dina profiluppgifter eller annonstexter skickas aldrig till OpenAI, Anthropic eller andra molntjänster.
- **Datalagring:** Dina inställningar, sökord och ansökningsutkast sparas lokalt på din hårddisk i `settings.json` och `jobseeker.db`.
- **API-anrop:** Programmet kommunicerar endast med Arbetsförmedlingens officiella JobSearch API för att hämta annonser.

## Rapportera sårbarheter

Om du hittar en säkerhetsbrist i Jobseeker, vänligen öppna en "Issue" på GitHub eller kontakta oss direkt. Eftersom detta är ett lokalt verktyg är den största risken oftast relaterad till beroenden (dependencies) – vi försöker hålla dessa uppdaterade för att minimera risker.

---
*Ditt jobbsökande är din ensak.*
