#!/bin/bash
# Automated Portable Windows Build Script for Jobseeker
# This script builds a portable Windows version that can run from USB

set -e  # Exit on error

echo "======================================"
echo "  Jobseeker Portable Windows Builder"
echo "======================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're on Linux or Windows
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "${GREEN}Detected Linux - will cross-compile for Windows${NC}"
    CROSS_COMPILE=true
    TARGET="x86_64-pc-windows-gnu"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    echo -e "${GREEN}Detected Windows - will compile natively${NC}"
    CROSS_COMPILE=false
    TARGET="x86_64-pc-windows-msvc"
else
    echo -e "${YELLOW}Unknown OS - assuming Linux cross-compile${NC}"
    CROSS_COMPILE=true
    TARGET="x86_64-pc-windows-gnu"
fi

# Ensure target is installed
echo "Checking Rust target: $TARGET"
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "Installing target $TARGET..."
    rustup target add $TARGET
fi

# Check for cross-compile dependencies on Linux
if [ "$CROSS_COMPILE" = true ]; then
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo -e "${RED}Error: MinGW compiler not found!${NC}"
        echo "Install with: sudo dnf install mingw64-gcc mingw64-gcc-c++"
        exit 1
    fi
fi

# Clean old builds
echo ""
echo "Cleaning old portable builds..."
rm -rf portable-windows
rm -f jobseeker-portable-windows.zip

# Build the Windows executable
echo ""
echo -e "${GREEN}Building Jobseeker for Windows...${NC}"
echo "Target: $TARGET"
echo "This may take a few minutes..."
echo ""

cargo build --release --target $TARGET

# Check if build succeeded
if [ ! -f "target/$TARGET/release/Jobseeker.exe" ]; then
    echo -e "${RED}Build failed! Executable not found.${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"

# Get executable size
EXEC_SIZE=$(du -h "target/$TARGET/release/Jobseeker.exe" | cut -f1)
echo "Executable size: $EXEC_SIZE"

# Create portable directory structure
echo ""
echo "Creating portable package..."
mkdir -p portable-windows

# Copy executable
cp "target/$TARGET/release/Jobseeker.exe" portable-windows/

# Create default settings.json
cat > portable-windows/settings.json << 'EOF'
{
  "keywords": "",
  "blacklist_keywords": "",
  "locations_p1": "",
  "locations_p2": "",
  "locations_p3": "",
  "my_profile": "",
  "ollama_url": "http://localhost:11434/v1"
}
EOF

# Create README.txt
cat > portable-windows/README.txt << 'EOF'
================================================================================
                    JOBSEEKER - PORTABLE EDITION
================================================================================

VERSION: 0.2.0
BUILD DATE: $(date +%Y-%m-%d)

DESCRIPTION:
------------
Jobseeker är en modern jobbsökningsapplikation för den svenska arbetsmarknaden.
Detta är en portabel version som kan köras direkt från USB-minne utan installation.

SNABBSTART:
-----------
1. Dubbelklicka på Jobseeker.exe
2. Gå till fliken "Inställningar"
3. Fyll i dina sökord och önskade geografiska områden
4. Klicka på "💾 Spara inställningar"
5. Gå tillbaka till "Inbox" och klicka på knapparna 1, 2 eller 3 för att söka

FILER I DENNA MAPP:
-------------------
- Jobseeker.exe      : Huvudprogrammet
- settings.json      : Dina inställningar (skapas automatiskt)
- jobseeker.db       : Din jobbdatabas (skapas automatiskt vid första körning)
- README.txt         : Denna fil

ANVÄNDNING:
-----------
• INBOX: Visa och hantera jobbannonser
  - Filtrera: Alla, Bokmärken, Toppen, Sökta
  - Navigera: Klicka på jobb i listan för att visa detaljer
  - Status: Markera jobb som Nej/Spara/Toppen/Har Sökt

• UTKAST: Hantera ansökningar
  - Skriv personliga brev med Markdown-formatering
  - Exportera till Word (.docx) eller PDF (via HTML)
  - Spara automatiskt

• INSTÄLLNINGAR:
  - Sökord: Kommaseparerad lista (t.ex. "it, support, utvecklare")
  - Svartlista: Ord som filtrerar bort annonser
  - Områden 1-3: Geografiska områden med prioritet
  - Min profil: Din standardtext för ansökningar

MARKDOWN-FORMATERING:
---------------------
I editor kan du använda Markdown:
  **fetstil**        → fetstil
  *kursiv*           → kursiv
  # Rubrik 1         → Stor rubrik
  ## Rubrik 2        → Mellan rubrik
  - Listpunkt        → Punktlista
  1. Numrerad        → Numrerad lista
  [text](url)        → Länk

SYSTEMKRAV:
-----------
- Windows 10 eller senare (64-bit)
- Ingen installation krävs
- Inga administratörsrättigheter behövs
- ~100 MB ledigt utrymme för databas

PORTABILITET:
-------------
• Alla dina data sparas i denna mapp
• Kopiera hela mappen till USB-minne för att använda på andra datorer
• Säkerhetskopiera jobseeker.db regelbundet - den innehåller all din data

TIPS:
-----
• Använd Område 1 för dina högst prioriterade orter
• Svartlistan är användbar för att filtrera bort irrelevanta jobb
• Exportera ansökningar till Word innan du skickar dem
• Databas växer med tiden - rensa gamla jobb ibland

FELSÖKNING:
-----------
Problem: "Windows skyddade din dator"
Lösning: Klicka "Mer info" → "Kör ändå"
         (Programmet är osignerat men säkert)

Problem: Inställningar sparas inte
Lösning: Kontrollera att USB-minnet inte är skrivskyddat

Problem: Långsam prestanda från USB
Lösning: Använd USB 3.0-minne eller kopiera mappen till lokal disk

Problem: Databasen är låst
Lösning: Stäng alla instanser av programmet och försök igen

DATASKYDD:
----------
• Alla data sparas lokalt på ditt USB-minne
• Ingen data skickas till utvecklaren
• API-anrop går endast till Arbetsförmedlingens öppna API
• Din integritet är skyddad

LICENS:
-------
Se källkodsrepository för licensinformation.
Utvecklad av Gnaw Software.

SUPPORT:
--------
GitHub: https://github.com/Gnaw-Software/Jobseeker
Issues: https://github.com/Gnaw-Software/Jobseeker/issues

UPPDATERINGAR:
--------------
Ladda ner senaste versionen från GitHub Releases.
Kopiera bara över Jobseeker.exe - dina data bevaras automatiskt.

================================================================================
                        Lycka till med jobbsökningen!
================================================================================
EOF

# Create optional launcher batch file
cat > portable-windows/start.bat << 'EOF'
@echo off
title Jobseeker - Jobbsökningsassistent
echo =====================================
echo   Startar Jobseeker...
echo =====================================
echo.
Jobseeker.exe
if errorlevel 1 (
    echo.
    echo Fel vid start av Jobseeker!
    echo Tryck på valfri tangent för att avsluta...
    pause > nul
)
EOF

# Create version info file
cat > portable-windows/VERSION.txt << EOF
Jobseeker Portable Edition
Version: 0.2.0
Build Date: $(date +"%Y-%m-%d %H:%M:%S")
Target: $TARGET
Built on: $(uname -s) $(uname -m)
EOF

echo -e "${GREEN}Portable package created successfully!${NC}"
echo ""

# Calculate total size
TOTAL_SIZE=$(du -sh portable-windows | cut -f1)
echo "Package contents:"
echo "  - Jobseeker.exe (${EXEC_SIZE})"
echo "  - settings.json"
echo "  - README.txt"
echo "  - start.bat"
echo "  - VERSION.txt"
echo ""
echo "Total size: ${TOTAL_SIZE}"

# Create ZIP archive
echo ""
echo "Creating ZIP archive..."
if command -v zip &> /dev/null; then
    zip -r jobseeker-portable-windows.zip portable-windows/ > /dev/null
    ZIP_SIZE=$(du -h jobseeker-portable-windows.zip | cut -f1)
    echo -e "${GREEN}ZIP archive created: jobseeker-portable-windows.zip (${ZIP_SIZE})${NC}"
else
    echo -e "${YELLOW}Warning: 'zip' command not found - skipping ZIP creation${NC}"
    echo "Install with: sudo dnf install zip"
fi

# Summary
echo ""
echo "======================================"
echo -e "${GREEN}BUILD COMPLETE!${NC}"
echo "======================================"
echo ""
echo "Portable package location:"
echo "  → portable-windows/"
echo ""
echo "To use:"
echo "  1. Copy 'portable-windows' folder to USB drive"
echo "  2. Run Jobseeker.exe on any Windows 10+ computer"
echo "  3. No installation needed!"
echo ""
echo "To distribute:"
echo "  → Use jobseeker-portable-windows.zip"
echo ""
echo -e "${YELLOW}Note: Windows may show 'Unknown publisher' warning${NC}"
echo "      This is normal for unsigned executables."
echo "      Users should click 'More info' → 'Run anyway'"
echo ""
echo "Happy job hunting! 🎯"
