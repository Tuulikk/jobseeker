use crate::models::JobAd;
use serde_json::Value;
use anyhow::{Result, Context};
use log::{info, error};
use reqwest::Client;

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
    ("bromölla", "1271"), ("stockholm", "0180"), ("huddinge", "0126"), ("nacka", "0182"),
    ("göteborg", "1480"), ("mölndal", "1481"), ("uppsala", "0380"), ("västerås", "1980"),
];

impl JobSearchClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: "https://jobsearch.api.jobtechdev.se".to_string(),
        }
    }

    pub fn get_municipality_code(name: &str) -> Option<&'static str> {
        let name_lower = name.to_lowercase();
        MUNICIPALITIES.iter().find(|(n, _)| *n == name_lower.as_str()).map(|(_, c)| *c)
    }

    pub fn get_municipality_name(code: &str) -> Option<String> {
        MUNICIPALITIES.iter().find(|(_, c)| *c == code).map(|(n, _)| {
            let mut chars = n.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
    }

    pub fn parse_locations(input: &str) -> Vec<String> {
        input.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| {
            if s.chars().all(char::is_numeric) { s.to_string() }
            else { Self::get_municipality_code(s).map(|c| c.to_string()).unwrap_or_default() }
        }).filter(|s| !s.is_empty()).collect()
    }

    pub async fn search(&self, query: &str, municipalities: &[String], limit: u32) -> Result<Vec<JobAd>> {
        if municipalities.len() > 1 {
            return self.search_multi_municipalities(query, municipalities, limit).await;
        }

        let mut params = vec![("q", query.to_string()), ("limit", limit.to_string())];
        for m in municipalities { if !m.is_empty() { params.push(("municipality", m.to_string())); } }

        let url = format!("{}/search", self.base_url);
        info!("Fetching: {}", url);

        let response = self.client.get(&url)
            .header("accept", "application/json")
            .query(&params)
            .send()
            .await
            .context("Failed to send request to JobSearch API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("API Error: {}", body);
            return Err(anyhow::anyhow!("API returned HTTP {}", status));
        }

        let json: Value = response.json().await?;
        let hits = json["hits"].as_array().ok_or_else(|| anyhow::anyhow!("Missing hits"))?;

        let mut results = Vec::new();
        for hit in hits {
            if let Ok(mut ad) = serde_json::from_value::<JobAd>(hit.clone()) {
                ad.webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());
                results.push(ad);
            }
        }
        Ok(results)
    }

    async fn search_multi_municipalities(&self, query: &str, municipalities: &[String], limit: u32) -> Result<Vec<JobAd>> {
        use std::collections::HashSet;
        let mut all_ads = Vec::new();
        let mut seen_ids = HashSet::new();

        for m in municipalities {
            if m.is_empty() { continue; }
            let params = vec![("q", query.to_string()), ("limit", limit.to_string()), ("municipality", m.to_string())];
            let url = format!("{}/search", self.base_url);
            
            let resp_res = self.client.get(&url)
                .header("accept", "application/json")
                .query(&params)
                .send()
                .await;

            if let Ok(resp) = resp_res {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(hits) = json["hits"].as_array() {
                        for hit in hits {
                            if let Ok(mut ad) = serde_json::from_value::<JobAd>(hit.clone()) {
                                ad.webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());
                                if seen_ids.insert(ad.id.clone()) { all_ads.push(ad); }
                            }
                        }
                    }
                }
            }
        }
        Ok(all_ads)
    }
}
