use iced::widget::{button, container, row, text, text_editor};
use iced::{Alignment, Color, Element, Length, Padding, Theme};
use pulldown_cmark::{Options, Parser, html};

#[derive(Debug, Clone)]
pub enum RichEditorMessage {
    ActionPerformed(text_editor::Action),
    Bold,
    Italic,
    Heading1,
    Heading2,
    Heading3,
    BulletList,
    NumberedList,
    Link,
    InsertText(String),
}

#[derive(Clone, Debug)]
pub struct RichEditor {
    content: text_editor::Content,
}

impl RichEditor {
    pub fn new() -> Self {
        Self {
            content: text_editor::Content::new(),
        }
    }

    pub fn with_text(text: &str) -> Self {
        Self {
            content: text_editor::Content::with_text(text),
        }
    }

    #[allow(dead_code)]
    pub fn content(&self) -> &text_editor::Content {
        &self.content
    }

    pub fn text(&self) -> String {
        self.content.text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.content = text_editor::Content::with_text(text);
    }

    pub fn update(&mut self, message: RichEditorMessage) {
        match message {
            RichEditorMessage::ActionPerformed(action) => {
                self.content.perform(action);
            }
            RichEditorMessage::Bold => {
                self.wrap_selection("**", "**");
            }
            RichEditorMessage::Italic => {
                self.wrap_selection("*", "*");
            }
            RichEditorMessage::Heading1 => {
                self.insert_at_line_start("# ");
            }
            RichEditorMessage::Heading2 => {
                self.insert_at_line_start("## ");
            }
            RichEditorMessage::Heading3 => {
                self.insert_at_line_start("### ");
            }
            RichEditorMessage::BulletList => {
                self.insert_at_line_start("- ");
            }
            RichEditorMessage::NumberedList => {
                self.insert_at_line_start("1. ");
            }
            RichEditorMessage::Link => {
                self.wrap_selection("[", "](url)");
            }
            RichEditorMessage::InsertText(text) => {
                let current = self.content.text();
                let new_text = if current.is_empty() {
                    text
                } else {
                    format!("{}\n\n{}", current, text)
                };
                self.content = text_editor::Content::with_text(&new_text);
            }
        }
    }

    fn wrap_selection(&mut self, prefix: &str, suffix: &str) {
        let text = self.content.text();
        let new_text = format!("{}{}{}", prefix, text, suffix);
        self.content = text_editor::Content::with_text(&new_text);
        // Note: Proper selection wrapping would require cursor position tracking
        // which text_editor doesn't expose yet. This is a simplified version.
    }

    fn insert_at_line_start(&mut self, prefix: &str) {
        let text = self.content.text();
        let new_text = format!("{}{}", prefix, text);
        self.content = text_editor::Content::with_text(&new_text);
    }

    pub fn view<'a>(&'a self, show_toolbar: bool) -> Element<'a, RichEditorMessage> {
        let editor = container(
            text_editor(&self.content)
                .placeholder("Skriv ditt personliga brev här...\n\nTips: Använd Markdown-formatering:\n**fetstil**, *kursiv*, # Rubrik")
                .on_action(RichEditorMessage::ActionPerformed)
        )
        .padding(Padding { top: 40.0, right: 60.0, bottom: 40.0, left: 60.0 })
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(Color::WHITE.into()),
            border: iced::Border {
                color: Color::from_rgb(0.7, 0.7, 0.7),
                width: 1.0,
                radius: 2.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(2.0, 2.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        });

        if show_toolbar {
            let toolbar = container(
                row![
                    button(text("B").size(14).color(Color::WHITE))
                        .on_press(RichEditorMessage::Bold)
                        .padding(8)
                        .style(|_theme: &Theme, status| {
                            button::Style {
                                background: Some(if status == button::Status::Hovered {
                                    Color::from_rgb(0.3, 0.3, 0.4).into()
                                } else {
                                    Color::from_rgb(0.2, 0.2, 0.3).into()
                                }),
                                text_color: Color::WHITE,
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        }),
                    button(text("I").size(14).color(Color::WHITE))
                        .on_press(RichEditorMessage::Italic)
                        .padding(8)
                        .style(|_theme: &Theme, status| {
                            button::Style {
                                background: Some(if status == button::Status::Hovered {
                                    Color::from_rgb(0.3, 0.3, 0.4).into()
                                } else {
                                    Color::from_rgb(0.2, 0.2, 0.3).into()
                                }),
                                text_color: Color::WHITE,
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        }),
                    button(text("H1").size(12))
                        .on_press(RichEditorMessage::Heading1)
                        .padding(8),
                    button(text("H2").size(12))
                        .on_press(RichEditorMessage::Heading2)
                        .padding(8),
                    button(text("H3").size(12))
                        .on_press(RichEditorMessage::Heading3)
                        .padding(8),
                    button(text("• Lista").size(12))
                        .on_press(RichEditorMessage::BulletList)
                        .padding(8),
                    button(text("1. Lista").size(12))
                        .on_press(RichEditorMessage::NumberedList)
                        .padding(8),
                    button(text("🔗 Länk").size(12))
                        .on_press(RichEditorMessage::Link)
                        .padding(8),
                ]
                .spacing(8)
                .padding(10)
                .align_y(Alignment::Center),
            )
            .style(|_theme: &Theme| container::Style {
                background: Some(Color::from_rgba(0.1, 0.1, 0.15, 0.95).into()),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.6, 0.8),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 15.0,
                },
                ..Default::default()
            });

            iced::widget::stack![
                editor,
                container(toolbar)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(30)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::End)
            ]
            .into()
        } else {
            editor.into()
        }
    }
}

impl Default for RichEditor {
    fn default() -> Self {
        Self::new()
    }
}

// Markdown utilities
pub mod markdown {
    use super::*;
    use pulldown_cmark::{Event, TagEnd};

    /// Convert Markdown to HTML with proper styling
    pub fn to_html(markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Wrap in HTML template with styling
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            font-family: 'Segoe UI', 'Arial', sans-serif;
            font-size: 12pt;
            line-height: 1.6;
            max-width: 800px;
            margin: 40px auto;
            padding: 40px 60px;
            color: #333;
        }}
        h1 {{
            font-size: 24pt;
            font-weight: bold;
            margin: 24px 0 12px;
            color: #1a1a1a;
        }}
        h2 {{
            font-size: 18pt;
            font-weight: bold;
            margin: 20px 0 10px;
            color: #1a1a1a;
        }}
        h3 {{
            font-size: 14pt;
            font-weight: bold;
            margin: 16px 0 8px;
            color: #1a1a1a;
        }}
        p {{
            margin: 10px 0;
            text-align: justify;
        }}
        strong {{
            font-weight: bold;
        }}
        em {{
            font-style: italic;
        }}
        ul, ol {{
            margin: 10px 0;
            padding-left: 30px;
        }}
        li {{
            margin: 5px 0;
        }}
        blockquote {{
            border-left: 4px solid #ccc;
            padding-left: 16px;
            margin: 16px 0;
            color: #666;
            font-style: italic;
        }}
        a {{
            color: #0066cc;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        code {{
            background: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
        }}
        pre {{
            background: #f4f4f4;
            padding: 12px;
            border-radius: 4px;
            overflow-x: auto;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 16px 0;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }}
        th {{
            background-color: #f4f4f4;
            font-weight: bold;
        }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            html_output
        )
    }

    /// Extract plain text from Markdown (removing all formatting)
    #[allow(dead_code)]
    pub fn to_plain_text(markdown: &str) -> String {
        let parser = Parser::new(markdown);
        let mut plain_text = String::new();

        for event in parser {
            match event {
                Event::Text(text) | Event::Code(text) => {
                    plain_text.push_str(&text);
                }
                Event::SoftBreak | Event::HardBreak => {
                    plain_text.push('\n');
                }
                Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Heading(_)) => {
                    plain_text.push_str("\n\n");
                }
                Event::End(TagEnd::Item) => {
                    plain_text.push('\n');
                }
                _ => {}
            }
        }

        plain_text.trim().to_string()
    }

    /// Render Markdown into an Iced Element for a richer, styled preview.
    ///
    /// This is a lightweight renderer intended for a live preview inside the app:
    /// - supports headings (#/##/###), paragraphs and simple unordered lists (- /*)
    /// - preserves line breaks and basic structure
    pub fn to_iced<'a, M: 'static>(markdown: &str) -> iced::Element<'a, M> {
        use iced::Alignment;
        use iced::Length;
        use iced::widget::Column;
        use iced::widget::text;

        let mut col = Column::new().spacing(8).padding(8).width(Length::Fill);

        let mut paragraph_buf = String::new();

        for line in markdown.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !paragraph_buf.trim().is_empty() {
                    col = col.push(text(paragraph_buf.trim().to_string()).size(14));
                    paragraph_buf.clear();
                }
                continue;
            }

            if trimmed.starts_with("# ") {
                if !paragraph_buf.trim().is_empty() {
                    col = col.push(text(paragraph_buf.trim().to_string()).size(14));
                    paragraph_buf.clear();
                }
                col = col.push(text(trimmed.trim_start_matches("# ").trim().to_string()).size(22));
            } else if trimmed.starts_with("## ") {
                if !paragraph_buf.trim().is_empty() {
                    col = col.push(text(paragraph_buf.trim().to_string()).size(14));
                    paragraph_buf.clear();
                }
                col = col.push(text(trimmed.trim_start_matches("## ").trim().to_string()).size(18));
            } else if trimmed.starts_with("### ") {
                if !paragraph_buf.trim().is_empty() {
                    col = col.push(text(paragraph_buf.trim().to_string()).size(14));
                    paragraph_buf.clear();
                }
                col =
                    col.push(text(trimmed.trim_start_matches("### ").trim().to_string()).size(16));
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                if !paragraph_buf.trim().is_empty() {
                    col = col.push(text(paragraph_buf.trim().to_string()).size(14));
                    paragraph_buf.clear();
                }
                let bullet = trimmed
                    .trim_start_matches("- ")
                    .trim_start_matches("* ")
                    .trim();
                col = col.push(text(format!("• {}", bullet)).size(14));
            } else {
                if !paragraph_buf.is_empty() {
                    paragraph_buf.push(' ');
                }
                paragraph_buf.push_str(trimmed);
            }
        }

        if !paragraph_buf.trim().is_empty() {
            col = col.push(text(paragraph_buf.trim().to_string()).size(14));
        }

        col.align_x(Alignment::Start).into()
    }

    /// Create a professional application letter template
    pub fn create_template(company: &str, position: &str, profile: &str) -> String {
        format!(
            r#"# Ansökan - {}

**Till:** {}

Hej,

Jag skriver för att uttrycka mitt intresse för tjänsten som {} hos {}.

## Om mig

{}

## Varför just denna tjänst?

[Beskriv varför du är intresserad av just denna tjänst och detta företag]

## Vad jag kan bidra med

[Förklara hur din kompetens och erfarenhet matchar tjänstens krav]

Jag ser fram emot att höra från er och hoppas på möjligheten att diskutera hur jag kan bidra till ert team.

Med vänliga hälsningar,
[Ditt namn]
[Din kontaktinformation]"#,
            position, company, position, company, profile
        )
    }
}

// Export utilities
pub mod export {
    use super::markdown;
    use anyhow::Result;
    use docx_rs::*;
    use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
    use std::path::Path;

    /// Export Markdown to PDF via HTML
    #[allow(dead_code)]
    pub async fn markdown_to_pdf(markdown: &str, output_path: &Path) -> Result<()> {
        let html = markdown::to_html(markdown);

        // Write HTML for debugging/preview
        let html_path = output_path.with_extension("html");
        tokio::fs::write(&html_path, &html).await?;

        // For proper PDF conversion, we would need:
        // 1. headless_chrome to render HTML to PDF
        // 2. wkhtmltopdf binary
        // 3. or printpdf with custom HTML parsing

        // For now, inform user to use the HTML file for PDF conversion
        println!("HTML exported to: {:?}", html_path);
        println!("Use your browser to print this HTML file as PDF");

        Ok(())
    }

    /// Export Markdown to DOCX with formatting
    pub async fn markdown_to_docx(markdown: &str, output_path: &Path) -> Result<()> {
        let mut doc = Docx::new();

        // Configure document with professional styling
        doc = doc.add_paragraph(Paragraph::new().style("Normal").align(AlignmentType::Left));

        let parser = Parser::new(markdown);
        let mut current_paragraph = Paragraph::new();
        let _in_list = false;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let style = match level {
                        HeadingLevel::H1 => "Heading1",
                        HeadingLevel::H2 => "Heading2",
                        HeadingLevel::H3 => "Heading3",
                        _ => "Heading4",
                    };
                    current_paragraph = Paragraph::new().style(style);
                }
                Event::Start(Tag::Paragraph) => {
                    current_paragraph = Paragraph::new().align(AlignmentType::Both);
                }
                Event::Start(Tag::Emphasis) => {
                    // Mark for italic
                }
                Event::Start(Tag::Strong) => {
                    // Mark for bold
                }
                Event::Text(text) => {
                    let run = Run::new().add_text(text.to_string());
                    current_paragraph = current_paragraph.add_run(run);
                }
                Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Heading(_)) => {
                    doc = doc.add_paragraph(current_paragraph.clone());
                    current_paragraph = Paragraph::new();
                }
                Event::SoftBreak => {
                    current_paragraph = current_paragraph.add_run(Run::new().add_text(" "));
                }
                Event::HardBreak => {
                    current_paragraph =
                        current_paragraph.add_run(Run::new().add_break(BreakType::TextWrapping));
                }
                _ => {}
            }
        }

        let file = std::fs::File::create(output_path)?;
        doc.build().pack(file)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RichEditor;
    use super::RichEditorMessage;
    use super::markdown;

    #[test]
    fn test_plain_text_basic() {
        let md = "# Hello\n\nThis is a *test*.\n\n- One\n- Two";
        let plain = markdown::to_plain_text(md);
        assert!(plain.contains("Hello"));
        assert!(plain.contains("test"));
        assert!(plain.contains("One"));
    }

    #[test]
    fn test_to_html_contains_heading_and_paragraph() {
        let md = "# Title\n\nParagraph";
        let html = markdown::to_html(md);
        assert!(html.contains("<h1"));
        assert!(html.contains("Paragraph"));
    }

    #[test]
    fn test_to_iced_no_panic() {
        let md = "# Hello\n\n- Item1\n- Item2\n\nParagraph";
        // Smoke test: ensure the renderer doesn't panic and returns an Element
        let _element: iced::Element<'_, RichEditorMessage> =
            markdown::to_iced::<RichEditorMessage>(md);
    }

    #[test]
    fn test_create_template_contains_position_and_profile() {
        let tpl = markdown::create_template("Acme AB", "Utvecklare", "Jag är erfaren");
        assert!(tpl.contains("Acme AB"));
        assert!(tpl.contains("Utvecklare"));
        assert!(tpl.contains("Jag är erfaren"));
        assert!(tpl.contains("# Ansökan"));
    }

    #[test]
    fn test_rich_editor_bold_wraps_selection() {
        let mut editor = RichEditor::with_text("Ord");
        editor.update(RichEditorMessage::Bold);
        assert_eq!(editor.text(), "**Ord**");
    }
}
