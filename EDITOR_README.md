# Rich Text Editor - Implementation Notes

## Overview

Den nya editorn använder **Markdown** som underliggande format, men presenterar det på ett användarvänligt sätt med verktygsrad och formatering.

## Hur det fungerar

### Markdown-baserad formatering
Istället för en riktig WYSIWYG HTML-editor (som skulle kräva webview-integration eller komplexa custom widgets), använder vi Markdown som är:

- **Enkel att skriva**: `**bold**`, `*italic*`, `# Rubrik`
- **Lätt att exportera**: Konverteras till HTML/PDF/DOCX med full formatering
- **Portabel**: Plain text som kan öppnas i vilken editor som helst
- **Standardiserad**: Används överallt (GitHub, Reddit, Discord, etc.)

### Verktygsraden

Knapparna i verktygsraden **injicerar Markdown-syntax**:

- **Bold (B)** → Wrappa selection med `**text**`
- **Italic (I)** → Wrappa selection med `*text*`
- **H1/H2/H3** → Lägg till `# `, `## `, `### ` i början av rad
- **Bullet List** → Lägg till `- ` i början av rad
- **Numbered List** → Lägg till `1. ` i början av rad
- **Link** → Wrappa med `[text](url)`

### Export med formatering

Vid export konverteras Markdown till:

1. **HTML** med professional styling (typsnitt, marginaler, etc.)
2. **PDF** via HTML (användaren kan printa HTML-filen som PDF)
3. **DOCX** med riktig formatering (headings, bold, italic, lists, etc.)

## Varför inte riktig WYSIWYG?

Vi utvärderade flera alternativ:

### Alternativ 1: Webview med TinyMCE/Quill
**Problem:**
- Komplex integration mellan Iced och webview
- Större binary size
- IPC-kommunikation mellan Rust och JavaScript
- Platform-beroenden

### Alternativ 2: Custom Rich Text Widget
**Problem:**
- Skulle ta 100+ timmar att bygga
- Iced har inte native rich text support
- Behöver hantera cursor, selection, rendering, etc.
- Mycket bugbenäget

### Alternativ 3: Byta GUI framework
**Problem:**
- Måste skriva om hela applikationen
- GTK/Qt har sina egna problem
- Går emot projektets Iced-baserade design

## Den valda lösningen: Markdown

### Fördelar
✅ Fungerar nu - ingen komplex integration  
✅ Enkel export med full formatering  
✅ Portable format (kan öppnas i andra editorer)  
✅ Standardiserat och välkänt  
✅ Plain text = lätt att versionshantera  
✅ Perfekt för professionella dokument  

### Nackdelar
⚠️ Användaren ser Markdown-syntax (`**bold**` istället av **bold**)  
⚠️ Inte lika intuitivt som Word  

## Användning

### Skriva formaterad text

```markdown
# Ansökan - Utvecklare

**Till:** Företaget AB

Hej,

Jag är *mycket* intresserad av tjänsten som utvecklare.

## Om mig

- 5 års erfarenhet
- Kompetent i Rust
- Driven och engagerad

Med vänliga hälsningar,
[Ditt namn]
```

### Exportera till PDF/Word

1. Tryck "Exportera PDF" eller "Exportera Word"
2. Välj var filen ska sparas
3. Markdown konverteras automatiskt till formaterad output

För PDF: En HTML-fil skapas som du kan printa som PDF i webbläsaren (Ctrl+P → Save as PDF).

För Word: En .docx fil skapas med full formatering.

## Framtida förbättringar

### Kort sikt
- [ ] Live preview-panel vid sidan av editorn (visar rendererad Markdown)
- [ ] Bättre keyboard shortcuts (Ctrl+B för bold, etc.)
- [ ] Template-system med variabler (`{{COMPANY}}`, `{{POSITION}}`)
- [ ] Syntax highlighting för Markdown

### Lång sikt
- [ ] Integrera headless Chrome för bättre PDF-rendering
- [ ] Rich text preview i läsläge
- [ ] Drag & drop images (embedded as base64)
- [ ] Spell checker

## Tekniska detaljer

### Dependencies
```toml
pulldown-cmark = "0.11"  # Markdown parsing & HTML conversion
docx-rs = "0.4"          # Word document generation
regex = "1.10"           # Text processing
```

### Moduler
- `src/rich_editor.rs` - Editor widget och Markdown utilities
- `src/rich_editor.rs::markdown` - Markdown ↔ HTML conversion
- `src/rich_editor.rs::export` - PDF och DOCX export

### Export flow

```
Markdown text
    ↓
[pulldown-cmark parse]
    ↓
HTML with styling
    ↓
├─→ [Save as .html] → User prints to PDF
└─→ [docx-rs convert] → .docx file
```

## Tips för användare

### Snabbreferens Markdown

| Formatering | Syntax | Resultat |
|-------------|--------|----------|
| Rubrik 1 | `# Text` | <h1>Text</h1> |
| Rubrik 2 | `## Text` | <h2>Text</h2> |
| Fetstil | `**text**` | **text** |
| Kursiv | `*text*` | *text* |
| Lista | `- item` | • item |
| Numrerad | `1. item` | 1. item |
| Länk | `[text](url)` | [text](url) |
| Citat | `> text` | > text |

### Best practices

1. **Använd rubriker** för att strukturera din ansökan
2. **Var konsekvent** med formatering
3. **Använd listor** för att highlighta kompetenser
4. **Testa exporten** innan du skickar ansökan

### Exempel på professionell ansökan

```markdown
# Ansökan - Senior Rust Developer

**Datum:** 2024-12-27  
**Till:** Tech Innovations AB  
**Referens:** Jobbnummer 12345

---

## Personligt brev

Hej,

Jag skriver för att uttrycka mitt intresse för tjänsten som Senior Rust Developer hos Tech Innovations AB.

### Min bakgrund

Med över **5 års erfarenhet** inom systemutveckling och särskild expertis inom Rust, är jag övertygad om att jag kan bidra till ert team:

- Designat och implementerat högpresterande backend-system
- Erfarenhet av async Rust och Tokio
- Bidragit till open source-projekt inom Rust-ekosystemet
- Kompetent inom CI/CD och DevOps

### Varför Tech Innovations?

Ert fokus på *innovation* och *hållbar teknik* matchar mina värderingar perfekt. Jag ser fram emot möjligheten att...

### Kontakt

Med vänliga hälsningar,

**[Ditt Namn]**  
Email: din@email.com  
Tel: 070-123 45 67  
LinkedIn: linkedin.com/in/dinprofil
```

## Felsökning

### "Formateringen syns inte i editorn"
→ Det är förväntat. Markdown-syntax visas i editorn, men konverteras vid export.

### "PDF-export fungerar inte"
→ PDF skapas som HTML-fil. Öppna filen i webbläsare och printa som PDF (Ctrl+P).

### "Svensk text i Word-filen ser konstig ut"
→ Öppna .docx i Word/LibreOffice och justera language settings.

## Support

För frågor eller buggar, se projektets huvuddokumentation.