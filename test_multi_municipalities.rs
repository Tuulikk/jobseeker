use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test: search_multi_municipalities (som i src/api.rs)");
    println!("============================================");
    println!();

    let client = Client::new();
    let municipalities = vec!["1283".to_string(), "1277".to_string(), "1260".to_string()];
    let query = "it";
    let limit_per_municipality = 10;

    let mut all_ads = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    println!("Searching across {} municipalities (separate API calls)", municipalities.len());
    println!();

    for m in &municipalities {
        if m.is_empty() { continue; }

        let url = format!("https://jobsearch.api.jobtechdev.se/search?q={}&municipality={}&limit={}",
                     query, m, limit_per_municipality);
        println!("Fetching for municipality {}: {}", m, url);

        let response = client.get(&url)
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            println!("  ERROR: HTTP {}", response.status());
            continue;
        }

        if let Ok(json) = response.json::<Value>().await {
            if let Some(hits) = json["hits"].as_array() {
                println!("  Municipality {}: {} hits", m, hits.len());

                for hit in hits {
                    if let Some(id) = hit.get("id").and_then(|v| v.as_str()) {
                        if seen_ids.insert(id.to_string()) {
                            if let Some(headline) = hit.get("headline").and_then(|v| v.as_str()) {
                                if let Some(mun) = hit.get("workplace_address")
                                    .and_then(|a| a.get("municipality"))
                                    .and_then(|m| m.as_str()) {
                                    println!("    - {} (kommun: {})", headline, mun);
                                }
                            }
                            all_ads.push(hit.clone());
                        }
                    }
                }
            }
        }
    }

    println!();
    println!("Total unique ads after merging {} municipalities: {}", municipalities.len(), all_ads.len());

    Ok(())
}
