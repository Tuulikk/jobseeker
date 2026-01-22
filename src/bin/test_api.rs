use reqwest::Client;
use serde_json::Value;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = "https://jobsearch.api.jobtechdev.se";

    // Test P1: locations_p1 = "1283, 1277, 1260, 1292, 1284, 1276, 1231, 1282, 1261"
    let locations_p1 = vec![
        "1283".to_string(),
        "1277".to_string(),
        "1260".to_string(),
        "1292".to_string(),
        "1284".to_string(),
        "1276".to_string(),
        "1231".to_string(),
        "1282".to_string(),
        "1261".to_string(),
    ];

    // Test P2: locations_p2 = "1280, 1281"
    let locations_p2 = vec![
        "1280".to_string(),
        "1281".to_string(),
    ];

    println!("=== Test P1 (9 kommuner) ===");
    let ads_p1 = search_multi(&client, base_url, &locations_p1, "it", 5).await?;
    println!("Hittade {} annonser för P1", ads_p1.len());

    println!("\n=== Sample from P1 ===");
    for ad in ads_p1.iter().take(10) {
        if let Some(mun) = ad.get("workplace_address").and_then(|v| v.get("municipality")) {
            println!("  {}: {}", mun.as_str().unwrap_or("?"), ad.get("headline").and_then(|v| v.as_str()).unwrap_or("N/A"));
        }
    }

    println!("\n=== Test P2 (2 kommuner) ===");
    let ads_p2 = search_multi(&client, base_url, &locations_p2, "it", 5).await?;
    println!("Hittade {} annonser för P2", ads_p2.len());

    println!("\n=== Sample from P2 ===");
    for ad in ads_p2.iter().take(10) {
        if let Some(mun) = ad.get("workplace_address").and_then(|v| v.get("municipality")) {
            println!("  {}: {}", mun.as_str().unwrap_or("?"), ad.get("headline").and_then(|v| v.as_str()).unwrap_or("N/A"));
        }
    }

    Ok(())
}

async fn search_multi(client: &Client, base_url: &str, municipalities: &[String], query: &str, limit_per_municipality: u32) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut all_ads = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for m in municipalities {
        if m.is_empty() { continue; }

        let params = vec![
            ("q", query.to_string()),
            ("limit", limit_per_municipality.to_string()),
            ("municipality", m.to_string()),
        ];

        let url = format!("{}/search", base_url);
        println!("Fetching for municipality {}: {}", m, url);

        let response = client.get(&url)
            .header("accept", "application/json")
            .query(&params)
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
                            all_ads.push(hit.clone());
                        }
                    }
                }
            }
        }
    }

    println!("Total unique ads after merging {} municipalities: {}", municipalities.len(), all_ads.len());
    Ok(all_ads)
}
