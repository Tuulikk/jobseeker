use crate::models::JobAd;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

pub struct JobSearchClient {
    client: Client,
    base_url: String,
}

const MUNICIPALITIES: &[(&str, &str)] = &[
    // Skåne (Befintliga + Fler)
    ("helsingborg", "1283"),
    ("ängelholm", "1292"),
    ("höganäs", "1284"),
    ("bjuv", "1260"),
    ("klippan", "1276"),
    ("åstorp", "1277"),
    ("örkelljunga", "1257"),
    ("båstad", "1278"),
    ("perstorp", "1275"),
    ("landskrona", "1282"),
    ("svalöv", "1214"),
    ("burlöv", "1231"),
    ("kävlinge", "1261"),
    ("malmö", "1280"),
    ("lund", "1281"),
    ("eslöv", "1285"),
    ("vellinge", "1233"),
    ("trelleborg", "1287"),
    ("ystad", "1286"),
    ("kristianstad", "1290"),
    ("hässleholm", "1293"),
    ("lomma", "1262"),
    ("staffanstorp", "1230"),
    ("svedala", "1263"),
    ("skurup", "1264"),
    ("sjöbo", "1265"),
    ("höör", "1267"),
    ("hörby", "1266"),
    ("tomelilla", "1270"),
    ("simrishamn", "1291"),
    ("osby", "1272"),
    ("östra göinge", "1273"),
    ("bromölla", "1271"),
    // Stor-Stockholm & Mälardalen
    ("stockholm", "0180"),
    ("huddinge", "0126"),
    ("nacka", "0182"),
    ("botkyrka", "0127"),
    ("haninge", "0136"),
    ("tyresö", "0138"),
    ("täby", "0160"),
    ("sollentuna", "0163"),
    ("järfälla", "0180"),
    ("solna", "0184"),
    ("upplands väsby", "0114"),
    ("södertälje", "0181"),
    ("lidingö", "0186"),
    ("sigtuna", "0191"),
    ("sundbyberg", "0115"),
    ("uppsala", "0380"),
    ("enköping", "0381"),
    ("västerås", "1980"),
    ("eskilstuna", "0484"),
    ("nyköping", "0480"),
    // Stor-Göteborg & Västkusten
    ("göteborg", "1480"),
    ("mölndal", "1481"),
    ("partille", "1402"),
    ("härryda", "1401"),
    ("kungälv", "1482"),
    ("lerum", "1441"),
    ("alingsås", "1489"),
    ("borås", "1490"),
    ("kungsbacka", "1384"),
    ("varberg", "1383"),
    ("halmstad", "1380"),
    ("uddevalla", "1485"),
    ("trollhättan", "1488"),
    ("skövde", "1496"),
    // Övriga Större Städer & Regioner
    ("linköping", "0580"),
    ("norrköping", "0581"),
    ("jönköping", "0680"),
    ("växjö", "0780"),
    ("kalmar", "0880"),
    ("karlskrona", "1080"),
    ("karlstad", "1780"),
    ("örebro", "1880"),
    ("falun", "2080"),
    ("borlänge", "2081"),
    ("gävle", "2180"),
    ("sundsvall", "2281"),
    ("östersund", "2380"),
    ("umeå", "2480"),
    ("skellefteå", "2482"),
    ("luleå", "2580"),
    ("öckerö", "1407"),
    ("stenungsund", "1415"),
    ("tjörn", "1419"),
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
        MUNICIPALITIES
            .iter()
            .find(|(n, _)| *n == name_lower.as_str())
            .map(|(_, c)| *c)
    }

    pub fn get_municipality_name(code: &str) -> Option<String> {
        MUNICIPALITIES
            .iter()
            .find(|(_, c)| *c == code)
            .map(|(n, _)| {
                let mut chars = n.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
    }

    pub async fn search(
        &self,
        query: &str,
        municipalities: &[String],
        limit: u32,
    ) -> Result<Vec<JobAd>> {
        let mut params = vec![("q", query.to_string()), ("limit", limit.to_string())];

        for m in municipalities {
            if !m.is_empty() {
                params.push(("municipality", m.to_string()));
            }
        }

        let url = format!("{}/search", self.base_url);

        let response = self
            .client
            .get(&url)
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

        let json: Value = response
            .json()
            .await
            .context("Failed to parse JSON response")?;

        let hits = json["hits"]
            .as_array()
            .context("No 'hits' array found in response")?;

        let mut ads = Vec::new();
        for hit in hits {
            let ad_val = hit.clone();

            // Extract webpage_url from root if not present in nested structs
            let webpage_url = hit["webpage_url"].as_str().map(|s| s.to_string());

            match serde_json::from_value::<JobAd>(ad_val.clone()) {
                Ok(mut ad) => {
                    ad.webpage_url = webpage_url;

                    // Extrahera working_hours_type om det saknas i automatisk deserialisering
                    if ad.working_hours_type.is_none()
                        && let Some(label) = hit["working_hours_type"]["label"].as_str()
                    {
                        ad.working_hours_type = Some(crate::models::WorkingHours {
                            label: Some(label.to_string()),
                        });
                    }

                    let find_section =
                        |keys: &[&str]| -> Option<String> { find_section_in_hit(hit, keys) };

                    // Kvalifikationer (om serde inte redan fyllt detta)
                    if ad.qualifications.is_none() {
                        let q_keys = [
                            "qualifications",
                            "qualification",
                            "requirements",
                            "skills",
                            "kvalifikationer",
                            "kvalifikation",
                            "kompetenskrav",
                            "krav",
                        ];
                        if let Some(q) = find_section(&q_keys) {
                            ad.qualifications = Some(q);
                        }
                    }

                    // Övrig information
                    if ad.additional_information.is_none() {
                        let o_keys = [
                            "other_information",
                            "additional_information",
                            "otherinfo",
                            "övrig_information",
                            "övrig",
                            "övrig information",
                            "övrigt",
                            "additional",
                        ];
                        if let Some(o) = find_section(&o_keys) {
                            ad.additional_information = Some(o);
                        }
                    }

                    // Försök även hitta omfattning om den saknas (kan förekomma som en sektion)
                    if ad.working_hours_type.is_none()
                        && let Some(omf) =
                            find_section(&["working_hours", "omfattning", "employment_type"])
                    {
                        let first_line = omf.lines().next().unwrap_or("").trim().to_string();
                        if !first_line.is_empty() {
                            ad.working_hours_type = Some(crate::models::WorkingHours {
                                label: Some(first_line),
                            });
                        }
                    }

                    ads.push(ad);
                }
                Err(e) => {
                    eprintln!("Error parsing job ad: {}. Value: {:?}", e, hit);
                }
            }
        }

        Ok(ads)
    }
}

fn find_section_in_hit(hit: &Value, keys: &[&str]) -> Option<String> {
    // Direkta nycklar (toppnivå eller nested i description)
    for k in keys {
        if let Some(s) = hit[k].as_str()
            && !s.trim().is_empty()
        {
            return Some(s.to_string());
        }
        if let Some(s) = hit["description"][k].as_str()
            && !s.trim().is_empty()
        {
            return Some(s.to_string());
        }
    }

    // description.sections (vanligt format för uppdelade sektioner)
    if let Some(secs) = hit["description"]["sections"].as_array() {
        for sec in secs {
            let sec_label = sec["heading"]
                .as_str()
                .or_else(|| sec["label"].as_str())
                .or_else(|| sec["title"].as_str())
                .unwrap_or("")
                .to_lowercase();

            for k in keys {
                if sec_label.contains(&k.to_lowercase()) {
                    if let Some(text) = sec["text"].as_str().or_else(|| sec["content"].as_str())
                        && !text.trim().is_empty()
                    {
                        return Some(text.to_string());
                    }
                    if let Some(pars) = sec["paragraphs"].as_array() {
                        let parts: Vec<_> = pars.iter().filter_map(|p| p.as_str()).collect();
                        if !parts.is_empty() {
                            return Some(parts.join("\n\n"));
                        }
                    }
                    if let Some(s) = sec.as_str()
                        && !s.trim().is_empty()
                    {
                        return Some(s.to_string());
                    }
                }
            }
        }
    }

    // sections på toppnivå (annan variant)
    if let Some(secs) = hit["sections"].as_array() {
        for sec in secs {
            let sec_label = sec["heading"]
                .as_str()
                .or_else(|| sec["label"].as_str())
                .or_else(|| sec["title"].as_str())
                .unwrap_or("")
                .to_lowercase();

            for k in keys {
                if sec_label.contains(&k.to_lowercase())
                    && let Some(text) = sec["text"].as_str().or_else(|| sec["content"].as_str())
                    && !text.trim().is_empty()
                {
                    return Some(text.to_string());
                }
            }
        }
    }

    None
}

impl Default for JobSearchClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_find_section_direct_key() {
        let hit = json!({
            "qualifications": "Erfarenhet inom support"
        });
        let keys = ["qualifications", "kvalifikationer"];
        assert_eq!(
            find_section_in_hit(&hit, &keys),
            Some("Erfarenhet inom support".to_string())
        );
    }

    #[test]
    fn test_find_section_description_sections() {
        let hit = json!({
            "description": {
                "sections": [
                    { "heading": "Kvalifikationer", "text": "Kvalifikationer text" },
                    { "heading": "Övrig information", "content": "Övrigt info" }
                ]
            }
        });
        let q_keys = ["kvalifikationer"];
        let o_keys = ["övrig", "other"];
        assert_eq!(
            find_section_in_hit(&hit, &q_keys),
            Some("Kvalifikationer text".to_string())
        );
        assert_eq!(
            find_section_in_hit(&hit, &o_keys),
            Some("Övrigt info".to_string())
        );
    }

    #[test]
    fn test_find_section_paragraphs() {
        let hit = json!({
            "description": {
                "sections": [
                    { "heading": "Kvalifikationer", "paragraphs": ["en rad", "andra raden"] }
                ]
            }
        });
        let q_keys = ["kvalifikationer"];
        assert_eq!(
            find_section_in_hit(&hit, &q_keys),
            Some("en rad\n\nandra raden".to_string())
        );
    }

    #[test]
    fn test_find_section_none() {
        let hit = json!({});
        let keys = ["nosuchthing"];
        assert_eq!(find_section_in_hit(&hit, &keys), None);
    }
}
