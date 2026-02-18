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
            .find(|c| c.is_ascii_digit())
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
        // 1. Förbered texten - hantera punkter, utropstecken, frågetecken, nya rader och bullets
        let sentences: Vec<&str> = text.split_inclusive(|c| c == '.' || c == '!' || c == '?' || c == '\n' || c == '•')
            .map(|s| s.trim())
            .filter(|s| s.len() > 10)
            .collect();

        if sentences.is_empty() { return Ok("Kunde inte extrahera tillräckligt med text.".into()); }

        // 2. Definiera kluster av ord-stammar (Fuzzy-ish matching)
        let clusters = vec![
            ("ARBETSPLATSEN", vec!["kult", "miljö", "team", "kolleg", "värder", "visio", "fika", "trygg", "erbjud", "förmån", "kontor", "gemenskap"]),
            ("ROLLEN", vec!["ansvar", "arbetsuppgift", "roll", "varda", "utför", "daglig", "bidra", "projekt", "fokus", "innebär", "utveckl", "utman"]),
            ("KRAV", vec!["krav", "erfaren", "utbild", "kompeten", "merit", "sök", "behärsk", "kvalif", "bakgrund", "kunskap", "skall", "bör"])
        ];

        let mut extracted = Vec::new();
        let mut used_indices = std::collections::HashSet::new();

        for (cat_name, stems) in clusters {
            let mut scores: Vec<(usize, f32)> = Vec::new();

            for (idx, sent) in sentences.iter().enumerate() {
                if used_indices.contains(&idx) { continue; }

                let low_sent = sent.to_lowercase();
                let mut score = 0.0;

                // A. Fuzzy-ish Stam-matchning
                for stem in &stems {
                    if low_sent.contains(stem) { 
                        score += 1.5; 
                        if low_sent.starts_with(stem) { score += 0.5; }
                    }
                }

                if score == 0.0 { continue; }

                // B. Struktur-analys
                if sent.starts_with('-') || sent.starts_with('*') || sent.starts_with('•') {
                    score += 2.0;
                }

                // C. Salience (Längd-viktning)
                let len = sent.len();
                if len > 60 && len < 160 { score += 1.2; }
                else if len > 220 { score -= 0.8; }

                // D. Positions-magi
                let pos_ratio = idx as f32 / sentences.len() as f32;
                if cat_name == "ARBETSPLATSEN" { score *= 1.0 + (1.0 - pos_ratio); }
                if cat_name == "KRAV" { score *= 1.0 + pos_ratio; }

                // E. Signal-fraser
                if cat_name == "KRAV" && (low_sent.contains("vi söker dig") || low_sent.contains("du har")) { score += 3.5; }
                if cat_name == "ROLLEN" && (low_sent.contains("dina arbetsuppgifter") || low_sent.contains("du kommer att")) { score += 3.5; }
                if cat_name == "ARBETSPLATSEN" && low_sent.contains("vi erbjuder") { score += 3.5; }

                scores.push((idx, score));
            }

            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            if let Some(&(idx, _)) = scores.get(0) {
                extracted.push((idx, cat_name, sentences[idx]));
                used_indices.insert(idx);
            }
        }

        extracted.sort_by_key(|&(idx, _, _)| idx);

        let result = extracted.into_iter()
            .map(|(_, cat, text)| {
                let clean_text = text.trim_start_matches(|c| c == '-' || c == '*' || c == '•' || c == ' ').trim();
                format!("[{}] {}", cat, clean_text)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(result)
    }
}
