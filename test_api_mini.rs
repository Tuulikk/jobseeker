use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Jobseeker API Test App");
    println!("======================");
    println!();

    let client = Client::new();
    let base_url = "https://jobsearch.api.jobtechdev.se/search";

    // Test 1: En kommun, ett sökord
    println!("Test 1: En kommun (1283), sökord 'it'");
    let url1 = format!("{}?q=it&municipality=1283&limit=10", base_url);
    test_request(&client, &url1).await?;

    // Test 2: Två kommuner, ett sökord
    println!("\nTest 2: Två kommuner (1283,1277), sökord 'it'");
    let url2 = format!("{}?q=it&municipality=1283&municipality=1277&limit=10", base_url);
    test_request(&client, &url2).await?;

    // Test 3: Kommaseparerad kommunlista (om API stöder det)
    println!("\nTest 3: Kommaseparerad (1283,1277), sökord 'it'");
    let url3 = format!("{}?q=it&municipality=1283,1277&limit=10", base_url);
    test_request(&client, &url3).await?;

    // Test 4: Två sökord, en kommun
    println!("\nTest 4: En kommun (1283), två sökord 'it support'");
    let url4 = format!("{}?q=it%20support&municipality=1283&limit=10", base_url);
    test_request(&client, &url4).await?;

    Ok(())
}

async fn test_request(client: &Client, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("  URL: {}", url);
    
    let response = client.get(url)
        .header("accept", "application/json")
        .send()
        .await?;

    println!("  Status: {}", response.status());

    if response.status().is_success() {
        let json: Value = response.json().await?;
        
        if let Some(hits) = json.get("hits").and_then(|h| h.as_array()) {
            println!("  Hits: {}", hits.len());
            
            if !hits.is_empty() {
                println!("  Sample:");
                for (i, hit) in hits.iter().take(3).enumerate() {
                    if let Some(headline) = hit.get("headline").and_then(|h| h.as_str()) {
                        if let Some(mun) = hit.get("workplace_address")
                            .and_then(|a| a.get("municipality"))
                            .and_then(|m| m.as_str()) {
                            println!("    {}. {} (kommun: {})", i+1, headline, mun);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
