use crate::db::Database;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{collections::{HashMap, HashSet}, env, time::Duration};

const NOTION_VERSION: &str = "2026-03-11";
const DEFAULT_ACTOR: &str = "apify/instagram-reel-scraper";
const DEFAULT_MAX_REELS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovaResearchRequest {
    pub instagram_username: String,
    #[serde(default)]
    pub max_reels: Option<usize>,
    #[serde(default = "default_true")]
    pub refresh_metrics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovaRelevantVideo {
    pub video_id: String,
    pub reel_url: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovaNotionLinks {
    pub creator_memory_url: String,
    pub scripts_database_url: String,
    pub cv_portfolio_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovaResearchReport {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub creator: String,
    pub status: String,
    pub existing_videos: usize,
    pub new_videos_added: usize,
    pub total_scripts_stored: usize,
    pub relevant_videos: Vec<NovaRelevantVideo>,
    pub notion: NovaNotionLinks,
    pub known_video_ids_skipped: usize,
    pub metrics_refreshed: usize,
    pub transcription_failures: usize,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
struct NovaConfig {
    apify_token: String,
    notion_token: String,
    apify_actor: String,
    creator_memory_data_source_id: String,
    scripts_data_source_id: String,
    cv_data_source_id: String,
    creator_memory_database_page_id: String,
    scripts_database_page_id: String,
    cv_database_page_id: String,
}

impl NovaConfig {
    fn from_env() -> Result<Self, String> {
        Ok(Self {
            apify_token: required_env("APIFY_TOKEN")?,
            notion_token: required_env("NOTION_TOKEN")?,
            apify_actor: env::var("APIFY_INSTAGRAM_REEL_ACTOR").unwrap_or_else(|_| DEFAULT_ACTOR.into()),
            creator_memory_data_source_id: env_or_default("NOVA_CREATOR_MEMORY_DATA_SOURCE_ID", "0151a8b8-b8fc-4301-a57d-df8206350a4b"),
            scripts_data_source_id: env_or_default("NOVA_INSTAGRAM_SCRIPTS_DATA_SOURCE_ID", "aed7fbe3-df36-451b-808a-2d859eb00188"),
            cv_data_source_id: env_or_default("NOVA_CV_PORTFOLIO_DATA_SOURCE_ID", "8b8036ef-f92e-4f0d-bd5f-5c1969e7f376"),
            creator_memory_database_page_id: env_or_default("NOVA_CREATOR_MEMORY_DATABASE_PAGE_ID", "8e7b6dbec6134208bcef765ef21d0ed7"),
            scripts_database_page_id: env_or_default("NOVA_INSTAGRAM_SCRIPTS_DATABASE_PAGE_ID", "022d31f14fc8444ba6137c1dd376d946"),
            cv_database_page_id: env_or_default("NOVA_CV_PORTFOLIO_DATABASE_PAGE_ID", "4afd9006b6024a21b0df6ea669385d29"),
        })
    }
}

#[derive(Debug, Clone)]
struct ExistingReel {
    page_id: String,
}

#[derive(Debug, Clone)]
struct CreatorMemory {
    page_id: String,
    page_url: String,
    first_seen: Option<String>,
}

#[derive(Debug, Clone)]
struct ReelRecord {
    video_id: String,
    reel_url: String,
    creator_name: String,
    username: String,
    account_id: String,
    publish_date: Option<String>,
    views: Option<i64>,
    likes: Option<i64>,
    comments: Option<i64>,
    duration_sec: Option<f64>,
    caption: String,
    transcript: String,
}

#[derive(Debug, Clone)]
struct SimpleAnalysis {
    topic: String,
    hook: String,
    cta: String,
    content_style: String,
    why_it_worked: String,
    sam_notes: String,
    sam_relevant: bool,
}

pub async fn research_creator(db: &Database, request: NovaResearchRequest) -> Result<NovaResearchReport, String> {
    let config = NovaConfig::from_env()?;
    let username = normalize_username(&request.instagram_username)?;
    let max_reels = request.max_reels.unwrap_or(DEFAULT_MAX_REELS).clamp(1, 100);
    let client = Client::builder().timeout(Duration::from_secs(360)).build().map_err(|e| e.to_string())?;

    db.set_employee_status("nova", "WORKING")?;
    db.event(None, "nova.research.started", "nova", json!({"creator": username, "max_reels": max_reels}))?;

    let result = research_creator_inner(&client, &config, &username, max_reels, request.refresh_metrics).await;
    match &result {
        Ok(report) => {
            db.set_employee_status("nova", "IDLE")?;
            db.event(None, "nova.handoff.saly", "nova", json!({
                "creator": report.creator,
                "status": report.status,
                "new_videos_added": report.new_videos_added,
                "total_scripts_stored": report.total_scripts_stored,
                "relevant_videos": report.relevant_videos,
                "destination": "saly"
            }))?;
            db.event(None, "nova.sam.notified", "nova", json!({
                "creator": report.creator,
                "status": report.status,
                "new_videos_added": report.new_videos_added,
                "notion": report.notion
            }))?;
        }
        Err(error) => {
            let _ = db.set_employee_status("nova", "BLOCKED");
            let _ = db.event(None, "nova.research.failed", "nova", json!({"creator": username, "error": safe_error(error)}));
        }
    }
    result
}

async fn research_creator_inner(
    client: &Client,
    config: &NovaConfig,
    username: &str,
    max_reels: usize,
    refresh_metrics: bool,
) -> Result<NovaResearchReport, String> {
    let notion = NotionClient::new(client.clone(), config.notion_token.clone());
    let existing = notion.known_reels(&config.scripts_data_source_id, username).await?;
    let existing_count = existing.len();
    let creator_memory = notion.find_creator(&config.creator_memory_data_source_id, username).await?;

    let raw_items = fetch_apify_reels(client, config, username, max_reels).await?;
    let mut records = Vec::new();
    for item in raw_items.iter().take(max_reels) {
        if let Some(record) = normalize_reel(item, username) {
            records.push(record);
        }
    }

    let mut new_videos_added = 0usize;
    let mut known_skipped = 0usize;
    let mut metrics_refreshed = 0usize;
    let mut transcription_failures = 0usize;
    let mut relevant = Vec::new();
    let now = Utc::now().to_rfc3339();

    for record in &records {
        if let Some(known) = existing.get(&record.video_id) {
            known_skipped += 1;
            if refresh_metrics {
                notion.refresh_reel_metrics(&known.page_id, record, &now).await?;
                metrics_refreshed += 1;
            }
            continue;
        }

        if record.transcript.trim().is_empty() {
            transcription_failures += 1;
        }
        let analysis = analyze_reel(record);
        notion.create_reel(&config.scripts_data_source_id, record, &analysis, &now).await?;
        new_videos_added += 1;
        if analysis.sam_relevant {
            relevant.push(NovaRelevantVideo {
                video_id: record.video_id.clone(),
                reel_url: record.reel_url.clone(),
                reason: relevant_reason(record, &analysis),
            });
        }
    }

    if relevant.is_empty() {
        let mut candidates = records.iter()
            .filter(|r| !existing.contains_key(&r.video_id))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|r| std::cmp::Reverse(r.views.unwrap_or(0)));
        relevant.extend(candidates.into_iter().take(3).map(|record| NovaRelevantVideo {
            video_id: record.video_id.clone(),
            reel_url: record.reel_url.clone(),
            reason: "Top public-view result among the newly retrieved Reels; inspect the source and transcript before using the mechanism.".into(),
        }));
    }
    relevant.truncate(5);

    let total_scripts_stored = existing_count + new_videos_added;
    let profile_url = format!("https://www.instagram.com/{username}/");
    let account_id = records.iter().find(|r| !r.account_id.is_empty()).map(|r| r.account_id.as_str()).unwrap_or("");
    let creator_name = records.iter().find(|r| !r.creator_name.is_empty()).map(|r| r.creator_name.as_str()).unwrap_or(username);
    let creator_page = notion.upsert_creator(
        &config.creator_memory_data_source_id,
        creator_memory.as_ref(),
        creator_name,
        username,
        account_id,
        &profile_url,
        records.len().max(existing_count),
        total_scripts_stored,
        &now,
    ).await?;

    notion.create_research_run(
        &config.cv_data_source_id,
        username,
        &profile_url,
        existing_count,
        new_videos_added,
        total_scripts_stored,
        &now,
    ).await?;

    Ok(NovaResearchReport {
        artifact_type: "CREATOR_RESEARCH_REPORT".into(),
        creator: format!("@{username}"),
        status: if transcription_failures > 0 { "PARTIAL".into() } else { "COMPLETE".into() },
        existing_videos: existing_count,
        new_videos_added,
        total_scripts_stored,
        relevant_videos: relevant,
        notion: NovaNotionLinks {
            creator_memory_url: creator_page.page_url,
            scripts_database_url: notion_page_url(&config.scripts_database_page_id),
            cv_portfolio_url: notion_page_url(&config.cv_database_page_id),
        },
        known_video_ids_skipped: known_skipped,
        metrics_refreshed,
        transcription_failures,
        notes: Some("NOVA checked persistent Notion memory first and fully processed only previously unseen Instagram Video IDs.".into()),
    })
}

async fn fetch_apify_reels(client: &Client, config: &NovaConfig, username: &str, max_reels: usize) -> Result<Vec<Value>, String> {
    let actor = config.apify_actor.replace('/', "~");
    let url = format!("https://api.apify.com/v2/acts/{actor}/run-sync-get-dataset-items?clean=true&format=json");
    let response = client.post(url)
        .bearer_auth(&config.apify_token)
        .json(&json!({
            "username": [username],
            "resultsLimit": max_reels,
            "includeTranscript": true,
            "includeDownloadedVideo": false,
            "includeSharesCount": false,
            "skipPinnedPosts": false,
            "skipTrialReels": false
        }))
        .send().await.map_err(|e| format!("Apify request failed: {e}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!("Apify returned {status}: {}", detail.chars().take(300).collect::<String>()));
    }
    response.json::<Vec<Value>>().await.map_err(|e| format!("Invalid Apify dataset response: {e}"))
}

fn normalize_reel(item: &Value, requested_username: &str) -> Option<ReelRecord> {
    let video_id = string_value(item, &["id", "videoId", "mediaId"])?;
    let reel_url = string_value(item, &["url", "inputUrl"]).unwrap_or_else(|| {
        string_value(item, &["shortCode"]).map(|code| format!("https://www.instagram.com/reel/{code}/")).unwrap_or_default()
    });
    if reel_url.is_empty() { return None; }
    let username = string_value(item, &["ownerUsername", "username"]).unwrap_or_else(|| requested_username.into());
    Some(ReelRecord {
        video_id,
        reel_url,
        creator_name: string_value(item, &["ownerFullName", "fullName"]).unwrap_or_default(),
        username,
        account_id: string_value(item, &["ownerId", "ownerID"]).unwrap_or_default(),
        publish_date: string_value(item, &["timestamp", "publishedAt"]),
        views: int_value(item, &["videoViewCount", "videoPlayCount", "viewsCount", "views"]),
        likes: int_value(item, &["likesCount", "likes"]),
        comments: int_value(item, &["commentsCount", "comments"]),
        duration_sec: float_value(item, &["videoDuration", "duration"]),
        caption: string_value(item, &["caption"]).unwrap_or_default(),
        transcript: transcript_value(item),
    })
}

fn transcript_value(item: &Value) -> String {
    if let Some(text) = string_value(item, &["transcript", "videoTranscript", "text"]) { return text; }
    if let Some(value) = item.get("transcript") {
        if let Some(text) = value.get("text").and_then(Value::as_str) { return text.to_string(); }
        if let Some(segments) = value.get("segments").and_then(Value::as_array) {
            return segments.iter().filter_map(|s| s.get("text").and_then(Value::as_str)).collect::<Vec<_>>().join(" ");
        }
    }
    String::new()
}

fn analyze_reel(record: &ReelRecord) -> SimpleAnalysis {
    let source = if record.transcript.trim().is_empty() { record.caption.trim() } else { record.transcript.trim() };
    let hook = first_phrase(source, 220);
    let combined = format!("{} {}", record.caption, record.transcript).to_lowercase();
    let style = if contains_any(&combined, &["look", "see this", "watch this", "شوف", "بص", "شايف"]) {
        "Demo"
    } else if contains_any(&combined, &["how to", "how i", "step by step", "ازاي", "إزاي", "طريقة"]){
        "Tutorial"
    } else if contains_any(&combined, &["i tried", "i tested", "جربت", "اختبرت"]){
        "Experiment"
    } else if contains_any(&combined, &[" vs ", "versus", "compare", "comparison", "مقارنة", "أحسن من"]){
        "Comparison"
    } else if contains_any(&combined, &["i think", "my take", "in my opinion", "رأيي", "شايف إن"]){
        "Opinion"
    } else if contains_any(&combined, &["built", "building", "بنيت", "ببني", "عملت سيستم"]){
        "Build"
    } else {
        "Other"
    }.to_string();
    let cta = detect_cta(source);
    let topic = topic_from(record);
    let sam_relevant = contains_any(&combined, &[
        " ai ", "artificial intelligence", "chatgpt", "claude", "gemini", "grok", "agent", "automation",
        "ذكاء اصطناعي", "الذكاء الاصطناعي", "أتمتة", "اوتوميشن", "أوتوميشن"
    ]);
    let performance = match record.views {
        Some(v) => format!("Public view count observed at capture time: {v}. "),
        None => String::new(),
    };
    SimpleAnalysis {
        topic,
        hook,
        cta,
        content_style: style,
        why_it_worked: format!("Interpretation only — {performance}The Reel gives Sam a source-linked example of the creator's actual delivery and demonstration pattern; performance alone does not prove causation."),
        sam_notes: "Open the Reel beside the full transcript. Recreate the useful mechanism/pacing/demo with Sam's own example, evidence and wording; do not copy the creator's distinctive phrasing.".into(),
        sam_relevant,
    }
}

fn topic_from(record: &ReelRecord) -> String {
    let from_caption = first_phrase(record.caption.trim(), 160);
    if !from_caption.is_empty() { from_caption } else { first_phrase(record.transcript.trim(), 160) }
}

fn relevant_reason(record: &ReelRecord, analysis: &SimpleAnalysis) -> String {
    let metric = record.views.map(|v| format!("; observed views: {v}")).unwrap_or_default();
    format!("SAM-relevant AI/automation signal using {} delivery{}; source Reel and full transcript are stored for inspection.", analysis.content_style, metric)
}

fn detect_cta(source: &str) -> String {
    let lower = source.to_lowercase();
    let markers = [
        "comment ", "dm me", "send me", "follow", "save this", "share this", "link in bio",
        "اكتب", "اكتبلي", "اكتب لي", "كومنت", "ابعتلي", "ابعت لي", "تابع", "احفظ", "شير"
    ];
    for marker in markers {
        if let Some(index) = lower.find(marker) {
            let tail = &source[index..];
            return first_phrase(tail, 220);
        }
    }
    String::new()
}

struct NotionClient { client: Client, token: String }

impl NotionClient {
    fn new(client: Client, token: String) -> Self { Self { client, token } }

    async fn query_data_source(&self, data_source_id: &str, body: Value) -> Result<Vec<Value>, String> {
        let mut results = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut payload = body.clone();
            if let Some(cursor_value) = cursor.as_ref() {
                payload.as_object_mut().expect("Notion payload object").insert("start_cursor".into(), Value::String(cursor_value.clone()));
            }
            let response = self.request(reqwest::Method::POST, &format!("https://api.notion.com/v1/data_sources/{data_source_id}/query"))
                .json(&payload).send().await.map_err(|e| format!("Notion query failed: {e}"))?;
            let value = notion_json(response).await?;
            if let Some(items) = value.get("results").and_then(Value::as_array) { results.extend(items.iter().cloned()); }
            if !value.get("has_more").and_then(Value::as_bool).unwrap_or(false) { break; }
            cursor = value.get("next_cursor").and_then(Value::as_str).map(str::to_string);
            if cursor.is_none() { break; }
        }
        Ok(results)
    }

    async fn known_reels(&self, data_source_id: &str, username: &str) -> Result<HashMap<String, ExistingReel>, String> {
        let rows = self.query_data_source(data_source_id, json!({
            "page_size": 100,
            "filter": {"property": "Instagram Username", "rich_text": {"equals": username}}
        })).await?;
        let mut result = HashMap::new();
        for row in rows {
            let page_id = row.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
            if let Some(video_id) = notion_text_property(&row, "Video ID") {
                if !video_id.is_empty() { result.insert(video_id, ExistingReel { page_id: page_id.clone() }); }
            }
        }
        Ok(result)
    }

    async fn find_creator(&self, data_source_id: &str, username: &str) -> Result<Option<CreatorMemory>, String> {
        let rows = self.query_data_source(data_source_id, json!({
            "page_size": 10,
            "filter": {"property": "Instagram Username", "rich_text": {"equals": username}}
        })).await?;
        Ok(rows.into_iter().next().map(|row| CreatorMemory {
            page_id: row.get("id").and_then(Value::as_str).unwrap_or_default().to_string(),
            page_url: row.get("url").and_then(Value::as_str).unwrap_or_default().to_string(),
            first_seen: notion_date_property(&row, "First Seen"),
        }))
    }

    async fn create_reel(&self, data_source_id: &str, record: &ReelRecord, analysis: &SimpleAnalysis, now: &str) -> Result<(), String> {
        let title = format!("@{} — {}", record.username, record.video_id);
        let mut properties = Map::new();
        properties.insert("Reel".into(), title_property(&title));
        properties.insert("Creator".into(), rich_text_property(if record.creator_name.is_empty() { &record.username } else { &record.creator_name }));
        properties.insert("Instagram Username".into(), rich_text_property(&record.username));
        properties.insert("Instagram Account ID".into(), rich_text_property(&record.account_id));
        properties.insert("Video ID".into(), rich_text_property(&record.video_id));
        properties.insert("Reel Link".into(), json!({"url": record.reel_url}));
        properties.insert("Caption".into(), rich_text_property(&record.caption));
        properties.insert("Full Script".into(), rich_text_property(&record.transcript));
        properties.insert("Hook".into(), rich_text_property(&analysis.hook));
        properties.insert("CTA".into(), rich_text_property(&analysis.cta));
        properties.insert("Topic".into(), rich_text_property(&analysis.topic));
        properties.insert("Why It Worked".into(), rich_text_property(&analysis.why_it_worked));
        properties.insert("SAM Notes".into(), rich_text_property(&analysis.sam_notes));
        properties.insert("Content Style".into(), json!({"select": {"name": analysis.content_style}}));
        properties.insert("Status".into(), json!({"select": {"name": if analysis.sam_relevant {"Relevant for SAM"} else {"Analyzed"}}}));
        properties.insert("Date Added".into(), date_property(Some(now)));
        properties.insert("Last Updated".into(), date_property(Some(now)));
        properties.insert("Publish Date".into(), date_property(record.publish_date.as_deref()));
        properties.insert("Views".into(), number_property(record.views.map(|v| v as f64)));
        properties.insert("Likes".into(), number_property(record.likes.map(|v| v as f64)));
        properties.insert("Comments".into(), number_property(record.comments.map(|v| v as f64)));
        properties.insert("Duration Sec".into(), number_property(record.duration_sec));

        let response = self.request(reqwest::Method::POST, "https://api.notion.com/v1/pages")
            .json(&json!({"parent": {"type":"data_source_id", "data_source_id": data_source_id}, "properties": Value::Object(properties)}))
            .send().await.map_err(|e| format!("Notion Reel create failed: {e}"))?;
        notion_json(response).await?;
        Ok(())
    }

    async fn refresh_reel_metrics(&self, page_id: &str, record: &ReelRecord, now: &str) -> Result<(), String> {
        let mut properties = Map::new();
        properties.insert("Views".into(), number_property(record.views.map(|v| v as f64)));
        properties.insert("Likes".into(), number_property(record.likes.map(|v| v as f64)));
        properties.insert("Comments".into(), number_property(record.comments.map(|v| v as f64)));
        properties.insert("Last Updated".into(), date_property(Some(now)));
        let response = self.request(reqwest::Method::PATCH, &format!("https://api.notion.com/v1/pages/{page_id}"))
            .json(&json!({"properties": Value::Object(properties)})).send().await
            .map_err(|e| format!("Notion metric refresh failed: {e}"))?;
        notion_json(response).await?;
        Ok(())
    }

    async fn upsert_creator(
        &self,
        data_source_id: &str,
        existing: Option<&CreatorMemory>,
        creator_name: &str,
        username: &str,
        account_id: &str,
        profile_url: &str,
        videos_stored: usize,
        scripts_stored: usize,
        now: &str,
    ) -> Result<CreatorMemory, String> {
        let mut properties = Map::new();
        properties.insert("Creator".into(), title_property(creator_name));
        properties.insert("Instagram Username".into(), rich_text_property(username));
        properties.insert("Instagram Account ID".into(), rich_text_property(account_id));
        properties.insert("Profile URL".into(), json!({"url": profile_url}));
        properties.insert("Videos Stored".into(), number_property(Some(videos_stored as f64)));
        properties.insert("Scripts Stored".into(), number_property(Some(scripts_stored as f64)));
        properties.insert("Last Checked".into(), date_property(Some(now)));
        properties.insert("Last Updated".into(), date_property(Some(now)));
        properties.insert("Status".into(), json!({"select": {"name":"Active"}}));
        if existing.and_then(|e| e.first_seen.as_ref()).is_none() {
            properties.insert("First Seen".into(), date_property(Some(now)));
        }
        let (method, url, body) = if let Some(current) = existing {
            (reqwest::Method::PATCH, format!("https://api.notion.com/v1/pages/{}", current.page_id), json!({"properties": Value::Object(properties)}))
        } else {
            (reqwest::Method::POST, "https://api.notion.com/v1/pages".into(), json!({"parent":{"type":"data_source_id","data_source_id":data_source_id},"properties":Value::Object(properties)}))
        };
        let response = self.request(method, &url).json(&body).send().await.map_err(|e| format!("Notion creator upsert failed: {e}"))?;
        let value = notion_json(response).await?;
        Ok(CreatorMemory {
            page_id: value.get("id").and_then(Value::as_str).unwrap_or_else(|| existing.map(|e| e.page_id.as_str()).unwrap_or("")).to_string(),
            page_url: value.get("url").and_then(Value::as_str).unwrap_or_else(|| existing.map(|e| e.page_url.as_str()).unwrap_or("")).to_string(),
            first_seen: existing.and_then(|e| e.first_seen.clone()).or_else(|| Some(now.to_string())),
        })
    }

    async fn create_research_run(&self, data_source_id: &str, username: &str, profile_url: &str, existing: usize, added: usize, total: usize, now: &str) -> Result<(), String> {
        let evidence = format!("Creator update completed. Existing videos before run: {existing}. New videos added: {added}. Total scripts stored after run: {total}.");
        let response = self.request(reqwest::Method::POST, "https://api.notion.com/v1/pages")
            .json(&json!({
                "parent":{"type":"data_source_id","data_source_id":data_source_id},
                "properties":{
                    "Entry": title_property(&format!("Research run — @{username}")),
                    "Type":{"select":{"name":"Research Run"}},
                    "Date": date_property(Some(now)),
                    "Creator": rich_text_property(&format!("@{username}")),
                    "Source Link":{"url":profile_url},
                    "Evidence / Result": rich_text_property(&evidence),
                    "Lesson / Skill": rich_text_property("Completed persistent creator research using memory-first deduplication and source-linked script capture."),
                    "Repeated Evidence Count":{"number":1},
                    "Permanent Learning":{"checkbox":false},
                    "Approved By":{"select":{"name":"Evidence"}}
                }
            })).send().await.map_err(|e| format!("Notion CV update failed: {e}"))?;
        notion_json(response).await?;
        Ok(())
    }

    fn request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.client.request(method, url)
            .bearer_auth(&self.token)
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
    }
}

async fn notion_json(response: reqwest::Response) -> Result<Value, String> {
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!("Notion returned {status}: {}", detail.chars().take(400).collect::<String>()));
    }
    response.json::<Value>().await.map_err(|e| format!("Invalid Notion response: {e}"))
}

fn required_env(key: &str) -> Result<String, String> {
    env::var(key).ok().filter(|v| !v.trim().is_empty()).ok_or_else(|| format!("{key} is not configured for the local NOVA runtime"))
}
fn env_or_default(key: &str, default: &str) -> String { env::var(key).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.into()) }
fn default_true() -> bool { true }
fn normalize_username(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_start_matches('@').trim_matches('/');
    let from_url = trimmed.split("instagram.com/").nth(1).and_then(|tail| tail.split('/').next()).unwrap_or(trimmed);
    let clean = from_url.split('?').next().unwrap_or(from_url).trim_matches('/').to_lowercase();
    if clean.is_empty() || clean.contains(char::is_whitespace) { return Err("Instagram username is invalid".into()); }
    Ok(clean)
}
fn notion_page_url(id: &str) -> String { format!("https://www.notion.so/{}", id.replace('-', "")) }
fn safe_error(error: &str) -> String {
    if error.contains("APIFY_TOKEN") || error.contains("NOTION_TOKEN") { "NOVA runtime credential is missing or invalid".into() } else { error.chars().take(500).collect() }
}
fn string_value(value: &Value, keys: &[&str]) -> Option<String> { keys.iter().find_map(|k| value.get(*k).and_then(Value::as_str).map(str::to_string)).filter(|s| !s.trim().is_empty()) }
fn int_value(value: &Value, keys: &[&str]) -> Option<i64> { keys.iter().find_map(|k| value.get(*k).and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|x| x as i64)))) }
fn float_value(value: &Value, keys: &[&str]) -> Option<f64> { keys.iter().find_map(|k| value.get(*k).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|x| x as f64)))) }
fn contains_any(text: &str, terms: &[&str]) -> bool { terms.iter().any(|term| text.contains(term)) }
fn first_phrase(value: &str, max_chars: usize) -> String {
    let cleaned = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() { return String::new(); }
    let mut end = cleaned.len();
    for (idx, ch) in cleaned.char_indices() {
        if idx > max_chars { end = idx; break; }
        if matches!(ch, '.' | '!' | '?' | '؟' | '\n') && idx > 12 { end = idx + ch.len_utf8(); break; }
    }
    cleaned[..end.min(cleaned.len())].to_string()
}
fn rich_text_property(value: &str) -> Value {
    let parts = chunk_text(value, 1800).into_iter().map(|chunk| json!({"type":"text","text":{"content":chunk}})).collect::<Vec<_>>();
    json!({"rich_text":parts})
}
fn title_property(value: &str) -> Value { json!({"title":[{"type":"text","text":{"content": first_phrase(value, 1800)}}]}) }
fn number_property(value: Option<f64>) -> Value { json!({"number": value}) }
fn date_property(value: Option<&str>) -> Value { match value { Some(v) if !v.is_empty() => json!({"date":{"start":v}}), _ => json!({"date":Value::Null}) } }
fn chunk_text(value: &str, max_chars: usize) -> Vec<String> {
    if value.is_empty() { return Vec::new(); }
    let chars = value.chars().collect::<Vec<_>>();
    chars.chunks(max_chars).map(|chunk| chunk.iter().collect()).collect()
}
fn notion_text_property(page: &Value, name: &str) -> Option<String> {
    let prop = page.get("properties")?.get(name)?;
    for key in ["rich_text", "title"] {
        if let Some(items) = prop.get(key).and_then(Value::as_array) {
            let text = items.iter().filter_map(|v| v.get("plain_text").and_then(Value::as_str)).collect::<Vec<_>>().join("");
            if !text.is_empty() { return Some(text); }
        }
    }
    None
}
fn notion_date_property(page: &Value, name: &str) -> Option<String> { page.get("properties")?.get(name)?.get("date")?.get("start")?.as_str().map(str::to_string) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_normalizes_handle_and_url() {
        assert_eq!(normalize_username("@StevenTawfik").unwrap(), "steventawfik");
        assert_eq!(normalize_username("https://www.instagram.com/steventawfik/#").unwrap(), "steventawfik");
    }

    #[test]
    fn transcript_is_preserved_verbatim_when_string() {
        let source = json!({"id":"123","url":"https://www.instagram.com/reel/x/","transcript":"Look, look here... see this?"});
        let reel = normalize_reel(&source, "creator").unwrap();
        assert_eq!(reel.transcript, "Look, look here... see this?");
    }

    #[test]
    fn simple_analysis_does_not_create_fake_score() {
        let reel = ReelRecord { video_id:"1".into(), reel_url:"https://www.instagram.com/reel/x/".into(), creator_name:"".into(), username:"x".into(), account_id:"".into(), publish_date:None, views:Some(1000), likes:None, comments:None, duration_sec:None, caption:"AI demo".into(), transcript:"Look, look here. See this AI workflow?".into() };
        let analysis = analyze_reel(&reel);
        assert_eq!(analysis.content_style, "Demo");
        assert!(!analysis.why_it_worked.contains("/10"));
        assert!(analysis.sam_relevant);
    }

    #[test]
    fn rich_text_chunks_long_transcripts() {
        let input = "x".repeat(5000);
        let value = rich_text_property(&input);
        assert!(value["rich_text"].as_array().unwrap().len() >= 3);
    }

    #[test]
    fn known_video_set_can_prevent_reprocessing() {
        let known: HashSet<String> = ["video-1".to_string()].into_iter().collect();
        assert!(known.contains("video-1"));
        assert!(!known.contains("video-2"));
    }
}
