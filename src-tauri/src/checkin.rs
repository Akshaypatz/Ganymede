use crate::db::Database;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

// ─── Output structs ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckinItem {
    pub id: String,
    pub title: String,
    pub item_type: String,
    pub priority: String,
    pub status: String,
    pub assignee: String,
    pub project_name: String,
    pub due_at: Option<String>,
    pub updated_at: String,
    pub age_days: i64,
    /// "overdue" | "stale" | "blocked" | "due_soon"
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckinProject {
    pub id: String,
    pub name: String,
    pub health: String,
    pub stage: String,
    pub last_activity_days: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckinReport {
    pub overdue: Vec<CheckinItem>,
    pub stale: Vec<CheckinItem>,
    pub blocked: Vec<CheckinItem>,
    pub due_soon: Vec<CheckinItem>,
    pub pending_me: Vec<CheckinItem>,
    pub at_risk_projects: Vec<CheckinProject>,
    pub total_attention: usize,
    pub generated_at: String,
}

// ─── Helpers ────────────────────────────────────────

fn days_since(iso: &str) -> i64 {
    let now = Utc::now();
    if let Ok(dt) = DateTime::parse_from_rfc3339(iso) {
        (now - dt.with_timezone(&Utc)).num_days()
    } else {
        0
    }
}

fn is_overdue(due_at: &Option<String>) -> bool {
    let today = Utc::now().date_naive();
    if let Some(ref d) = due_at {
        // Try full ISO first, then date-only
        if let Ok(dt) = DateTime::parse_from_rfc3339(d) {
            return dt.date_naive() < today;
        }
        if let Ok(nd) = NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            return nd < today;
        }
    }
    false
}

fn is_due_soon(due_at: &Option<String>) -> bool {
    let today = Utc::now().date_naive();
    let horizon = today + Duration::days(2);
    if let Some(ref d) = due_at {
        if let Ok(dt) = DateTime::parse_from_rfc3339(d) {
            let nd = dt.date_naive();
            return nd >= today && nd <= horizon;
        }
        if let Ok(nd) = NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            return nd >= today && nd <= horizon;
        }
    }
    false
}

// ─── Commands ───────────────────────────────────────

#[tauri::command]
pub fn get_checkin_report(db: State<'_, Database>) -> Result<CheckinReport, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Load snoozed items from settings
    let snoozed_json: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key='checkin_snoozes'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    let snoozed: std::collections::HashMap<String, String> =
        serde_json::from_str(&snoozed_json).unwrap_or_default();
    let now_iso = Utc::now().to_rfc3339();

    // Active (not done, not snoozed) items with project name
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.type, i.title, i.priority, i.status, i.assignee,
                    i.due_at, i.updated_at, COALESCE(p.name,'') as project_name
             FROM items i
             LEFT JOIN projects p ON i.project_id = p.id
             WHERE i.status NOT IN ('done','snoozed')
             ORDER BY CASE i.priority WHEN 'p0' THEN 0 WHEN 'p1' THEN 1 WHEN 'p2' THEN 2 ELSE 3 END,
                      i.updated_at ASC",
        )
        .map_err(|e| e.to_string())?;

    struct Row {
        id: String,
        item_type: String,
        title: String,
        priority: String,
        status: String,
        assignee: String,
        due_at: Option<String>,
        updated_at: String,
        project_name: String,
    }

    let rows: Vec<Row> = stmt
        .query_map([], |r| {
            Ok(Row {
                id: r.get(0)?,
                item_type: r.get(1)?,
                title: r.get(2)?,
                priority: r.get(3)?,
                status: r.get(4)?,
                assignee: r.get(5)?,
                due_at: r.get(6)?,
                updated_at: r.get(7)?,
                project_name: r.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut overdue: Vec<CheckinItem> = Vec::new();
    let mut stale: Vec<CheckinItem> = Vec::new();
    let mut blocked: Vec<CheckinItem> = Vec::new();
    let mut due_soon: Vec<CheckinItem> = Vec::new();
    let mut pending_me: Vec<CheckinItem> = Vec::new();

    for row in &rows {
        // Skip snoozed items
        if let Some(snooze_until) = snoozed.get(&row.id) {
            if snooze_until.as_str() > now_iso.as_str() {
                continue;
            }
        }

        let age_days = days_since(&row.updated_at);

        // Pending on Me (user's dependency items — always shown in their own section)
        if row.status == "pending_me" {
            pending_me.push(CheckinItem {
                id: row.id.clone(),
                title: row.title.clone(),
                item_type: row.item_type.clone(),
                priority: row.priority.clone(),
                status: row.status.clone(),
                assignee: row.assignee.clone(),
                project_name: row.project_name.clone(),
                due_at: row.due_at.clone(),
                updated_at: row.updated_at.clone(),
                age_days,
                reason: "pending_me".into(),
            });
            continue;
        }

        // Overdue
        if is_overdue(&row.due_at) {
            overdue.push(CheckinItem {
                id: row.id.clone(),
                title: row.title.clone(),
                item_type: row.item_type.clone(),
                priority: row.priority.clone(),
                status: row.status.clone(),
                assignee: row.assignee.clone(),
                project_name: row.project_name.clone(),
                due_at: row.due_at.clone(),
                updated_at: row.updated_at.clone(),
                age_days,
                reason: "overdue".into(),
            });
            continue; // Don't double-count in other buckets
        }

        // Blocked
        if row.status == "blocked" && age_days >= 3 {
            blocked.push(CheckinItem {
                id: row.id.clone(),
                title: row.title.clone(),
                item_type: row.item_type.clone(),
                priority: row.priority.clone(),
                status: row.status.clone(),
                assignee: row.assignee.clone(),
                project_name: row.project_name.clone(),
                due_at: row.due_at.clone(),
                updated_at: row.updated_at.clone(),
                age_days,
                reason: "blocked".into(),
            });
            continue;
        }

        // Stale thresholds by priority
        let stale_threshold: i64 = match row.priority.as_str() {
            "p0" | "p1" => 3,
            "p2" => 7,
            _ => 14,
        };
        if age_days >= stale_threshold
            && row.item_type != "idea"
        {
            stale.push(CheckinItem {
                id: row.id.clone(),
                title: row.title.clone(),
                item_type: row.item_type.clone(),
                priority: row.priority.clone(),
                status: row.status.clone(),
                assignee: row.assignee.clone(),
                project_name: row.project_name.clone(),
                due_at: row.due_at.clone(),
                updated_at: row.updated_at.clone(),
                age_days,
                reason: "stale".into(),
            });
            continue;
        }

        // Due soon (and not overdue)
        if is_due_soon(&row.due_at) {
            due_soon.push(CheckinItem {
                id: row.id.clone(),
                title: row.title.clone(),
                item_type: row.item_type.clone(),
                priority: row.priority.clone(),
                status: row.status.clone(),
                assignee: row.assignee.clone(),
                project_name: row.project_name.clone(),
                due_at: row.due_at.clone(),
                updated_at: row.updated_at.clone(),
                age_days,
                reason: "due_soon".into(),
            });
        }
    }

    // At-risk projects: health=red OR has items but none updated in 14+ days
    let mut proj_stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.health, p.stage, p.updated_at
             FROM projects p
             WHERE p.health = 'red'
                OR (
                  (SELECT COUNT(*) FROM items i WHERE i.project_id = p.id) > 0
                  AND (SELECT MAX(i.updated_at) FROM items i WHERE i.project_id = p.id) < datetime('now', '-14 days')
                )
             LIMIT 10",
        )
        .map_err(|e| e.to_string())?;

    let at_risk_projects: Vec<CheckinProject> = proj_stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .map(|(id, name, health, stage, updated_at)| {
            let last_activity_days = days_since(&updated_at);
            CheckinProject {
                id,
                name,
                health,
                stage,
                last_activity_days,
            }
        })
        .collect();

    let total_attention =
        overdue.len() + stale.len() + blocked.len() + due_soon.len() + pending_me.len() + at_risk_projects.len();

    Ok(CheckinReport {
        overdue,
        stale,
        blocked,
        due_soon,
        pending_me,
        at_risk_projects,
        total_attention,
        generated_at: now_iso,
    })
}

#[tauri::command]
pub fn snooze_checkin_item(
    db: State<'_, Database>,
    id: String,
    days: i64,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Load existing snoozes
    let existing_json: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key='checkin_snoozes'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();

    let mut snoozes: std::collections::HashMap<String, String> =
        serde_json::from_str(&existing_json).unwrap_or_default();

    // Add/update snooze
    let snooze_until = (Utc::now() + Duration::days(days)).to_rfc3339();
    snoozes.insert(id, snooze_until);

    // Prune expired snoozes
    let now_iso = Utc::now().to_rfc3339();
    snoozes.retain(|_, v| v.as_str() > now_iso.as_str());

    let new_json = serde_json::to_string(&snoozes).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('checkin_snoozes', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [&new_json],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
