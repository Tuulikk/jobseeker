use crate::models::JobAd;
use async_openai::{
    types::chat::{ 
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use anyhow::Result;

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