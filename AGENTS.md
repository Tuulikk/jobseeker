# Instructions for AI Agents 🤖

This file contains critical architectural decisions and API quirks that must be respected to avoid breaking the application.

## ⚠️ Critical: JobSearch API (JobTech)

The application communicates with the Swedish Public Employment Service (Arbetsförmedlingen) API. It has several quirks that have been hard-won through debugging:

1. **Search Query Logic (OR vs AND):**
   - By default, spaces in a query act as **AND** when combined with geographic filters.
   - To use **OR** logic for multiple keywords, you **MUST** use the format: `(word1 OR word2 OR word3)`.
   - **The parentheses are mandatory.** Without them, the API often returns 0 hits when a municipality filter is active.
   - Ref: See implementation in `src/lib.rs` -> `perform_search`.

2. **Municipalities:**
   - The API requires **numeric municipality codes** (e.g., "1283" for Helsingborg).
   - Do **NOT** send municipality names directly to the search endpoint.
   - Use `JobSearchClient::parse_locations` to resolve names to codes.

3. **Multi-Municipality Strategy:**
   - The current stable strategy is to perform one API call per municipality and merge the results. This ensures we don't hit "relevance" issues that the API has with complex multi-municipality queries.
   - Ref: `src/api.rs` -> `search_multi_municipalities`.

## 🧪 Testing as Source of Truth

Before making changes to the API communication or search logic, run the standalone test binaries:

- `cargo run --bin test_query_logic`: Verifies the OR-logic and Blacklist filtering.
- `cargo run --bin test_api_mini`: Verifies basic connectivity and multi-parameter handling.

If these tests fail, your changes have broken the core business value of the app.

## 🛠️ Data & Settings

- **Offline-First:** All search results are auto-saved to the local database in `perform_search`. This allows the app to function as an "inbox" when offline. Do NOT remove the `db.save_job_ad(&ad)` call during search.
- If the UI shows 0 hits for Prio-zones that should have data, the database might be corrupted.
- Use `cargo run --bin reset_settings` to restore known-good defaults.
- **Blacklist:** Be careful with keywords like "körkort" (driving license). In many regions/roles, this is a requirement and adding it to a blacklist will filter out almost all valid jobs.

---
*Motto: "Allting är relativt" - Gnaw your way through the problems, don't just patch them.*
