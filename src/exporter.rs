use genpdf::Element;
use genpdf::elements;
use genpdf::fonts;

pub fn export_doc_to_pdf(name: &str, content: &str, path: &std::path::Path) -> Result<(), String> {
    // Försök hitta en font på systemet
    let font_path = std::path::Path::new("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
    let font_family = if font_path.exists() {
        fonts::from_files("/usr/share/fonts/truetype/dejavu", "DejaVuSans", None)
            .map_err(|e| format!("Font error: {}", e))?
    } else {
        // Fallback: Detta kommer troligen faila om vi inte har en font fil,
        // men genpdf kräver en. I en riktig app skickar vi med fonten som asset.
        return Err("Kunde inte hitta en standardfont (DejaVuSans.ttf) på systemet för PDF-export.".to_string());
    };

    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(name);
    
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    // Lägg till rubrik
    doc.push(elements::Paragraph::new(name).styled(genpdf::style::Color::Rgb(0, 0, 0)).styled(genpdf::style::Effect::Bold));
    doc.push(elements::Break::new(1));

    // Lägg till brödtext (väldigt enkel hantering av rader)
    for line in content.lines() {
        doc.push(elements::Paragraph::new(line));
    }

    doc.render_to_file(path).map_err(|e| e.to_string())
}

pub fn export_doc_to_md(content: &str, path: &std::path::Path) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}