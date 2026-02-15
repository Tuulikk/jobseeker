use crate::models::JobAd;
use serde_json::Value;
use anyhow::{Result, Context};
use log::{info, error, warn};
use reqwest::Client;

// Använd JNI HTTP på Android för att använda Android's native network stack
#[cfg(target_os = "android")]
use crate::jni_http;

// Android uses reqwest for HTTP requests
// Desktop uses reqwest
pub struct JobSearchClient {
    #[cfg(not(target_os = "android"))]
    client: Client,
    #[cfg(not(target_os = "android"))]
    base_url: String,
    #[cfg(target_os = "android")]
    client: Client,
    #[cfg(target_os = "android")]
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

    /// ⚠️ GUARDED: JobTech API requires numeric municipality codes for filtering.
    /// Do not change this to send names directly. Use JobSearchClient::get_municipality_code
    /// to resolve names before calling search.
    #[cfg(not(target_os = "android"))]
    pub async fn search(&self, query: &str, municipalities: &[String], limit: u32) -> Result<Vec<JobAd>> {
        if municipalities.len() > 1 {
            // Multiple municipalities: do separate API calls per municipality and merge results
            return self.search_multi_municipalities(query, municipalities, limit).await;
        }

        // Single municipality (or empty): use original logic
        // ⚠️ HARD API CONSTRAINTS - DO NOT MODIFY:
        // 1. 'limit' MUST be <= 100. Values like 200 trigger HTTP 400 Bad Request.
        // 2. Do NOT add 'sort' parameter. The server rejects most values with HTTP 400.
        // 3. Keep queries simple. Complex boolean logic is handled by caller via individual calls.
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
        let request = self.client.get(&url)
            .header("accept", "application/json")
            .query(&params);

        // Log the full URL for debugging (with parameters)
        if let Some(req_builder) = request.try_clone() {
            if let Ok(req) = req_builder.build() {
                info!("Full API URL: {}", req.url());
            }
        }

        let response = match request.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Raw reqwest error: {:?}", e);
                    return Err(e).context("Failed to send request to JobSearch API");
                }
            };

        info!("API Response Status: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let body = match response.text().await {
                Ok(b) => b,
                Err(e) => format!("(failed to read error body: {})", e),
            };
            error!("API Error Detail: {}", body);
            return Err(anyhow::anyhow!("API returned HTTP {}", status));
        }

        let json = response.json::<Value>().await
            .context("Failed to parse JSON response")?;

        let hits = json["hits"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Response missing 'hits' array"))?;

        info!("API found {} raw hits", hits.len());

        let mut results = Vec::new();
        for hit in hits {
            let ad_val = hit.clone();
            let webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());

            if let Ok(mut ad) = serde_json::from_value::<JobAd>(ad_val.clone()) {
                ad.webpage_url = webpage_url;

                if ad.working_hours_type.is_none() {
                    if let Some(label) = hit["working_hours_type"]["label"].as_str() {
                        ad.working_hours_type = Some(crate::models::WorkingHours {
                            label: Some(label.to_string()),
                        });
                    }
                }

                results.push(ad);
            }
        }

        Ok(results)
    }

    #[cfg(target_os = "android")]
    pub async fn search(&self, query: &str, municipalities: &[String], limit: u32) -> Result<Vec<JobAd>> {
        if municipalities.len() > 1 {
            return self.search_multi_municipalities(query, municipalities, limit).await;
        }

        // ⚠️ HARD API CONSTRAINTS - DO NOT MODIFY:
        // 1. 'limit' MUST be <= 100. Values like 200 trigger HTTP 400 Bad Request.
        // 2. Do NOT add 'sort' parameter. The server rejects most values with HTTP 400.
        // 3. Keep queries simple. Complex boolean logic is handled by caller via individual calls.
        
        let m = municipalities.get(0).cloned().unwrap_or_default();
        let url = format!("{}/search?q={}&limit={}{}",
            self.base_url,
            urlencoding::encode(query),
            limit,
            if m.is_empty() { "".to_string() } else { format!("&municipality={}", m) }
        );
        
        info!("JNI Android HTTP search: {}", url);

        let response_text = crate::jni_http::http_get(&url)?;
        let json: Value = serde_json::from_str(&response_text)
            .context("Failed to parse JSON response from JNI HTTP")?;

        let hits = json["hits"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Response missing 'hits' array"))?;

        info!("API found {} hits via JNI", hits.len());

        let mut results = Vec::new();
        for hit in hits {
            let ad_val = hit.clone();
            let webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());

            if let Ok(mut ad) = serde_json::from_value::<JobAd>(ad_val.clone()) {
                ad.webpage_url = webpage_url;

                if ad.working_hours_type.is_none() {
                    if let Some(label) = hit["working_hours_type"]["label"].as_str() {
                        ad.working_hours_type = Some(crate::models::WorkingHours {
                            label: Some(label.to_string()),
                        });
                    }
                }

                results.push(ad);
            }
        }

        Ok(results)
    }

    /// Multi-municipality search - Desktop version using reqwest
    #[cfg(not(target_os = "android"))]
    async fn search_multi_municipalities(&self, query: &str, municipalities: &[String], limit: u32) -> Result<Vec<JobAd>> {
        use std::collections::HashSet;

        info!("Searching across {} municipalities (separate API calls)", municipalities.len());
        let mut all_ads = Vec::new();
        let mut seen_ids = HashSet::new();

        for m in municipalities {
            if m.is_empty() { continue; }

            // ⚠️ API CONSTRAINT: 'limit' must be <= 100 per call.
            // ⚠️ API CONSTRAINT: Do NOT add 'sort' parameter. It triggers 400 Bad Request.
            let params = vec![
                ("q", query.to_string()),
                ("limit", limit.to_string()),
                ("municipality", m.to_string()),
            ];

            let url = format!("{}/search", self.base_url);
            info!("Fetching for municipality {}: {}", m, url);

            let response = match self.client.get(&url)
                .header("accept", "application/json")
                .query(&params)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Raw reqwest error for municipality {}: {:?}", m, e);
                    return Err(e).with_context(|| format!("Failed to fetch for municipality {}", m));
                }
            };

            if !response.status().is_success() {
                warn!("Skipping municipality {} due to HTTP {}", m, response.status());
                continue;
            }

            if let Ok(json) = response.json::<Value>().await {
                if let Some(hits) = json["hits"].as_array() {
                    info!("Municipality {}: {} hits", m, hits.len());

                    for hit in hits {
                        let ad_val = hit.clone();
                        let webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());

                        if let Ok(mut ad) = serde_json::from_value::<JobAd>(ad_val.clone()) {
                            ad.webpage_url = webpage_url;

                            if ad.working_hours_type.is_none() {
                                if let Some(label) = hit["working_hours_type"]["label"].as_str() {
                                    ad.working_hours_type = Some(crate::models::WorkingHours {
                                        label: Some(label.to_string()),
                                    });
                                }
                            }

                            // Deduplicate by ad ID
                            if seen_ids.insert(ad.id.clone()) {
                                all_ads.push(ad);
                            }
                        }
                    }
                }
            }
        }

        info!("Total unique ads after merging {} municipalities: {}", municipalities.len(), all_ads.len());
        Ok(all_ads)
    }

    /// Multi-municipality search - Android version using JNI HTTP
    #[cfg(target_os = "android")]
    async fn search_multi_municipalities(&self, query: &str, municipalities: &[String], limit: u32) -> Result<Vec<JobAd>> {
        use std::collections::HashSet;

        info!("Searching across {} municipalities (JNI Android HTTP)", municipalities.len());
        let mut all_ads = Vec::new();
        let mut seen_ids = HashSet::new();

        for m in municipalities {
            if m.is_empty() { continue; }

            let url = format!("{}/search?q={}&limit={}&municipality={}",
                self.base_url,
                urlencoding::encode(query),
                limit,
                m
            );
            info!("Fetching for municipality {} via JNI: {}", m, url);

            match crate::jni_http::http_get(&url) {
                Ok(response_text) => {
                    info!("JNI HTTP SUCCESS! Response length: {}", response_text.len());
                    if let Ok(json) = serde_json::from_str::<Value>(&response_text) {
                        if let Some(hits) = json["hits"].as_array() {
                            info!("Municipality {}: {} hits", m, hits.len());

                            for hit in hits {
                                let ad_val = hit.clone();
                                let webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());

                                if let Ok(mut ad) = serde_json::from_value::<JobAd>(ad_val.clone()) {
                                    ad.webpage_url = webpage_url;

                                    if ad.working_hours_type.is_none() {
                                        if let Some(label) = hit["working_hours_type"]["label"].as_str() {
                                            ad.working_hours_type = Some(crate::models::WorkingHours {
                                                label: Some(label.to_string()),
                                            });
                                        }
                                    }

                                    if seen_ids.insert(ad.id.clone()) {
                                        all_ads.push(ad);
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("JNI HTTP failed for municipality {}: {}", m, e);
                    continue;
                }
            }
        }

        info!("Total unique ads after merging {} municipalities: {}", municipalities.len(), all_ads.len());
        Ok(all_ads)
    }
}
