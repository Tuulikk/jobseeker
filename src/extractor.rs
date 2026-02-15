use std::path::Path;
use std::io::Read;

pub fn extract_text(path: &Path) -> Result<String, String> {
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "pdf" => extract_pdf(path),
        "docx" => extract_docx(path),
        "odt" => extract_odt(path),
        "txt" | "md" => std::fs::read_to_string(path).map_err(|e| e.to_string()),
        _ => Err(format!("Formatet .{} stöds inte ännu", extension)),
    }
}

fn extract_pdf(path: &Path) -> Result<String, String> {
    #[cfg(not(target_os = "android"))]
    {
        pdf_extract::extract_text(path).map_err(|e| e.to_string())
    }
    #[cfg(target_os = "android")]
    {
        Err("PDF-extrahering stöds inte på Android ännu".to_string())
    }
}

fn extract_docx(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    
    // DOCX lagrar texten i word/document.xml
    let mut xml_content = String::new();
    archive.by_name("word/document.xml")
        .map_err(|_| "Kunde inte hitta word/document.xml i DOCX-filen")?
        .read_to_string(&mut xml_content)
        .map_err(|e| e.to_string())?;

    // Väldigt enkel XML-strippning för att få ut brödtexten
    Ok(strip_xml_tags(&xml_content))
}

fn extract_odt(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    
    // ODT lagrar texten i content.xml
    let mut xml_content = String::new();
    archive.by_name("content.xml")
        .map_err(|_| "Kunde inte hitta content.xml i ODT-filen")?
        .read_to_string(&mut xml_content)
        .map_err(|e| e.to_string())?;

    Ok(strip_xml_tags(&xml_content))
}

fn strip_xml_tags(xml: &str) -> String {
    let mut result = String::new();
    let mut inside_tag = false;
    let mut last_was_tag_end = false;

    for c in xml.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
            last_was_tag_end = true;
        } else if !inside_tag {
            // Lägg till mellanrum efter vissa taggar för att inte klistra ihop ord
            if last_was_tag_end {
                result.push(' ');
                last_was_tag_end = false;
            }
            result.push(c);
        }
    }
    
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub struct MonthStat {
    pub month: String,
    pub count: usize,
}

pub fn get_monthly_stats(ads: &[crate::models::JobAd]) -> Vec<MonthStat> {
    use std::collections::BTreeMap;
    let mut stats = BTreeMap::new();

    for ad in ads {
        if ad.status == Some(crate::models::AdStatus::Applied) {
            let month = ad.publication_date.chars().take(7).collect::<String>(); // "YYYY-MM"
            *stats.entry(month).or_insert(0) += 1;
        }
    }

    stats.into_iter()
        .map(|(month, count)| MonthStat { month, count })
        .collect()
}

