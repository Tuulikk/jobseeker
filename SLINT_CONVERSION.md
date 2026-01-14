# GUI Components & Functions - Complete List

## Global Navigation (Tab Bar)
- **Inbox** - Main job listings view
- **Utkast** - Saved application drafts
- **Inställningar** - App settings
- **Dynamic tabs** - Open application editors (can be closed)

---

## 1. Inbox Tab

### 1.1 Toolbar (top)
- **Område: 1** button - Search priority 1 locations
- **Område: 2** button - Search priority 2 locations
- **Område: 3** button - Search priority 3 locations
- **Chevron Left** - Previous job
- **Chevron Right** - Next job
- **Trash + "Töm"** - Clear non-bookmarked ads

### 1.2 Filter Bar
- **Alla** button - Show all ads
- **Bookmark icon + "Bokm."** - Show bookmarked
- **Thumbs Up + "Toppen"** - Show thumbs up rated
- **Check + "Sökta"** - Show applied jobs

### 1.3 Month Navigator
- **Chevron Left** - Previous month
- **Text** - Current "YYYY-MM"
- **Chevron Right** - Next month

### 1.4 Job List (sidebar)
Each job row shows:
- **Status icon** (circle-fill, bookmark, thumbs-up, check, trash)
- **Rating** - [X★] or [---]
- **Headline** - White if unread, dimmed if read
- **Employer** - Gray text
- **Date** - MM-DD format
- **Search keyword + "•" + Municipality** - Blue text

### 1.5 Job Details Panel
#### Actions Toolbar
- **Thumbs Down + "Nej"** - Mark as rejected
- **Bookmark + "Spara"** - Mark as bookmarked
- **Thumbs Up + "Toppen"** - Mark as thumbs up
- **Check + "HAR SÖKT"** - Mark as applied
- **Globe icon** - Open in browser
- **Envelope icon** - Send email
- **Clipboard icon** - Copy ad to clipboard

#### Status Info
- **Green box** - "SÖKT: YYYY-MM-DD HH:MM:SS" (if applied)
- **Blue text** - "Sparad: YYYY-MM-DD HH:MM:SS" (if bookmarked)

#### Report Buttons (only if Applied)
- **Typ** - Copy job type
- **Företag** - Copy company
- **Datum** - Copy date
- **Omf.** - Copy working hours
- **Kommun** - Copy municipality

#### Job Info
- **Headline** - Large white text
- **Employer** - Medium gray text
- **Publication date** - Gray text

#### Actions
- **"Betygsätt med AI"** button
- **"Skriv ansökan"** button - Opens application editor

#### Description
- Scrollable text area with full job description

---

## 2. Utkast (Drafts) Tab

### 2.1 Toolbar
- **Text** - "Mina sparade utkast"
- **Space**
- **"Uppdatera lista"** button - Reload drafts list

### 2.2 Draft List
Each draft item:
- **Headline** - White text
- **"Senast sparad: YYYY-MM-DD"** - Gray text
- **"Öppna →"** button - Opens application editor

If no drafts: **"Inga utkast sparade ännu."** (gray)

---

## 3. Inställningar (Settings) Tab

### 3.1 Sökord Section
- **Title** - "Sökord" (cyan)
- **Help text** - "Ange sökord separerade med kommatecken (t.ex. rust, python, support)"
- **Text editor** - Multi-line input for keywords

### 3.2 Svartlista Section
- **Title** - "Svartlista" (red)
- **Help text** - "Annonser med dessa ord i rubrik eller beskrivning döljs"
- **Text editor** - Multi-line input for blacklist keywords

### 3.3 Geografiska områden Section
- **Title** - "Geografiska områden" (blue)
- **Help text** - "Du kan nu skriva kommunnamn (t.ex. Helsingborg, Malmö) eller koder"

#### Område 1 (Högsta prioritet)
- **Label** - "Område 1 (Högsta prioritet)"
- **Text input** - Location codes or names

#### Område 2
- **Label** - "Område 2"
- **Text input** - Location codes or names

#### Område 3
- **Label** - "Område 3"
- **Text input** - Location codes or names

### 3.4 AI & Profil Section
- **Title** - "AI & Profil" (yellow)

#### Min bakgrund
- **Label** - "Min bakgrund (används för AI-matchning)"
- **Text editor** - Multi-line profile text

#### Ollama API URL
- **Label** - "Ollama API URL"
- **Text input** - "http://localhost:11434/v1"

---

## 4. Application Editor Tab

### 4.1 Toolbar
- **"Klar (Läs-läge)" / "Redigera"** toggle button
- **"Dölj verktyg" / "Visa verktyg"** toggle button
- **"Öppna fil"** button - Import .txt/.md file
- **"Exportera PDF"** button (green) - Save as PDF
- **"Exportera Word"** button (blue) - Save as .docx

### 4.2 Editor Tools (floating panel, when visible)
#### Text Formatting
- **Bold** - Ctrl+B
- **Italic** - Ctrl+I
- **Underline** - Ctrl+U

#### Alignment
- **Left align** button
- **Center** button
- **Right align** button
- **Justify** button

#### Quick Actions
- **"Klistra in profil"** button - Inserts profile text
- **"Infoga [Företag]"** button - Inserts company name from current job

### 4.3 Editor Area (edit mode)
- **White background** text editor
- **Placeholder** - "Skriv ditt personliga brev här..."
- **Dark background border**
- **Drop shadow**

### 4.4 Preview Area (read mode)
- **White background**
- **Scrollable** text display
- **Gray border**
- **If empty** - "Inget skrivet ännu. Tryck på Redigera för att börja."

### 4.5 Job Side Panel (left)
- **Headline** - Large white text
- **Employer** - Medium text
- **Horizontal line**
- **Description** - Full job description text

---

## Backend Functions to Connect

### Database (Db)
- `new(db_path)` - Initialize
- `save_job_ad(ad)` - Save job
- `get_filtered_jobs(statuses, year, month)` - Get jobs list
- `mark_as_read(id)` - Mark as read
- `update_ad_status(id, status)` - Update status
- `update_rating(id, rating)` - Update rating
- `save_application_draft(id, content)` - Save draft
- `get_application_draft(id)` - Load draft
- `get_all_drafts()` - Get all drafts
- `save_settings(settings)` - Save settings
- `load_settings()` - Load settings
- `clear_non_bookmarked()` - Clear ads

### API (JobSearchClient)
- `new()` - Create client
- `search(keyword, locations, limit)` - Search API
- `get_municipality_code(name)` - Get location code
- `get_municipality_name(code)` - Get location name

### AI (AiRanker)
- `new(ollama_url, api_key)` - Create ranker
- `rate_job(ad, profile)` - Rate job 0-10 stars

### File Operations
- **Import** - Open .txt/.md file dialog
- **Export PDF** - Save file dialog + genpdf rendering
- **Export Word** - Save file dialog + docx-rs rendering
- **Open in browser** - webbrowser::open()
- **Send email** - mailto link with job details

---

## Color Scheme (Iced Dark Theme)
- Background: Dark gray (~0.1, 0.1, 0.15)
- Text: White or light gray
- Accent Blue: (0.3, 0.6, 0.8)
- Accent Green: (0.0, 1.0, 0.0) / (0.1, 0.3, 0.1)
- Accent Yellow: (1.0, 1.0, 0.0)
- Accent Red: (0.8, 0.3, 0.3)
- Borders: (0.3, 0.3, 0.3)
- Selected item: (0.2, 0.2, 0.3)
- Tab active: (0.25, 0.3, 0.4) with blue border
- Tab inactive: (0.12, 0.14, 0.18)

---

## Icons Used (SVG)
- circle-fill - Unread
- bookmark-star-fill - Bookmarked
- hand-thumbs-up-fill - Thumbs up
- hand-thumbs-down-fill - Thumbs down / Rejected
- check-circle-fill - Applied
- trash3-fill - Clear / Rejected
- globe - Open in browser
- envelope-fill - Send email
- clipboard-plus-fill - Copy
- gear-fill - Settings
- inbox-fill - Inbox
- chevron-left - Previous
- chevron-right - Next
- type-bold - Bold
- type-italic - Italic
- type-underline - Underline
- text-left - Left align
- text-center - Center
- text-right - Right align
- justify - Justify

---

## Event Handling
- Keyboard shortcuts (Ctrl+B, Ctrl+I, Ctrl+U)
- Tab switching
- Tab closing (except Inbox/Settings/Drafts)
- Scroll events
- Click events for all buttons
- Text input changes (save to settings on change)
- Editor content changes (auto-save draft)
