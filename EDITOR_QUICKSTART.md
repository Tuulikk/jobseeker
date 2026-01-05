# 📝 Markdown Editor - Snabbguide

## ✨ NYA FUNKTIONER I EDITORN

Editorn har nu **fungerande Markdown-formatering**! Alla knappar gör nu något.

### 🎯 Verktygsfält

När du öppnar en ansökning ser du nu verktyg som **faktiskt fungerar**:

| Knapp | Vad den gör | Resultat |
|-------|-------------|----------|
| **B** | Infoga fetstil | `**text**` |
| **I** | Infoga kursiv | `*text*` |
| **H1** | Rubrik nivå 1 | `# Rubrik` |
| **H2** | Rubrik nivå 2 | `## Rubrik` |
| **H3** | Rubrik nivå 3 | `### Rubrik` |
| **• Lista** | Punktlista | `- Punkt 1\n- Punkt 2` |
| **1. Lista** | Numrerad lista | `1. Första\n2. Andra` |
| **Infoga företag** | Lägger till företagsnamn från annonsen | `[Företagsnamn]` |
| **Klistra in profil** | Lägger till din profil från inställningar | Din bakgrundstext |

### 📖 Markdown Syntax

Du skriver i Markdown-format, som sedan konverteras till professionell formatering vid export.

#### Grundläggande formatering:
```markdown
**Fetstil text**
*Kursiv text*
# Stor rubrik (H1)
## Medelstor rubrik (H2)
### Liten rubrik (H3)
```

#### Listor:
```markdown
- Punkt ett
- Punkt två
- Punkt tre

1. Första punkten
2. Andra punkten
3. Tredje punkten
```

#### Länkar:
```markdown
[Länktext](https://example.com)
```

#### Citat:
```markdown
> Detta är ett citat
```

### 📄 Exempel på komplett ansökan

```markdown
# Ansökan - Senior Utvecklare

**Datum:** 2024-12-27  
**Till:** Tech Innovations AB

Hej,

Jag skriver för att uttrycka mitt *stora* intresse för tjänsten som **Senior Utvecklare**.

## Min bakgrund

Med över 5 års erfarenhet inom systemutveckling har jag:

- Designat och implementerat högpresterande backend-system
- Arbetat med Rust, Python och Go
- Lett utvecklingsteam på 5 personer
- Bidragit till flera open source-projekt

## Varför Tech Innovations?

Ert fokus på innovation och hållbar teknik matchar mina värderingar perfekt.

## Kontaktinformation

Med vänliga hälsningar,

**Ditt Namn**  
Email: din@email.com  
Tel: 070-123 45 67
```

### 💾 Export

#### **Exportera PDF** (egentligen HTML)
1. Klicka "Exportera PDF"
2. Välj var du vill spara (sparas som `.html`)
3. Öppna HTML-filen i din webbläsare
4. Tryck `Ctrl+P` eller `Cmd+P`
5. Välj "Spara som PDF" som destination
6. **Färdigt!** Du har en professionellt formaterad PDF

**Varför HTML först?**  
HTML med CSS-styling ger bäst resultat. Modern webbläsare har utmärkt PDF-export som bevarar all formatering perfekt.

#### **Exportera Word**
1. Klicka "Exportera Word"
2. Välj var du vill spara
3. **Färdigt!** `.docx` fil skapas med all formatering
4. Öppna i Word/LibreOffice för vidare redigering

**Vad konverteras:**
- ✅ Rubriker (H1, H2, H3) → Word Heading styles
- ✅ **Fetstil** → Bold formatting
- ✅ *Kursiv* → Italic formatting
- ✅ Listor → Bullet/Numbered lists
- ✅ Stycken → Proper paragraph spacing

### 🎨 Så ser det ut i exporten

**I editorn skriver du:**
```markdown
# Ansökan

**Till:** Företaget AB

Jag har följande kompetenser:

- Python
- Rust
- Linux
```

**Word/PDF visar:**
```
╔════════════════════════════════════╗
║  Ansökan (stor, fet rubrik)        ║
║                                    ║
║  Till: Företaget AB (fetstil)      ║
║                                    ║
║  Jag har följande kompetenser:     ║
║                                    ║
║  • Python                          ║
║  • Rust                            ║
║  • Linux                           ║
╚════════════════════════════════════╝
```

### ⌨️ Tips & Tricks

1. **Börja med en mall**  
   Använd "Klistra in profil" för att få din bakgrund direkt i dokumentet

2. **Använd rubriker**  
   Strukturera med H1 för titel, H2 för sektioner

3. **Formatera sparandes**  
   Markdown sparas automatiskt - ingen risk att förlora text

4. **Testa exporten**  
   Exportera och kolla hur det ser ut innan du skickar

5. **Markdown är standard**  
   Samma syntax används på GitHub, Reddit, Discord, Slack, etc.

### 🆘 Vanliga frågor

**Q: Varför ser jag `**text**` istället för fetstil?**  
A: Det är Markdown-syntax. Det konverteras till fetstil när du exporterar.

**Q: Kan jag använda vanlig text utan Markdown?**  
A: Ja! Vanlig text fungerar utmärkt. Markdown är frivilligt.

**Q: Vad händer om jag skriver fel Markdown?**  
A: Ingenting farligt! Fel syntax visas som vanlig text i exporten.

**Q: Kan jag byta tillbaka till hur det var förut?**  
A: Den gamla editorn var samma (plain text). Nu får du bara bonus-features!

**Q: Måste jag memorera alla kommandon?**  
A: Nej! Använd knapparna i verktygsfältet.

### 📚 Lär dig mer om Markdown

- GitHub Markdown Guide: https://guides.github.com/features/mastering-markdown/
- Markdown Cheatsheet: https://www.markdownguide.org/cheat-sheet/

### 🎯 Kom igång NU!

1. Klicka "Skriv ansökan" på en jobbannons
2. Tryck på **H1** knappen
3. Skriv "Ansökan - [Tjänst]"
4. Tryck på **• Lista** knappen  
5. Lägg till dina kompetenser
6. Klicka "Exportera Word"
7. **Klart!**

---

**Pro tip:** Spara denna fil och ha den framme första gången du använder editorn!