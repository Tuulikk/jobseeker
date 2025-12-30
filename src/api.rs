use reqwest::Client;
use crate::models::JobAd;
use serde_json::Value;
use anyhow::{Result, Context};

pub struct JobSearchClient {
    client: Client,
    base_url: String,
}

impl JobSearchClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://jobsearch.api.jobtechdev.se".to_string(),
        }
    }

    pub async fn search(&self, query: &str, municipalities: &[String], limit: u32) -> Result<Vec<JobAd>> {
        let mut params = vec![
            ("q", query.to_string()),
            ("limit", limit.to_string()),
        ];
        
        for m in municipalities {
            if !m.is_empty() {
                params.push(("municipality", m.to_string()));
            }
        }

        let url = format!("{}/search", self.base_url);
        
        let response = self.client.get(&url)
            .header("accept", "application/json")
            .query(&params)
            .send()
            .await
            .context("Failed to send request to JobSearch API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API Error: {} - {}", status, body));
        }

        let json: Value = response.json().await.context("Failed to parse JSON response")?;
        
        let hits = json["hits"].as_array()
            .context("No 'hits' array found in response")?;

        let mut ads = Vec::new();
        for hit in hits {
            let ad_val = hit.clone();
            
            // Extract webpage_url from root if not present in nested structs
            let webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());
            
            match serde_json::from_value::<JobAd>(ad_val.clone()) {
                Ok(mut ad) => {
                    ad.webpage_url = webpage_url;
                    ads.push(ad);
                },
                Err(e) => {
                    eprintln!("Error parsing job ad: {}. Value: {:?}", e, hit);
                }
            }
        }

        Ok(ads)
    }
}

impl Default for JobSearchClient {
    fn default() -> Self {
        Self::new()
    }
}