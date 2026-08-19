use crate::models::JobAd;
use anyhow::Result;
use async_openai::{
    types::chat::{ 
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};

pub struct AiRanker {
    client: Client<async_openai::config::OpenAIConfig>,
}

impl AiRanker {
    pub fn new(base_url: &str, api_key: &str) -> Result<Self> {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);
        
        Ok(Self {
            client: Client::with_config(config),
        })
    }

    pub async fn rate_job(&self, ad: &JobAd, my_profile: &str) -> Result<u8> {
        let description = ad.description.as_ref().and_then(|d| d.text.as_ref()).map(|s| s.as_str()).unwrap_or("");
        let prompt = format!(
            "Rate how well this job matches my profile. Output ONLY a single number from 1 to 10.\n\nMy Profile:\n{}\n\nJob Headline: {}\nJob Description: {}",
            my_profile, ad.headline, description
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model("llama3")
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a career advisor assistant. You rate job matches from 1 to 10. Output only the digit.")
                    .build()? 
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()? 
                    .into(),
            ])
            .max_tokens(10u32)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let content = response.choices[0].message.content.clone().unwrap_or_default();
        let rating = content.trim().chars()
            .rfind(|c| c.is_ascii_digit())
            .and_then(|c| c.to_digit(10))
            .unwrap_or(0) as u8;

        Ok(rating)
    }
}

pub struct LocalAi;

impl LocalAi {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn extractive_summarize(&self, text: &str) -> Result<String> {
        let lines: Vec<&str> = text.split('\n').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let mut sentences = Vec::new();
        for line in lines {
            if (line.len() < 50 && line.ends_with(':')) || (line.len() < 35 && line.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c.is_ascii_punctuation())) {
                sentences.push(line);
            } else {
                for s in line.split_inclusive(|c| c == '.' || c == '!' || c == '?') {
                    let trimmed = s.trim();
                    if trimmed.len() > 4 { sentences.push(trimmed); }
                }
            }
        }

        if sentences.is_empty() { return Ok("Kunde inte analysera innehållet.".into()); }

        // Definiera ordning och signaler
        let clusters = vec![
            ("ROLLEN", vec!["ansvar", "arbetsuppgift", "roll", "utför", "bidra", "projekt", "fokus", "innebär", "leda", "driva", "task"]),
            ("KRAV", vec!["krav", "erfaren", "utbild", "kompeten", "merit", "behärsk", "kvalif", "bakgrund", "kunskap", "du har", "förmåga", "skall"]),
            ("ARBETSPLATSEN", vec!["kult", "team", "kolleg", "värder", "visio", "gemenskap", "miljö", "om oss", "vi är", "erbjuder"])
        ];

        let mut results_per_cat = std::collections::HashMap::new();
        let mut used_indices = std::collections::HashSet::new();
        let mut current_header_cat: Option<&str> = None;

        for (cat_name, stems) in clusters.clone() {
            let mut scores: Vec<(usize, f32)> = Vec::new();

            for (idx, sent) in sentences.iter().enumerate() {
                if used_indices.contains(&idx) { continue; }

                let low_sent = sent.to_lowercase();
                
                // Rubrik-detektering för kontext
                if sent.ends_with(':') || (sent.len() < 35 && sent.chars().all(|c| c.is_uppercase() || c.is_whitespace())) {
                    if low_sent.contains("gör") || low_sent.contains("roll") || low_sent.contains("uppgift") { current_header_cat = Some("ROLLEN"); }
                    else if low_sent.contains("vem") || low_sent.contains("krav") || low_sent.contains("merit") || low_sent.contains("profil") { current_header_cat = Some("KRAV"); }
                    else if low_sent.contains("vi ") || low_sent.contains("oss") || low_sent.contains("erbjuder") { current_header_cat = Some("ARBETSPLATSEN"); }
                    continue;
                }

                let mut score = 0.0;

                // Brus-filter
                if low_sent.contains("http") || low_sent.contains("ansök") || low_sent.contains("läs mer") { continue; }
                let first_char = sent.chars().next().unwrap_or(' ');
                if !first_char.is_uppercase() && !sent.starts_with('•') && !sent.starts_with('-') { continue; }

                // Exklusivitet & Special-triggers
                if cat_name == "KRAV" {
                    if low_sent.contains("erfarenhet") || low_sent.contains("krav") || low_sent.contains("förmåga") || low_sent.contains("kunskap") { score += 12.0; }
                    if low_sent.contains("vi ") || low_sent.contains("erbjuder") { score -= 20.0; }
                }
                
                if cat_name == "ARBETSPLATSEN" {
                    if low_sent.contains("meriterande") || low_sent.contains("erfarenhet") || low_sent.contains("skall") { score -= 20.0; }
                    if low_sent.contains("vi ") || low_sent.contains("team") || low_sent.contains("miljö") { score += 5.0; }
                }

                // Stam-matchning
                for stem in &stems {
                    if low_sent.contains(stem) { score += 3.0; }
                }

                if current_header_cat == Some(cat_name) { score += 6.0; }
                
                if score <= 2.0 { continue; }

                let len = sent.len();
                if len > 40 && len < 180 { score += 2.0; }
                
                scores.push((idx, score));
            }

            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let mut cat_sentences = Vec::new();
            let mut count = 0;
            for (idx, _) in scores {
                if count >= 2 { break; }
                cat_sentences.push((idx, sentences[idx]));
                used_indices.insert(idx);
                count += 1;
            }
            // Sortera internt efter originalordning
            cat_sentences.sort_by_key(|&(idx, _)| idx);
            results_per_cat.insert(cat_name, cat_sentences);
        }

        let mut final_result = String::new();
        // Skriv ut i fix ordning: ROLLEN -> KRAV -> ARBETSPLATSEN
        for (cat_name, _) in clusters {
            if let Some(sents) = results_per_cat.get(cat_name) {
                if !sents.is_empty() {
                    if !final_result.is_empty() { final_result.push_str("\n"); }
                    final_result.push_str(&format!("{}:", cat_name));
                    for (_, text) in sents {
                        let clean_text = text.trim_start_matches(|c| c == '-' || c == '*' || c == '•' || c == ' ').trim();
                        final_result.push_str(&format!("\n • {}", clean_text));
                    }
                }
            }
        }

        if final_result.is_empty() { return Ok("Ingen kärna identifierad.".into()); }
        Ok(final_result)
    }
}