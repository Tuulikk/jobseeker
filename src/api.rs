use reqwest::Client;
use crate::models::JobAd;
use serde_json::Value;
use anyhow::{Result, Context};

pub struct JobSearchClient {
    client: Client,
    base_url: String,
}

const MUNICIPALITIES: &[(&str, &str)] = &[
    // Skåne (Befintliga + Fler)
    ("helsingborg", "1283"), ("ängelholm", "1292"), ("höganäs", "1284"), ("bjuv", "1260"),
    ("klippan", "1276"), ("åstorp", "1277"), ("örkelljunga", "1257"), ("båstad", "1278"),
    ("perstorp", "1275"), ("landskrona", "1282"), ("svalöv", "1214"), ("burlöv", "1231"),
    ("kävlinge", "1261"), ("malmö", "1280"), ("lund", "1281"), ("eslöv", "1285"),
    ("vellinge", "1233"), ("trelleborg", "1287"), ("ystad", "1286"), ("kristianstad", "1290"),
    ("hässleholm", "1293"), ("lomma", "1262"), ("staffanstorp", "1230"), ("svedala", "1263"),
    ("skurup", "1264"), ("sjöbo", "1265"), ("höör", "1267"), ("hörby", "1266"),
    ("tomelilla", "1270"), ("simrishamn", "1291"), ("osby", "1272"), ("östra göinge", "1273"),
    ("bromölla", "1271"),

    // Stor-Stockholm & Mälardalen
    ("stockholm", "0180"), ("huddinge", "0126"), ("nacka", "0182"), ("botkyrka", "0127"),
    ("haninge", "0136"), ("tyresö", "0138"), ("täby", "0160"), ("sollentuna", "0163"),
    ("järfälla", "0123"), ("solna", "0184"), ("upplands väsby", "0114"), ("södertälje", "0181"),
    ("lidingö", "0186"), ("sigtuna", "0191"), ("sundbyberg", "0115"), ("uppsala", "0380"),
    ("enköping", "0381"), ("västerås", "1980"), ("eskilstuna", "0484"), ("nyköping", "0480"),

    // Stor-Göteborg & Västkusten
    ("göteborg", "1480"), ("mölndal", "1481"), ("partille", "1402"), ("härryda", "1401"),
    ("kungälv", "1482"), ("lerum", "1441"), ("alingsås", "1489"), ("borås", "1490"),
    ("kungsbacka", "1384"), ("varberg", "1383"), ("halmstad", "1380"), ("uddevalla", "1485"),
    ("trollhättan", "1488"), ("skövde", "1496"),

    // Övriga Större Städer & Regioner
    ("linköping", "0580"), ("norrköping", "0581"), ("jönköping", "0680"), ("växjö", "0780"),
    ("kalmar", "0880"), ("karlskrona", "1080"), ("karlstad", "1780"), ("örebro", "1880"),
    ("falun", "2080"), ("borlänge", "2081"), ("gävle", "2180"), ("sundsvall", "2281"),
    ("östersund", "2380"), ("umeå", "2480"), ("skellefteå", "2482"), ("luleå", "2580"),
    ("öckerö", "1407"), ("stenungsund", "1415"), ("tjörn", "1419"),
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

    pub fn parse_locations(input: &str) -> Vec<String> {
        input.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                // If it looks like a code (digits), keep it. Otherwise try to resolve name.
                if s.chars().all(char::is_numeric) {
                    s.to_string()
                } else {
                    Self::get_municipality_code(s).map(|c| c.to_string()).unwrap_or_default()
                }
            })
            .filter(|s| !s.is_empty())
            .collect()
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
        tracing::debug!("API Call: {} with params {:?}", url, params);

        let response = self.client.get(&url)
            .header("accept", "application/json")
            .query(&params)
            .send()
            .await
            .context("Failed to send request to JobSearch API")?;

        tracing::info!("API Response Status: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("API Error Detail: {}", body);
            return Err(anyhow::anyhow!("API Error: {} - {}", status, body));
        }

        let json: Value = response.json().await.context("Failed to parse JSON response")?;

        let hits = json["hits"].as_array()
            .context("No 'hits' array found in response")?;

        tracing::info!("API found {} raw hits", hits.len());

        let mut ads = Vec::new();
        for hit in hits {
            let ad_val = hit.clone();

            // Extract webpage_url from root if not present in nested structs
            let webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());

            match serde_json::from_value::<JobAd>(ad_val.clone()) {
                Ok(mut ad) => {
                    ad.webpage_url = webpage_url;

                    // Extrahera working_hours_type om det saknas i automatisk deserialisering
                    if ad.working_hours_type.is_none() {
                        if let Some(label) = hit["working_hours_type"]["label"].as_str() {
                            ad.working_hours_type = Some(crate::models::WorkingHours {
                                label: Some(label.to_string()),
                            });
                        }
                    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_locations_numeric_and_name() {
        // numeric codes should be preserved, names should be resolved to codes
        let parsed = JobSearchClient::parse_locations("1283, malmö");
        assert_eq!(parsed, vec!["1283".to_string(), "1280".to_string()]);
    }

    #[test]
    fn parse_locations_ignores_empty_and_trims() {
        // empty entries and whitespace should be ignored
        let parsed = JobSearchClient::parse_locations(" , 1283,  malmö  , ");
        assert_eq!(parsed, vec!["1283".to_string(), "1280".to_string()]);
    }
}
