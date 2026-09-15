# NOVA Runtime V1

NOVA is the Instagram Content Researcher reporting to Saly.

## V1 loop

`CHECK NOTION MEMORY → FETCH INSTAGRAM REELS → DEDUP BY VIDEO ID → PROCESS NEW ONLY → SAVE SCRIPT + METRICS + ANALYSIS → UPDATE CREATOR MEMORY → ADD CV RESEARCH RUN → HANDOFF TO SALY → NOTIFY SAM`

## Local secrets

Copy the repository-root `.env.example` to `.env` on the machine running SAM Growth Machine.

Required secrets:

- `APIFY_TOKEN`
- `NOTION_TOKEN`

Never commit `.env`. The repository `.gitignore` already excludes it.

The desktop backend loads `.env` locally at startup. Secrets are not sent to the React frontend and are not stored in GitHub.

## Notion memory

NOVA V1 uses:

- `NOVA — Creator Memory`
- `SAM — Instagram Scripts Intelligence`
- `NOVA — CV, Portfolio & Learning Log`

The Notion integration represented by `NOTION_TOKEN` must have access to the NOVA page/databases.

## Apify V1 provider

Provider: `apify/instagram-reel-scraper`

NOVA requests only the configured number of Reels and enables transcript extraction. Downloaded-video storage and shares-count add-ons are disabled in V1.

Known Instagram Video/Media IDs are checked in Notion before new records are created. Known Reels are not re-analyzed or re-stored; observable metrics may be refreshed.

## Desktop command

Tauri command:

`nova_research_creator`

Request:

```json
{
  "instagramUsername": "@steventawfik",
  "maxReels": 20,
  "refreshMetrics": true
}
```

The Team view exposes this command from NOVA's employee card through **Research Instagram creator**.

## First smoke test

Use `@steventawfik` with a small limit first (5–10 Reels) to verify:

1. NOVA changes to `WORKING` during the run.
2. Apify returns Reel data and transcript output.
3. New Video IDs create rows in `SAM — Instagram Scripts Intelligence`.
4. `Full Script` preserves the spoken transcript.
5. A second run skips the same Video IDs rather than duplicating them.
6. Metrics can refresh on known rows.
7. Creator Memory is created/updated.
8. One `Research Run` entry is added to NOVA's CV log.
9. System events record the handoff to Saly and notification to Sam.

## V1 boundaries

- No theoretical content score.
- No automatic permanent learning from one Reel.
- No final content strategy decisions by NOVA.
- No publishing.
- No extra scraper agents.
- Notion remains V1 operational memory.
