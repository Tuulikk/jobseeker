use reqwest::Client;
use serde_json::Value;

/// Testverktyg för att verifiera söklogik (OR vs AND) och Blacklist-filtrering.
/// Används för att säkerställa att Prio-zoner inte blir tomma pga för restriktiv sökning.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = "https://jobsearch.api.jobtechdev.se/search";
    
    // Simulera inställningar (motsvarar keywords i appen)
    let keywords = "it, support, helpdesk";
    
    // Bygg söksträng med OR-logik och parenteser
    let query_parts: Vec<_> = keywords.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    let query = if query_parts.len() > 1 {
        format!("({})", query_parts.join(" OR "))
    } else {
        query_parts.get(0).cloned().unwrap_or_default().to_string()
    };
    
    // Prio 1 kommuner (Helsingborgsområdet)
    let municipalities = vec!["1283", "1277", "1260", "1292", "1284", "1276", "1231", "1282", "1261"];

    // Blacklist från default settings (utan 'körkort')
    let blacklist_str = "databas, barnvakt, barnflicka, nanny, myNanny, undersköterska, parkarbetare";
    let blacklist: Vec<String> = blacklist_str.split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();

    println!("Jobseeker Söktest");
    println!("=================");
    println!("Sökord (råa):  {}", keywords);
    println!("Sökord (OR):   {}", query);
    println!("Kommuner:      {} st", municipalities.len());
    println!("Blacklist:     {:?}", blacklist);
    println!();

    let mut total_found = 0;
    let mut total_kept = 0;

    for mun in municipalities {
        // Vi använder .query() för att reqwest ska sköta URL-kodning av " OR " korrekt
        let params = [
            ("q", &query),
            ("municipality", &mun.to_string()),
            ("limit", &"50".to_string()),
        ];
        
        let response = client.get(base_url)
            .header("accept", "application/json")
            .query(&params)
            .send()
            .await?;
        
        if let Ok(json) = response.json::<Value>().await {
            if let Some(hits) = json["hits"].as_array() {
                if hits.is_empty() { continue; }
                
                let mun_name = hits[0]["workplace_address"]["municipality"].as_str().unwrap_or(mun);
                println!("Kommun {}: {} träffar från API", mun_name, hits.len());
                total_found += hits.len();
                
                for hit in hits {
                    let headline = hit["headline"].as_str().unwrap_or("").to_lowercase();
                    let desc = hit["description"]["text"].as_str().unwrap_or("").to_lowercase();
                    
                    let mut is_blacklisted = false;
                    for word in &blacklist {
                        if headline.contains(word) || desc.contains(word) {
                            is_blacklisted = true;
                            break;
                        }
                    }
                    
                    if !is_blacklisted {
                        total_kept += 1;
                    }
                }
            }
        }
    }
    
    println!();
    println!("RESULTAT:");
    println!("API hittade totalt: {} annonser", total_found);
    println!("Kvar efter blacklist: {} annonser", total_kept);
    
    if total_kept > 0 {
        println!("Status: OK (Sökningen fungerar)");
    } else {
        println!("Status: VARNING (Sökningen är fortfarande tom!)");
    }
    
    Ok(())
}