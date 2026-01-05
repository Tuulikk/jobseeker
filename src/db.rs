use crate::models::{
    AdStatus, ApplicationDetails, Description, Employer, JobAd, Occupation, WorkingHours,
    WorkplaceAddress,
};
use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use sqlx::{Row, sqlite::SqlitePool};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(db_url).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS job_ads (
                id TEXT PRIMARY KEY,
                headline TEXT NOT NULL,
                description TEXT,
                employer_name TEXT,
                employer_workplace TEXT,
                application_url TEXT,
                webpage_url TEXT,
                publication_date TEXT,
                last_application_date TEXT,
                occupation_label TEXT,
                city TEXT,
                municipality TEXT,
                working_hours_label TEXT,
                is_read BOOLEAN DEFAULT 0,
                rating INTEGER,
                bookmarked_at TEXT,
                internal_created_at TEXT NOT NULL,
                search_keyword TEXT,
                status INTEGER DEFAULT 0,
                applied_at TEXT
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS job_applications (
                job_id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(job_id) REFERENCES job_ads(id)
            )",
        )
        .execute(&pool)
        .await?;

        // Säkerställ att kolumner finns för äldre databaser
        let _ = sqlx::query("ALTER TABLE job_ads ADD COLUMN search_keyword TEXT")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE job_ads ADD COLUMN status INTEGER DEFAULT 0")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE job_ads ADD COLUMN applied_at TEXT")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE job_ads ADD COLUMN municipality TEXT")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE job_ads ADD COLUMN working_hours_label TEXT")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE job_ads ADD COLUMN webpage_url TEXT")
            .execute(&pool)
            .await;

        Ok(Self { pool })
    }

    pub async fn save_application_draft(&self, job_id: &str, content: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO job_applications (job_id, content, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(job_id) DO UPDATE SET
                content = excluded.content,
                updated_at = excluded.updated_at",
        )
        .bind(job_id)
        .bind(content)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_application_draft(&self, job_id: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT content FROM job_applications WHERE job_id = ?")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("content")))
    }

    pub async fn get_all_drafts(&self) -> Result<Vec<(String, String, String)>> {
        let rows = sqlx::query(
            "SELECT ja.job_id, ja.updated_at, ads.headline
             FROM job_applications ja
             JOIN job_ads ads ON ja.job_id = ads.id
             ORDER BY ja.updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.get("job_id"), r.get("headline"), r.get("updated_at")))
            .collect())
    }

    pub async fn save_job_ad(&self, ad: &JobAd) -> Result<()> {
        sqlx::query(
            "INSERT INTO job_ads (
                id, headline, description, employer_name, employer_workplace,
                application_url, webpage_url, publication_date, last_application_date,
                occupation_label, city, municipality, is_read, rating, bookmarked_at,
                internal_created_at, search_keyword, status, applied_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                headline = excluded.headline,
                description = excluded.description,
                employer_name = excluded.employer_name,
                employer_workplace = excluded.employer_workplace,
                application_url = excluded.application_url,
                webpage_url = excluded.webpage_url,
                publication_date = excluded.publication_date,
                last_application_date = excluded.last_application_date,
                occupation_label = excluded.occupation_label,
                city = excluded.city,
                municipality = excluded.municipality,
                search_keyword = COALESCE(job_ads.search_keyword, excluded.search_keyword)",
        )
        .bind(&ad.id)
        .bind(&ad.headline)
        .bind(ad.description.as_ref().and_then(|d| d.text.as_ref()))
        .bind(ad.employer.as_ref().and_then(|e| e.name.as_ref()))
        .bind(ad.employer.as_ref().and_then(|e| e.workplace.as_ref()))
        .bind(ad.application_details.as_ref().and_then(|a| a.url.as_ref()))
        .bind(&ad.webpage_url)
        .bind(&ad.publication_date)
        .bind(&ad.last_application_date)
        .bind(ad.occupation.as_ref().and_then(|o| o.label.as_ref()))
        .bind(ad.workplace_address.as_ref().and_then(|w| w.city.as_ref()))
        .bind(
            ad.workplace_address
                .as_ref()
                .and_then(|w| w.municipality.as_ref()),
        )
        .bind(ad.is_read)
        .bind(ad.rating.map(|r| r as i32))
        .bind(ad.bookmarked_at.map(|d| d.to_rfc3339()))
        .bind(ad.internal_created_at.to_rfc3339())
        .bind(&ad.search_keyword)
        .bind(ad.status.unwrap_or(AdStatus::New) as i32)
        .bind(ad.applied_at.map(|d| d.to_rfc3339()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_filtered_jobs(
        &self,
        status_filter: &[AdStatus],
        year: Option<i32>,
        month: Option<u32>,
    ) -> Result<Vec<JobAd>> {
        // För month-filter och när en enskild status efterfrågas vill vi ta med historiska rader
        // (t.ex. visa alla jobb som har `applied_at` satt för en månad även om status senare ändrats).
        let query_str = if year.is_some() && month.is_some() && status_filter.len() == 1 {
            match status_filter[0] {
                AdStatus::Applied => {
                    "SELECT * FROM job_ads WHERE applied_at IS NOT NULL ORDER BY publication_date DESC"
                        .to_string()
                }
                AdStatus::Bookmarked | AdStatus::ThumbsUp => {
                    "SELECT * FROM job_ads WHERE bookmarked_at IS NOT NULL ORDER BY publication_date DESC"
                        .to_string()
                }
                _ => {
                    let status_ints: Vec<i32> = status_filter.iter().map(|s| *s as i32).collect();
                    let placeholders = status_ints
                        .iter()
                        .map(|_| "?")
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        "SELECT * FROM job_ads WHERE status IN ({}) ORDER BY publication_date DESC",
                        placeholders
                    )
                }
            }
        } else {
            if status_filter.is_empty() {
                "SELECT * FROM job_ads WHERE status != 1 ORDER BY publication_date DESC".to_string()
            } else {
                let status_ints: Vec<i32> = status_filter.iter().map(|s| *s as i32).collect();
                let placeholders = status_ints
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "SELECT * FROM job_ads WHERE status IN ({}) ORDER BY publication_date DESC",
                    placeholders
                )
            }
        };

        let mut query = sqlx::query(&query_str);
        if !status_filter.is_empty() {
            for s in status_filter {
                query = query.bind(*s as i32);
            }
        }

        let rows = query.fetch_all(&self.pool).await?;
        let mut ads = Vec::new();

        for row in rows {
            let ad = self.map_row_to_ad(row)?;

            if let (Some(y), Some(m)) = (year, month) {
                // Bestäm vilken tidsstämpel vi ska jämföra mot:
                // - Om användaren specifikt efterfrågar t.ex. `Applied` så vill vi matcha `applied_at`,
                //   även om `status` senare ändrats (så historiska objekt visas korrekt).
                // - Om flera/ingen status specificerats så använder vi en prioritering:
                //   applied_at -> bookmarked_at -> internal_created_at.
                let date_to_check = if status_filter.len() == 1 {
                    match status_filter[0] {
                        AdStatus::Applied => ad.applied_at,
                        AdStatus::Bookmarked | AdStatus::ThumbsUp => ad.bookmarked_at,
                        _ => Some(ad.internal_created_at),
                    }
                } else {
                    ad.applied_at
                        .or(ad.bookmarked_at)
                        .or(Some(ad.internal_created_at))
                };

                if let Some(dt) = date_to_check {
                    if dt.year() == y && dt.month() == m {
                        ads.push(ad);
                    }
                }
            } else {
                // Inget tidsfilter, inkludera alla
                ads.push(ad);
            }
        }
        Ok(ads)
    }

    pub async fn update_ad_status(&self, id: &str, status: AdStatus) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        match status {
            AdStatus::Applied => {
                sqlx::query("UPDATE job_ads SET status = ?, applied_at = ? WHERE id = ?")
                    .bind(status as i32)
                    .bind(now)
                    .bind(id)
                    .execute(&self.pool)
                    .await?;
            }
            AdStatus::Bookmarked | AdStatus::ThumbsUp => {
                sqlx::query("UPDATE job_ads SET status = ?, bookmarked_at = ? WHERE id = ?")
                    .bind(status as i32)
                    .bind(now)
                    .bind(id)
                    .execute(&self.pool)
                    .await?;
            }
            _ => {
                sqlx::query("UPDATE job_ads SET status = ? WHERE id = ?")
                    .bind(status as i32)
                    .bind(id)
                    .execute(&self.pool)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn mark_as_read(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE job_ads SET is_read = 1 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_rating(&self, id: &str, rating: u8) -> Result<()> {
        sqlx::query("UPDATE job_ads SET rating = ? WHERE id = ?")
            .bind(rating as i32)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn clear_non_bookmarked(&self) -> Result<()> {
        sqlx::query("DELETE FROM job_ads WHERE status IN (0, 1)")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    fn map_row_to_ad(&self, row: sqlx::sqlite::SqliteRow) -> Result<JobAd> {
        let created_at_str: String = row
            .try_get("internal_created_at")
            .unwrap_or_else(|_| Utc::now().to_rfc3339());
        let internal_created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let status_int: i32 = row.try_get("status").unwrap_or(0);
        let status = match status_int {
            1 => AdStatus::Rejected,
            2 => AdStatus::Bookmarked,
            3 => AdStatus::ThumbsUp,
            4 => AdStatus::Applied,
            _ => AdStatus::New,
        };

        Ok(JobAd {
            id: row.try_get("id").unwrap_or_default(),
            headline: row.try_get("headline").unwrap_or_default(),
            description: Some(Description {
                text: row.try_get("description").ok(),
            }),
            employer: Some(Employer {
                name: row.try_get("employer_name").ok(),
                workplace: row.try_get("employer_workplace").ok(),
            }),
            application_details: Some(ApplicationDetails {
                url: row.try_get("application_url").ok(),
            }),
            webpage_url: row.try_get("webpage_url").ok(),
            publication_date: row.try_get("publication_date").unwrap_or_default(),
            last_application_date: row.try_get("last_application_date").ok(),
            occupation: Some(Occupation {
                label: row.try_get("occupation_label").ok(),
            }),
            workplace_address: Some(WorkplaceAddress {
                city: row.try_get("city").ok(),
                municipality: row.try_get("municipality").ok(),
            }),
            working_hours_type: Some(WorkingHours {
                label: row.try_get("working_hours_label").ok(),
            }),
            is_read: row.try_get("is_read").unwrap_or(false),
            rating: row
                .try_get::<Option<i32>, _>("rating")
                .ok()
                .flatten()
                .map(|r| r as u8),
            bookmarked_at: row
                .try_get::<Option<String>, _>("bookmarked_at")
                .ok()
                .flatten()
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            internal_created_at,
            search_keyword: row.try_get("search_keyword").ok(),
            status: Some(status),
            applied_at: row
                .try_get::<Option<String>, _>("applied_at")
                .ok()
                .flatten()
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use chrono::Utc;

    // Async test that exercises saving and loading an application draft via the DB API using an in-memory DB.
    #[tokio::test]
    async fn test_save_and_get_application_draft() -> Result<()> {
        // Use in-memory SQLite to avoid filesystem dependencies in tests.
        let db = Db::new("sqlite::memory:").await?;

        // Create and save a minimal job ad (so the draft join logic in get_all_drafts works)
        let job_id = "test-draft-1";
        let ad = JobAd {
            id: job_id.to_string(),
            headline: "Test Draft".to_string(),
            description: None,
            employer: None,
            application_details: None,
            webpage_url: None,
            publication_date: Utc::now().to_rfc3339(),
            last_application_date: None,
            occupation: None,
            workplace_address: None,
            working_hours_type: None,
            is_read: false,
            rating: None,
            bookmarked_at: None,
            internal_created_at: Utc::now(),
            search_keyword: None,
            status: None,
            applied_at: None,
        };
        db.save_job_ad(&ad).await?;

        // Save draft content
        let content = "Detta är ett testutkast";
        db.save_application_draft(job_id, content).await?;

        // Retrieve draft and assert it matches
        let loaded = db.get_application_draft(job_id).await?;
        assert_eq!(loaded, Some(content.to_string()));

        Ok(())
    }
}
