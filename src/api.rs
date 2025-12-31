use reqwest::Client;
use crate::models::JobAd;
use serde_json::Value;
use anyhow::{Result, Context};

pub struct JobSearchClient {
    client: Client,
    base_url: String,
}

const MUNICIPALITIES: &[(&str, &str)] = &[
    ("helsingborg", "1283"), ("ängelholm", "1292"), ("höganäs", "1284"), ("bjuv", "1260"),
    ("klippan", "1276"), ("åstorp", "1277"), ("örkelljunga", "1257"), ("båstad", "1278"),
    ("perstorp", "1275"), ("landskrona", "1282"), ("svalöv", "1214"), ("burlöv", "1231"),
    ("kävlinge", "1261"), ("malmö", "1280"), ("lund", "1281"), ("eslöv", "1285"),
    ("vellinge", "1233"), ("trelleborg", "1287"), ("ystad", "1286"), ("kristianstad", "1290"),
    ("hässleholm", "1293"), ("lomma", "1262"), ("staffanstorp", "1230"), ("svedala", "1263"),
    ("skurup", "1264"), ("sjöbo", "1265"), ("höör", "1267"), ("hörby", "1266"),
    ("tomelilla", "1270"), ("simrishamn", "1291"), ("osby", "1272"), ("östra göinge", "1273"),
    ("bromölla", "1271"), ("stockholm", "0180"), ("göteborg", "1480"), ("uppsala", "0380"),
    ("västerås", "1980"), ("örebro", "1880"), ("linköping", "0580"), ("norrköping", "0581"),
    ("jönköping", "0680"), ("umeå", "2480"),
];

impl JobSearchClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://jobsearch.api.jobtechdev.se".to_string(),
        }
    }

    pub fn get_municipality_code(name: &str) -> Option<&'static str> {
        let name_lower = name.to_lowercase();
        MUNICIPALITIES.iter()
            .find(|(n, _)| *n == name_lower.as_str())
            .map(|(_, c)| *c)
    }

    pub fn get_municipality_name(code: &str) -> Option<String> {
        MUNICIPALITIES.iter()
            .find(|(_, c)| *c == code)
            .map(|(n, _)| {
                let mut chars = n.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
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