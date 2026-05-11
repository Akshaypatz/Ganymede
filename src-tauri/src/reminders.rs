use crate::db::Database;
use crate::models::{CreateReminder, Reminder};
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn create_reminder(db: State<Database>, data: CreateReminder) -> Result<Reminder, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let tune = data.tune.unwrap_or_else(|| "bell".into());
    let label = data.label.unwrap_or_default();

    // Fetch item title for convenience
    let item_title: String = conn
        .query_row("SELECT title FROM items WHERE id=?1", [&data.item_id], |r| r.get(0))
        .unwrap_or_default();

    conn.execute(
        "INSERT INTO reminders (id, item_id, due_at, recurrence, completed, tune, label) VALUES (?1,?2,?3,?4,0,?5,?6)",
        rusqlite::params![id, data.item_id, data.due_at, data.recurrence, tune, label],
    )
    .map_err(|e| e.to_string())?;

    Ok(Reminder {
        id,
        item_id: data.item_id,
        item_title,
        due_at: data.due_at,
        recurrence: data.recurrence,
        completed: false,
        snoozed_to: None,
        tune,
        label,
    })
}

#[tauri::command]
pub fn list_reminders(db: State<Database>, item_id: Option<String>) -> Result<Vec<Reminder>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let sql = if item_id.is_some() {
        "SELECT r.id, r.item_id, r.due_at, r.recurrence, r.completed, r.snoozed_to, r.tune, r.label, i.title \
         FROM reminders r LEFT JOIN items i ON r.item_id = i.id \
         WHERE r.completed=0 AND r.item_id=?1 ORDER BY r.due_at ASC"
    } else {
        "SELECT r.id, r.item_id, r.due_at, r.recurrence, r.completed, r.snoozed_to, r.tune, r.label, i.title \
         FROM reminders r LEFT JOIN items i ON r.item_id = i.id \
         WHERE r.completed=0 ORDER BY r.due_at ASC"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = if let Some(ref id) = item_id {
        stmt.query_map([id], map_reminder).map_err(|e| e.to_string())?
    } else {
        stmt.query_map([], map_reminder).map_err(|e| e.to_string())?
    };

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

fn map_reminder(row: &rusqlite::Row<'_>) -> rusqlite::Result<Reminder> {
    Ok(Reminder {
        id: row.get(0)?,
        item_id: row.get(1)?,
        due_at: row.get(2)?,
        recurrence: row.get(3)?,
        completed: row.get::<_, i32>(4)? != 0,
        snoozed_to: row.get(5)?,
        tune: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "bell".into()),
        label: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
        item_title: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
    })
}

#[tauri::command]
pub fn get_due_reminders(db: State<Database>) -> Result<Vec<Reminder>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let mut stmt = conn.prepare(
        "SELECT r.id, r.item_id, r.due_at, r.recurrence, r.completed, r.snoozed_to, r.tune, r.label, i.title \
         FROM reminders r LEFT JOIN items i ON r.item_id = i.id \
         WHERE r.completed=0 AND r.due_at <= ?1 \
           AND (r.snoozed_to IS NULL OR r.snoozed_to <= ?1) \
         ORDER BY r.due_at ASC",
    )
    .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([&now], map_reminder).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

#[tauri::command]
pub fn complete_reminder(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE reminders SET completed=1 WHERE id=?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn snooze_reminder(db: State<Database>, id: String, minutes: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let snoozed_to = (Utc::now() + chrono::Duration::minutes(minutes)).to_rfc3339();
    conn.execute(
        "UPDATE reminders SET snoozed_to=?1 WHERE id=?2",
        rusqlite::params![snoozed_to, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_reminder(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM reminders WHERE id=?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
