use crate::db::Db;
use anyhow::Result;
use chrono::Utc;
use docx_rs::*;

/// Generate a DOCX report for selected job applications (drafts)
///
/// This creates a professional document with:
/// - Header with title and generation date
/// - Summary with selected applications metadata
/// - Individual application content
/// - Footer with application info
pub async fn generate_handläggare_report(
    db: &Db,
    selected_draft_ids: &[String],
    output_path: &std::path::Path,
) -> Result<()> {
    let mut doc = Docx::new();

    // Document header
    let report_date = Utc::now().format("%Y-%m-%d %H:%M").to_string();

    doc = doc.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text("Handläggarrapport"))
            .size(28)
            .bold(),
    );

    doc = doc.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text(&format!("Genererad: {}", report_date)))
            .size(12),
    );

    doc = doc.add_paragraph(Paragraph::new()); // Empty line

    // Process each selected draft
    for (index, draft_id) in selected_draft_ids.iter().enumerate() {
        // Get job ad metadata
        if let Some(ad) = db.get_job_by_id(draft_id).await? {
            // Section header
            doc = doc.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&format!("Ansökan {}", index + 1)))
                    .size(18)
                    .bold(),
            );

            // Metadata row
            let employer = ad
                .employer
                .as_ref()
                .and_then(|e| e.name.as_ref())
                .unwrap_or("Okänd arbetsgivare");

            doc = doc.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(&format!("Företag: {}", employer))),
            );

            let publication = ad
                .publication_date
                .split('T')
                .next()
                .unwrap_or(&ad.publication_date);

            doc = doc.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(&format!("Publicerad: {}", publication))),
            );

            // Get application content
            if let Some(Some(content)) = db.get_application_draft(draft_id).await.ok() {
                doc = doc.add_paragraph(Paragraph::new()); // Empty line
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Ansökningstext:"))
                        .bold(),
                );

                // Split content into paragraphs for readability
                for line in content.lines() {
                    if !line.trim().is_empty() {
                        doc =
                            doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line)));
                    } else {
                        doc = doc.add_paragraph(Paragraph::new()); // Preserve empty lines
                    }
                }
            }

            // Separator between applications (except last)
            if index < selected_draft_ids.len() - 1 {
                doc = doc.add_paragraph(Paragraph::new()); // Empty line
                doc = doc
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text("─".repeat(60))));
                doc = doc.add_paragraph(Paragraph::new()); // Empty line
            }
        }
    }

    // Footer
    doc = doc.add_paragraph(Paragraph::new()); // Empty line
    doc = doc.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text("─".repeat(60))),
    );

    doc = doc.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text(
                "Detta dokument är en sammanställning av ansökningar till Arbetsförmedlingen.",
            ))
            .size(10),
    );

    // Save to file
    let file = std::fs::File::create(output_path)?;
    doc.build().pack(file)?;

    Ok(())
}
