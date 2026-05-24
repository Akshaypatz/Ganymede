use crate::db::Database;
use crate::models::*;
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

// ─── Projects ───────────────────────────────────────

#[tauri::command]
pub fn list_projects(db: State<Database>) -> Result<Vec<Project>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT p.id, p.name, p.description, p.stage, p.health, p.progress, p.due_date, p.owner_id, p.created_at, p.updated_at, m.name FROM projects p LEFT JOIN members m ON p.owner_id = m.id ORDER BY p.created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                stage: row.get(3)?,
                health: row.get(4)?,
                progress: row.get(5)?,
                due_date: row.get(6)?,
                owner_id: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                owner_name: row.get(10)?,
                members: Vec::new(),
                tasks: Vec::new(),
            })
        })
        .map_err(|e| e.to_string())?;
    let mut projects = Vec::new();
    for row in rows {
        projects.push(row.map_err(|e| e.to_string())?);
    }
    Ok(projects)
}

#[tauri::command]
pub fn create_project(db: State<Database>, data: CreateProject) -> Result<Project, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let project = Project {
        id: id.clone(),
        name: data.name,
        description: data.description.unwrap_or_default(),
        stage: data.stage.unwrap_or_else(|| "solutioning_pending".into()),
        health: data.health.unwrap_or_else(|| "green".into()),
        progress: data.progress.unwrap_or(0),
        due_date: data.due_date,
        owner_id: data.owner_id,
        created_at: now.clone(),
        updated_at: now,
        owner_name: None,
        members: Vec::new(),
        tasks: Vec::new(),
    };
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO projects (id, name, description, stage, health, progress, due_date, owner_id, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        rusqlite::params![project.id, project.name, project.description, project.stage, project.health, project.progress, project.due_date, project.owner_id, project.created_at, project.updated_at],
    ).map_err(|e| e.to_string())?;
    // Attach members
    if let Some(ref member_ids) = data.member_ids {
        for mid in member_ids {
            conn.execute("INSERT OR IGNORE INTO project_members (project_id, member_id, role) VALUES (?1, ?2, 'member')", rusqlite::params![project.id, mid]).ok();
        }
    }
    // Log activity
    conn.execute(
        "INSERT INTO activity_log (id, project_id, action, detail, created_at) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![Uuid::new_v4().to_string(), project.id, "created", format!("Project '{}' created", project.name), project.created_at],
    ).ok();
    Ok(project)
}

#[tauri::command]
pub fn update_project(db: State<Database>, data: UpdateProject) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let mut existing = conn
        .query_row("SELECT p.id, p.name, p.description, p.stage, p.health, p.progress, p.due_date, p.owner_id, p.created_at, p.updated_at FROM projects p WHERE p.id=?1", [&data.id], |row| {
            Ok(Project { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, stage: row.get(3)?, health: row.get(4)?, progress: row.get(5)?, due_date: row.get(6)?, owner_id: row.get(7)?, created_at: row.get(8)?, updated_at: row.get(9)?, owner_name: None, members: Vec::new(), tasks: Vec::new() })
        }).map_err(|e| e.to_string())?;
    if let Some(v) = data.name { existing.name = v; }
    if let Some(v) = data.description { existing.description = v; }
    if let Some(v) = data.stage { existing.stage = v; }
    if let Some(v) = data.health { existing.health = v; }
    if let Some(v) = data.progress { existing.progress = v; }
    if data.due_date.is_some() { existing.due_date = data.due_date; }
    if data.owner_id.is_some() { existing.owner_id = data.owner_id; }
    existing.updated_at = now;
    conn.execute(
        "UPDATE projects SET name=?1, description=?2, stage=?3, health=?4, progress=?5, due_date=?6, owner_id=?7, updated_at=?8 WHERE id=?9",
        rusqlite::params![existing.name, existing.description, existing.stage, existing.health, existing.progress, existing.due_date, existing.owner_id, existing.updated_at, existing.id],
    ).map_err(|e| e.to_string())?;
    // Update members if provided
    if let Some(ref member_ids) = data.member_ids {
        conn.execute("DELETE FROM project_members WHERE project_id=?1", [&existing.id]).ok();
        for mid in member_ids {
            conn.execute("INSERT OR IGNORE INTO project_members (project_id, member_id, role) VALUES (?1, ?2, 'member')", rusqlite::params![existing.id, mid]).ok();
        }
    }
    Ok(existing)
}

#[tauri::command]
pub fn delete_project(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id=?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Items ──────────────────────────────────────────

#[tauri::command]
pub fn list_items(db: State<Database>, item_type: Option<String>, status: Option<String>, project_id: Option<String>, priority: Option<String>) -> Result<Vec<Item>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut sql = String::from("SELECT i.id, i.project_id, i.type, i.title, i.body, i.priority, i.status, i.assignee, i.due_at, i.created_at, i.updated_at, p.name, i.category FROM items i LEFT JOIN projects p ON i.project_id = p.id WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1;
    if let Some(ref t) = item_type {
        sql.push_str(&format!(" AND i.type=?{}", param_idx));
        params.push(Box::new(t.clone()));
        param_idx += 1;
    }
    if let Some(ref s) = status {
        sql.push_str(&format!(" AND i.status=?{}", param_idx));
        params.push(Box::new(s.clone()));
        param_idx += 1;
    }
    if let Some(ref p) = project_id {
        sql.push_str(&format!(" AND i.project_id=?{}", param_idx));
        params.push(Box::new(p.clone()));
        param_idx += 1;
    }
    if let Some(ref pr) = priority {
        sql.push_str(&format!(" AND i.priority=?{}", param_idx));
        params.push(Box::new(pr.clone()));
        let _ = param_idx;
    }
    sql.push_str(" ORDER BY CASE i.priority WHEN 'p0' THEN 0 WHEN 'p1' THEN 1 WHEN 'p2' THEN 2 ELSE 3 END, i.created_at DESC");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Item {
            id: row.get(0)?,
            project_id: row.get(1)?,
            r#type: row.get(2)?,
            title: row.get(3)?,
            body: row.get(4)?,
            priority: row.get(5)?,
            status: row.get(6)?,
            assignee: row.get(7)?,
            due_at: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            tags: Vec::new(),
            project_name: row.get(11)?,
            category: row.get::<_, Option<String>>(12)?.unwrap_or_default(),
        })
    }).map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        let mut item = row.map_err(|e| e.to_string())?;
        // Fetch tags
        let mut tag_stmt = conn.prepare("SELECT t.id, t.name, t.color, t.category FROM tags t INNER JOIN item_tags it ON t.id = it.tag_id WHERE it.item_id = ?1").map_err(|e| e.to_string())?;
        let tag_rows = tag_stmt.query_map([&item.id], |r| {
            Ok(Tag { id: r.get(0)?, name: r.get(1)?, color: r.get(2)?, category: r.get(3)? })
        }).map_err(|e| e.to_string())?;
        for t in tag_rows { item.tags.push(t.map_err(|e| e.to_string())?); }
        items.push(item);
    }
    Ok(items)
}

#[tauri::command]
pub fn create_item(db: State<Database>, data: CreateItem) -> Result<Item, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let item = Item {
        id: id.clone(),
        project_id: data.project_id,
        r#type: data.r#type.unwrap_or_else(|| "issue".into()),
        title: data.title,
        body: data.body.unwrap_or_default(),
        priority: data.priority.unwrap_or_else(|| "p2".into()),
        status: data.status.unwrap_or_else(|| "open".into()),
        assignee: data.assignee.unwrap_or_default(),
        due_at: data.due_at,
        created_at: now.clone(),
        updated_at: now,
        tags: Vec::new(),
        project_name: None,
        category: data.category.unwrap_or_default(),
    };
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO items (id, project_id, type, title, body, priority, status, assignee, due_at, created_at, updated_at, category) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        rusqlite::params![item.id, item.project_id, item.r#type, item.title, item.body, item.priority, item.status, item.assignee, item.due_at, item.created_at, item.updated_at, item.category],
    ).map_err(|e| e.to_string())?;
    // Attach tags
    if let Some(ref tag_ids) = data.tag_ids {
        for tid in tag_ids {
            conn.execute("INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1, ?2)", rusqlite::params![item.id, tid]).ok();
        }
    }
    // Log activity
    conn.execute(
        "INSERT INTO activity_log (id, item_id, project_id, action, detail, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![Uuid::new_v4().to_string(), item.id, item.project_id, "created", format!("{} '{}' created", item.r#type, item.title), item.created_at],
    ).ok();
    Ok(item)
}

#[tauri::command]
pub fn update_item(db: State<Database>, data: UpdateItem) -> Result<Item, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let mut existing = conn.query_row(
        "SELECT id, project_id, type, title, body, priority, status, assignee, due_at, created_at, updated_at, category FROM items WHERE id=?1", [&data.id],
        |row| Ok(Item { id: row.get(0)?, project_id: row.get(1)?, r#type: row.get(2)?, title: row.get(3)?, body: row.get(4)?, priority: row.get(5)?, status: row.get(6)?, assignee: row.get(7)?, due_at: row.get(8)?, created_at: row.get(9)?, updated_at: row.get(10)?, tags: Vec::new(), project_name: None, category: row.get::<_, Option<String>>(11)?.unwrap_or_default() })
    ).map_err(|e| e.to_string())?;
    if data.project_id.is_some() { existing.project_id = data.project_id; }
    if let Some(v) = data.r#type { existing.r#type = v; }
    if let Some(v) = data.title { existing.title = v; }
    if let Some(v) = data.body { existing.body = v; }
    if let Some(v) = data.priority { existing.priority = v; }
    if let Some(v) = data.status {
        let old_status = existing.status.clone();
        existing.status = v.clone();
        // Log status change
        conn.execute(
            "INSERT INTO activity_log (id, item_id, project_id, action, detail, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            rusqlite::params![Uuid::new_v4().to_string(), existing.id, existing.project_id, "status_changed", format!("'{}' status: {} → {}", existing.title, old_status, v), &now],
        ).ok();
    }
    if let Some(v) = data.assignee { existing.assignee = v; }
    if data.due_at.is_some() { existing.due_at = data.due_at; }
    if let Some(v) = data.category { existing.category = v; }
    existing.updated_at = now;
    conn.execute(
        "UPDATE items SET project_id=?1, type=?2, title=?3, body=?4, priority=?5, status=?6, assignee=?7, due_at=?8, updated_at=?9, category=?10 WHERE id=?11",
        rusqlite::params![existing.project_id, existing.r#type, existing.title, existing.body, existing.priority, existing.status, existing.assignee, existing.due_at, existing.updated_at, existing.category, existing.id],
    ).map_err(|e| e.to_string())?;
    // Update tags if provided
    if let Some(ref tag_ids) = data.tag_ids {
        conn.execute("DELETE FROM item_tags WHERE item_id=?1", [&existing.id]).ok();
        for tid in tag_ids {
            conn.execute("INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1, ?2)", rusqlite::params![existing.id, tid]).ok();
        }
    }
    Ok(existing)
}

#[tauri::command]
pub fn delete_item(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM items WHERE id=?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Tags ───────────────────────────────────────────

#[tauri::command]
pub fn list_tags(db: State<Database>) -> Result<Vec<Tag>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, color, category FROM tags ORDER BY name").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| Ok(Tag { id: row.get(0)?, name: row.get(1)?, color: row.get(2)?, category: row.get(3)? })).map_err(|e| e.to_string())?;
    let mut tags = Vec::new();
    for row in rows { tags.push(row.map_err(|e| e.to_string())?); }
    Ok(tags)
}

#[tauri::command]
pub fn create_tag(db: State<Database>, data: CreateTag) -> Result<Tag, String> {
    let id = Uuid::new_v4().to_string();
    let tag = Tag { id, name: data.name, color: data.color.unwrap_or_else(|| "#737373".into()), category: data.category.unwrap_or_default() };
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO tags (id, name, color, category) VALUES (?1,?2,?3,?4)", rusqlite::params![tag.id, tag.name, tag.color, tag.category]).map_err(|e| e.to_string())?;
    Ok(tag)
}

#[tauri::command]
pub fn delete_tag(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tags WHERE id=?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Boards ─────────────────────────────────────────

#[tauri::command]
pub fn list_boards(db: State<Database>) -> Result<Vec<Board>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM boards ORDER BY created_at").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| Ok(Board { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, columns: Vec::new() })).map_err(|e| e.to_string())?;
    let mut boards = Vec::new();
    for row in rows { boards.push(row.map_err(|e| e.to_string())?); }
    Ok(boards)
}

#[tauri::command]
pub fn get_board(db: State<Database>, id: String) -> Result<Board, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut board = conn.query_row("SELECT id, name, created_at FROM boards WHERE id=?1", [&id], |row| Ok(Board { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, columns: Vec::new() })).map_err(|e| e.to_string())?;
    // Fetch columns
    let mut col_stmt = conn.prepare("SELECT id, board_id, name, color, position FROM board_columns WHERE board_id=?1 ORDER BY position").map_err(|e| e.to_string())?;
    let col_rows = col_stmt.query_map([&id], |row| Ok(BoardColumn { id: row.get(0)?, board_id: row.get(1)?, name: row.get(2)?, color: row.get(3)?, position: row.get(4)?, cards: Vec::new() })).map_err(|e| e.to_string())?;
    for col_row in col_rows {
        let mut col = col_row.map_err(|e| e.to_string())?;
        // Fetch cards for this column
        let mut card_stmt = conn.prepare("SELECT id, column_id, item_id, title, body, position, created_at FROM board_cards WHERE column_id=?1 ORDER BY position").map_err(|e| e.to_string())?;
        let card_rows = card_stmt.query_map([&col.id], |row| Ok(BoardCard { id: row.get(0)?, column_id: row.get(1)?, item_id: row.get(2)?, title: row.get(3)?, body: row.get(4)?, position: row.get(5)?, created_at: row.get(6)?, tags: Vec::new() })).map_err(|e| e.to_string())?;
        for card_row in card_rows {
            let mut card = card_row.map_err(|e| e.to_string())?;
            // Fetch card tags
            let mut tag_stmt = conn.prepare("SELECT t.id, t.name, t.color, t.category FROM tags t INNER JOIN card_tags ct ON t.id = ct.tag_id WHERE ct.card_id = ?1").map_err(|e| e.to_string())?;
            let tag_rows = tag_stmt.query_map([&card.id], |r| Ok(Tag { id: r.get(0)?, name: r.get(1)?, color: r.get(2)?, category: r.get(3)? })).map_err(|e| e.to_string())?;
            for t in tag_rows { card.tags.push(t.map_err(|e| e.to_string())?); }
            col.cards.push(card);
        }
        board.columns.push(col);
    }
    Ok(board)
}

#[tauri::command]
pub fn create_board(db: State<Database>, data: CreateBoard) -> Result<Board, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO boards (id, name, created_at) VALUES (?1,?2,?3)", rusqlite::params![id, data.name, now]).map_err(|e| e.to_string())?;
    // Create columns if provided
    let mut columns = Vec::new();
    if let Some(cols) = data.columns {
        for (i, col) in cols.iter().enumerate() {
            let col_id = Uuid::new_v4().to_string();
            let color = col.color.clone().unwrap_or_default();
            conn.execute("INSERT INTO board_columns (id, board_id, name, color, position) VALUES (?1,?2,?3,?4,?5)", rusqlite::params![col_id, id, col.name, color, i as i32]).map_err(|e| e.to_string())?;
            columns.push(BoardColumn { id: col_id, board_id: id.clone(), name: col.name.clone(), color, position: i as i32, cards: Vec::new() });
        }
    }
    Ok(Board { id, name: data.name, created_at: now, columns })
}

#[tauri::command]
pub fn add_board_column(db: State<Database>, board_id: String, name: String, color: Option<String>) -> Result<BoardColumn, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let max_pos: i32 = conn.query_row("SELECT COALESCE(MAX(position),0) FROM board_columns WHERE board_id=?1", [&board_id], |r| r.get(0)).unwrap_or(0);
    let id = Uuid::new_v4().to_string();
    let c = color.unwrap_or_default();
    conn.execute("INSERT INTO board_columns (id, board_id, name, color, position) VALUES (?1,?2,?3,?4,?5)", rusqlite::params![id, board_id, name, c, max_pos + 1]).map_err(|e| e.to_string())?;
    Ok(BoardColumn { id, board_id, name, color: c, position: max_pos + 1, cards: Vec::new() })
}

#[tauri::command]
pub fn add_board_card(db: State<Database>, data: CreateBoardCard) -> Result<BoardCard, String> {
    let now = Utc::now().to_rfc3339();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let max_pos: i32 = conn.query_row("SELECT COALESCE(MAX(position),0) FROM board_cards WHERE column_id=?1", [&data.column_id], |r| r.get(0)).unwrap_or(0);
    let id = Uuid::new_v4().to_string();
    let body = data.body.unwrap_or_default();
    conn.execute("INSERT INTO board_cards (id, column_id, item_id, title, body, position, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", rusqlite::params![id, data.column_id, data.item_id, data.title, body, max_pos + 1, now]).map_err(|e| e.to_string())?;
    // Attach tags
    if let Some(ref tag_ids) = data.tag_ids {
        for tid in tag_ids {
            conn.execute("INSERT OR IGNORE INTO card_tags (card_id, tag_id) VALUES (?1, ?2)", rusqlite::params![id, tid]).ok();
        }
    }
    Ok(BoardCard { id, column_id: data.column_id, item_id: data.item_id, title: data.title, body, position: max_pos + 1, created_at: now, tags: Vec::new() })
}

#[tauri::command]
pub fn move_card(db: State<Database>, data: MoveCard) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    // Shift cards in target column to make room
    conn.execute(
        "UPDATE board_cards SET position = position + 1 WHERE column_id=?1 AND position >= ?2",
        rusqlite::params![data.target_column_id, data.target_position],
    ).map_err(|e| e.to_string())?;
    // Move the card
    conn.execute(
        "UPDATE board_cards SET column_id=?1, position=?2 WHERE id=?3",
        rusqlite::params![data.target_column_id, data.target_position, data.card_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_board(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM boards WHERE id=?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_board_column(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM board_columns WHERE id=?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_board_card(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM board_cards WHERE id=?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Activity ───────────────────────────────────────

#[tauri::command]
pub fn list_activity(db: State<Database>, limit: Option<i32>) -> Result<Vec<ActivityEntry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let lim = limit.unwrap_or(20);
    let mut stmt = conn.prepare("SELECT id, item_id, project_id, action, detail, created_at FROM activity_log ORDER BY created_at DESC LIMIT ?1").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([lim], |row| Ok(ActivityEntry { id: row.get(0)?, item_id: row.get(1)?, project_id: row.get(2)?, action: row.get(3)?, detail: row.get(4)?, created_at: row.get(5)? })).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    for row in rows { entries.push(row.map_err(|e| e.to_string())?); }
    Ok(entries)
}

// ─── Dashboard Stats ────────────────────────────────

#[tauri::command]
pub fn get_dashboard_stats(db: State<Database>) -> Result<DashboardStats, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let count = |sql: &str| -> i32 {
        conn.query_row(sql, [], |r| r.get(0)).unwrap_or(0)
    };
    let today = Utc::now().format("%Y-%m-%d").to_string();
    Ok(DashboardStats {
        p0_count: count("SELECT COUNT(*) FROM items WHERE priority='p0' AND status NOT IN ('done')"),
        p1_count: count("SELECT COUNT(*) FROM items WHERE priority='p1' AND status NOT IN ('done')"),
        p2_count: count("SELECT COUNT(*) FROM items WHERE priority='p2' AND status NOT IN ('done')"),
        p3_count: count("SELECT COUNT(*) FROM items WHERE priority='p3' AND status NOT IN ('done')"),
        waiting_count: count("SELECT COUNT(*) FROM items WHERE status='waiting'"),
        waiting_overdue: {
            let sql = format!("SELECT COUNT(*) FROM items WHERE status='waiting' AND due_at IS NOT NULL AND due_at < '{}'", today);
            conn.query_row(&sql, [], |r| r.get(0)).unwrap_or(0)
        },
        followups_today: {
            let sql = format!("SELECT COUNT(*) FROM items WHERE type='followup' AND status NOT IN ('done') AND (due_at LIKE '{}%' OR due_at IS NULL)", today);
            conn.query_row(&sql, [], |r| r.get(0)).unwrap_or(0)
        },
        projects_at_risk: count("SELECT COUNT(*) FROM projects WHERE health='red'"),
        projects_on_track: count("SELECT COUNT(*) FROM projects WHERE health='green'"),
        total_projects: count("SELECT COUNT(*) FROM projects"),
    })
}

// ─── Settings ───────────────────────────────────────

#[tauri::command]
pub fn get_setting(db: State<Database>, key: String) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let result = conn.query_row("SELECT value FROM settings WHERE key=?1", [&key], |r| r.get(0));
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_setting(db: State<Database>, key: String, value: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", rusqlite::params![key, value]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reset_app_data(db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = OFF;
        DELETE FROM card_tags;
        DELETE FROM item_tags;
        DELETE FROM board_cards;
        DELETE FROM board_columns;
        DELETE FROM boards;
        DELETE FROM reminders;
        DELETE FROM tasks;
        DELETE FROM project_members;
        DELETE FROM activity_log;
        DELETE FROM items;
        DELETE FROM tags;
        DELETE FROM projects;
        DELETE FROM members;
        DELETE FROM ai_messages;
        DELETE FROM ai_conversations;
        DELETE FROM settings;
        PRAGMA foreign_keys = ON;
        "
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Get Project Detail ─────────────────────────────

#[tauri::command]
pub fn get_project(db: State<Database>, id: String) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut project = conn.query_row(
        "SELECT p.id, p.name, p.description, p.stage, p.health, p.progress, p.due_date, p.owner_id, p.created_at, p.updated_at, m.name FROM projects p LEFT JOIN members m ON p.owner_id = m.id WHERE p.id=?1",
        [&id],
        |row| Ok(Project {
            id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, stage: row.get(3)?,
            health: row.get(4)?, progress: row.get(5)?, due_date: row.get(6)?, owner_id: row.get(7)?,
            created_at: row.get(8)?, updated_at: row.get(9)?, owner_name: row.get(10)?,
            members: Vec::new(), tasks: Vec::new(),
        })
    ).map_err(|e| e.to_string())?;
    // Fetch members
    let mut mem_stmt = conn.prepare(
        "SELECT m.id, m.name, m.email, m.color, m.created_at FROM members m INNER JOIN project_members pm ON m.id = pm.member_id WHERE pm.project_id=?1"
    ).map_err(|e| e.to_string())?;
    let mem_rows = mem_stmt.query_map([&id], |r| Ok(Member {
        id: r.get(0)?, name: r.get(1)?, email: r.get(2)?, color: r.get(3)?, created_at: r.get(4)?,
    })).map_err(|e| e.to_string())?;
    for m in mem_rows { project.members.push(m.map_err(|e| e.to_string())?); }
    // Fetch tasks
    let mut task_stmt = conn.prepare(
        "SELECT t.id, t.project_id, t.title, t.description, t.status, t.assignee_id, t.position, t.created_at, t.updated_at, m.name FROM tasks t LEFT JOIN members m ON t.assignee_id = m.id WHERE t.project_id=?1 ORDER BY t.position, t.created_at"
    ).map_err(|e| e.to_string())?;
    let task_rows = task_stmt.query_map([&id], |r| Ok(Task {
        id: r.get(0)?, project_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?,
        status: r.get(4)?, assignee_id: r.get(5)?, position: r.get(6)?,
        created_at: r.get(7)?, updated_at: r.get(8)?, assignee_name: r.get(9)?,
        project_name: None,
    })).map_err(|e| e.to_string())?;
    for t in task_rows { project.tasks.push(t.map_err(|e| e.to_string())?); }
    Ok(project)
}

// ─── Members ────────────────────────────────────────

#[tauri::command]
pub fn list_members(db: State<Database>) -> Result<Vec<Member>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, email, color, created_at FROM members ORDER BY name").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| Ok(Member {
        id: row.get(0)?, name: row.get(1)?, email: row.get(2)?, color: row.get(3)?, created_at: row.get(4)?,
    })).map_err(|e| e.to_string())?;
    let mut members = Vec::new();
    for row in rows { members.push(row.map_err(|e| e.to_string())?); }
    Ok(members)
}

#[tauri::command]
pub fn create_member(db: State<Database>, data: CreateMember) -> Result<Member, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let member = Member {
        id, name: data.name, email: data.email.unwrap_or_default(),
        color: data.color.unwrap_or_else(|| "#f97316".into()), created_at: now,
    };
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO members (id, name, email, color, created_at) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![member.id, member.name, member.email, member.color, member.created_at],
    ).map_err(|e| e.to_string())?;
    Ok(member)
}

#[tauri::command]
pub fn update_member(db: State<Database>, data: UpdateMember) -> Result<Member, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut existing = conn.query_row("SELECT id, name, email, color, created_at FROM members WHERE id=?1", [&data.id], |r| {
        Ok(Member { id: r.get(0)?, name: r.get(1)?, email: r.get(2)?, color: r.get(3)?, created_at: r.get(4)? })
    }).map_err(|e| e.to_string())?;
    if let Some(v) = data.name { existing.name = v; }
    if let Some(v) = data.email { existing.email = v; }
    if let Some(v) = data.color { existing.color = v; }
    conn.execute("UPDATE members SET name=?1, email=?2, color=?3 WHERE id=?4",
        rusqlite::params![existing.name, existing.email, existing.color, existing.id],
    ).map_err(|e| e.to_string())?;
    Ok(existing)
}

#[tauri::command]
pub fn delete_member(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM members WHERE id=?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Tasks ──────────────────────────────────────────

#[tauri::command]
pub fn list_tasks(db: State<Database>, project_id: String) -> Result<Vec<Task>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.project_id, t.title, t.description, t.status, t.assignee_id, t.position, t.created_at, t.updated_at, m.name FROM tasks t LEFT JOIN members m ON t.assignee_id = m.id WHERE t.project_id=?1 ORDER BY t.position, t.created_at"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([&project_id], |r| Ok(Task {
        id: r.get(0)?, project_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?,
        status: r.get(4)?, assignee_id: r.get(5)?, position: r.get(6)?,
        created_at: r.get(7)?, updated_at: r.get(8)?, assignee_name: r.get(9)?,
        project_name: None,
    })).map_err(|e| e.to_string())?;
    let mut tasks = Vec::new();
    for r in rows { tasks.push(r.map_err(|e| e.to_string())?); }
    Ok(tasks)
}

#[tauri::command]
pub fn create_task(db: State<Database>, data: CreateTask) -> Result<Task, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let max_pos: i32 = conn.query_row("SELECT COALESCE(MAX(position),0) FROM tasks WHERE project_id=?1", [&data.project_id], |r| r.get(0)).unwrap_or(0);
    let task = Task {
        id, project_id: data.project_id, title: data.title,
        description: data.description.unwrap_or_default(),
        status: data.status.unwrap_or_else(|| "todo".into()),
        assignee_id: data.assignee_id, position: max_pos + 1,
        created_at: now.clone(), updated_at: now, assignee_name: None, project_name: None,
    };
    conn.execute("INSERT INTO tasks (id, project_id, title, description, status, assignee_id, position, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params![task.id, task.project_id, task.title, task.description, task.status, task.assignee_id, task.position, task.created_at, task.updated_at],
    ).map_err(|e| e.to_string())?;
    // Update project progress
    update_project_progress(&conn, &task.project_id);
    Ok(task)
}

#[tauri::command]
pub fn update_task(db: State<Database>, data: UpdateTask) -> Result<Task, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let mut existing = conn.query_row(
        "SELECT id, project_id, title, description, status, assignee_id, position, created_at, updated_at FROM tasks WHERE id=?1", [&data.id],
        |r| Ok(Task { id: r.get(0)?, project_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?, status: r.get(4)?, assignee_id: r.get(5)?, position: r.get(6)?, created_at: r.get(7)?, updated_at: r.get(8)?, assignee_name: None, project_name: None })
    ).map_err(|e| e.to_string())?;
    if let Some(v) = data.title { existing.title = v; }
    if let Some(v) = data.description { existing.description = v; }
    if let Some(v) = data.status { existing.status = v; }
    if data.assignee_id.is_some() { existing.assignee_id = data.assignee_id; }
    if let Some(v) = data.position { existing.position = v; }
    existing.updated_at = now;
    conn.execute("UPDATE tasks SET title=?1, description=?2, status=?3, assignee_id=?4, position=?5, updated_at=?6 WHERE id=?7",
        rusqlite::params![existing.title, existing.description, existing.status, existing.assignee_id, existing.position, existing.updated_at, existing.id],
    ).map_err(|e| e.to_string())?;
    // Update project progress
    update_project_progress(&conn, &existing.project_id);
    Ok(existing)
}

#[tauri::command]
pub fn list_pending_me_tasks(db: State<Database>) -> Result<Vec<Task>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.project_id, t.title, t.description, t.status, t.assignee_id, t.position, t.created_at, t.updated_at, m.name, p.name FROM tasks t LEFT JOIN members m ON t.assignee_id = m.id JOIN projects p ON t.project_id = p.id WHERE t.status = 'pending_me' ORDER BY t.updated_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Task {
        id: r.get(0)?, project_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?,
        status: r.get(4)?, assignee_id: r.get(5)?, position: r.get(6)?,
        created_at: r.get(7)?, updated_at: r.get(8)?, assignee_name: r.get(9)?,
        project_name: r.get(10)?,
    })).map_err(|e| e.to_string())?;
    let mut tasks = Vec::new();
    for r in rows { tasks.push(r.map_err(|e| e.to_string())?); }
    Ok(tasks)
}

#[tauri::command]
pub fn list_blocked_tasks(db: State<Database>) -> Result<Vec<Task>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.project_id, t.title, t.description, t.status, t.assignee_id, t.position, t.created_at, t.updated_at, m.name, p.name FROM tasks t LEFT JOIN members m ON t.assignee_id = m.id JOIN projects p ON t.project_id = p.id WHERE t.status = 'blocked' ORDER BY t.created_at"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Task {
        id: r.get(0)?, project_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?,
        status: r.get(4)?, assignee_id: r.get(5)?, position: r.get(6)?,
        created_at: r.get(7)?, updated_at: r.get(8)?, assignee_name: r.get(9)?,
        project_name: r.get(10)?,
    })).map_err(|e| e.to_string())?;
    let mut tasks = Vec::new();
    for r in rows { tasks.push(r.map_err(|e| e.to_string())?); }
    Ok(tasks)
}

#[tauri::command]
pub fn delete_task(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let project_id: String = conn.query_row("SELECT project_id FROM tasks WHERE id=?1", [&id], |r| r.get(0)).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tasks WHERE id=?1", [&id]).map_err(|e| e.to_string())?;
    update_project_progress(&conn, &project_id);
    Ok(())
}

fn update_project_progress(conn: &rusqlite::Connection, project_id: &str) {
    let total: i32 = conn.query_row("SELECT COUNT(*) FROM tasks WHERE project_id=?1", [project_id], |r| r.get(0)).unwrap_or(0);
    if total == 0 { return; }
    let done: i32 = conn.query_row("SELECT COUNT(*) FROM tasks WHERE project_id=?1 AND status='done'", [project_id], |r| r.get(0)).unwrap_or(0);
    let progress = (done * 100) / total;
    conn.execute("UPDATE projects SET progress=?1 WHERE id=?2", rusqlite::params![progress, project_id]).ok();
}

// ─── Export / Import ────────────────────────────────

/// Returns a JSON string containing all user data. Used for backup and re-import.
#[tauri::command]
pub fn export_json(db: State<Database>) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Projects with their member IDs
    let mut proj_stmt = conn.prepare(
        "SELECT p.id, p.name, p.description, p.stage, p.health, p.progress, p.due_date, p.owner_id, p.created_at, p.updated_at FROM projects p ORDER BY p.created_at"
    ).map_err(|e| e.to_string())?;
    let projects_raw: Vec<serde_json::Value> = proj_stmt.query_map([], |r| {
        Ok((
            r.get::<_,String>(0)?,  // id
            r.get::<_,String>(1)?,  // name
            r.get::<_,String>(2)?,  // description
            r.get::<_,String>(3)?,  // stage
            r.get::<_,String>(4)?,  // health
            r.get::<_,i32>(5)?,     // progress
            r.get::<_,Option<String>>(6)?,  // due_date
            r.get::<_,Option<String>>(7)?,  // owner_id
            r.get::<_,String>(8)?,  // created_at
            r.get::<_,String>(9)?,  // updated_at
        ))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .map(|(id, name, desc, stage, health, progress, due_date, owner_id, created_at, updated_at)| {
        let member_ids: Vec<String> = conn.prepare(
            "SELECT member_id FROM project_members WHERE project_id=?1"
        ).ok().and_then(|mut s| {
            s.query_map([&id], |r| r.get::<_,String>(0)).ok().map(|rows| {
                rows.filter_map(|r| r.ok()).collect()
            })
        }).unwrap_or_default();

        serde_json::json!({
            "id": id, "name": name, "description": desc,
            "stage": stage, "health": health, "progress": progress,
            "due_date": due_date, "owner_id": owner_id,
            "created_at": created_at, "updated_at": updated_at,
            "member_ids": member_ids,
        })
    })
    .collect();

    // Items with tag IDs
    let mut item_stmt = conn.prepare(
        "SELECT id, project_id, type, title, body, priority, status, assignee, due_at, created_at, updated_at, category FROM items ORDER BY created_at"
    ).map_err(|e| e.to_string())?;
    let items_raw: Vec<serde_json::Value> = item_stmt.query_map([], |r| {
        Ok((
            r.get::<_,String>(0)?,
            r.get::<_,Option<String>>(1)?,
            r.get::<_,String>(2)?,
            r.get::<_,String>(3)?,
            r.get::<_,String>(4)?,
            r.get::<_,String>(5)?,
            r.get::<_,String>(6)?,
            r.get::<_,String>(7)?,
            r.get::<_,Option<String>>(8)?,
            r.get::<_,String>(9)?,
            r.get::<_,String>(10)?,
            r.get::<_,Option<String>>(11)?,
        ))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .map(|(id, project_id, itype, title, body, priority, status, assignee, due_at, created_at, updated_at, category)| {
        let tag_ids: Vec<String> = conn.prepare(
            "SELECT tag_id FROM item_tags WHERE item_id=?1"
        ).ok().and_then(|mut s| {
            s.query_map([&id], |r| r.get::<_,String>(0)).ok().map(|rows| {
                rows.filter_map(|r| r.ok()).collect()
            })
        }).unwrap_or_default();

        serde_json::json!({
            "id": id, "project_id": project_id, "type": itype,
            "title": title, "body": body, "priority": priority, "status": status,
            "assignee": assignee, "due_at": due_at,
            "created_at": created_at, "updated_at": updated_at,
            "category": category, "tag_ids": tag_ids,
        })
    })
    .collect();

    // Members
    let mut mem_stmt = conn.prepare(
        "SELECT id, name, email, color, created_at FROM members ORDER BY name"
    ).map_err(|e| e.to_string())?;
    let members_raw: Vec<serde_json::Value> = mem_stmt.query_map([], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_,String>(0)?,
            "name": r.get::<_,String>(1)?,
            "email": r.get::<_,String>(2)?,
            "color": r.get::<_,String>(3)?,
            "created_at": r.get::<_,String>(4)?,
        }))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Tasks
    let mut task_stmt = conn.prepare(
        "SELECT id, project_id, title, description, status, assignee_id, position, created_at, updated_at FROM tasks ORDER BY created_at"
    ).map_err(|e| e.to_string())?;
    let tasks_raw: Vec<serde_json::Value> = task_stmt.query_map([], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_,String>(0)?,
            "project_id": r.get::<_,String>(1)?,
            "title": r.get::<_,String>(2)?,
            "description": r.get::<_,String>(3)?,
            "status": r.get::<_,String>(4)?,
            "assignee_id": r.get::<_,Option<String>>(5)?,
            "position": r.get::<_,i32>(6)?,
            "created_at": r.get::<_,String>(7)?,
            "updated_at": r.get::<_,String>(8)?,
        }))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Tags
    let mut tag_stmt = conn.prepare(
        "SELECT id, name, color, category FROM tags ORDER BY name"
    ).map_err(|e| e.to_string())?;
    let tags_raw: Vec<serde_json::Value> = tag_stmt.query_map([], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_,String>(0)?,
            "name": r.get::<_,String>(1)?,
            "color": r.get::<_,String>(2)?,
            "category": r.get::<_,String>(3)?,
        }))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let export = serde_json::json!({
        "version": 1,
        "exported_at": Utc::now().to_rfc3339(),
        "projects": projects_raw,
        "items": items_raw,
        "members": members_raw,
        "tasks": tasks_raw,
        "tags": tags_raw,
    });

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

/// Imports a JSON backup produced by `export_json`. Skips records that already exist (by id).
#[tauri::command]
pub fn import_json(db: State<Database>, data: String) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let version = parsed["version"].as_u64().unwrap_or(0);
    if version != 1 {
        return Err(format!("Unsupported export version: {}. Expected 1.", version));
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    let mut imported = std::collections::HashMap::<&str, usize>::new();
    imported.insert("members", 0);
    imported.insert("tags", 0);
    imported.insert("projects", 0);
    imported.insert("items", 0);
    imported.insert("tasks", 0);

    // Import members first (referenced by projects and items)
    if let Some(members) = parsed["members"].as_array() {
        for m in members {
            let id = m["id"].as_str().unwrap_or("");
            let name = m["name"].as_str().unwrap_or("");
            let email = m["email"].as_str().unwrap_or("");
            let color = m["color"].as_str().unwrap_or("#f97316");
            let created_at = m["created_at"].as_str().unwrap_or(&now);
            if id.is_empty() || name.is_empty() { continue; }
            let affected = conn.execute(
                "INSERT OR IGNORE INTO members (id, name, email, color, created_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![id, name, email, color, created_at],
            ).unwrap_or(0);
            *imported.get_mut("members").unwrap() += affected;
        }
    }

    // Import tags
    if let Some(tags) = parsed["tags"].as_array() {
        for t in tags {
            let id = t["id"].as_str().unwrap_or("");
            let name = t["name"].as_str().unwrap_or("");
            let color = t["color"].as_str().unwrap_or("#737373");
            let category = t["category"].as_str().unwrap_or("");
            if id.is_empty() || name.is_empty() { continue; }
            let affected = conn.execute(
                "INSERT OR IGNORE INTO tags (id, name, color, category) VALUES (?1,?2,?3,?4)",
                rusqlite::params![id, name, color, category],
            ).unwrap_or(0);
            *imported.get_mut("tags").unwrap() += affected;
        }
    }

    // Import projects
    if let Some(projects) = parsed["projects"].as_array() {
        for p in projects {
            let id = p["id"].as_str().unwrap_or("");
            let name = p["name"].as_str().unwrap_or("");
            let desc = p["description"].as_str().unwrap_or("");
            let stage = p["stage"].as_str().unwrap_or("solutioning_pending");
            let health = p["health"].as_str().unwrap_or("green");
            let progress = p["progress"].as_i64().unwrap_or(0) as i32;
            let due_date: Option<&str> = p["due_date"].as_str();
            let owner_id: Option<&str> = p["owner_id"].as_str();
            let created_at = p["created_at"].as_str().unwrap_or(&now);
            let updated_at = p["updated_at"].as_str().unwrap_or(&now);
            if id.is_empty() || name.is_empty() { continue; }
            let affected = conn.execute(
                "INSERT OR IGNORE INTO projects (id, name, description, stage, health, progress, due_date, owner_id, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                rusqlite::params![id, name, desc, stage, health, progress, due_date, owner_id, created_at, updated_at],
            ).unwrap_or(0);
            *imported.get_mut("projects").unwrap() += affected;
            // Restore project members
            if let Some(member_ids) = p["member_ids"].as_array() {
                for mid in member_ids {
                    if let Some(mid_str) = mid.as_str() {
                        conn.execute(
                            "INSERT OR IGNORE INTO project_members (project_id, member_id, role) VALUES (?1,?2,'member')",
                            rusqlite::params![id, mid_str],
                        ).ok();
                    }
                }
            }
        }
    }

    // Import items
    if let Some(items) = parsed["items"].as_array() {
        for item in items {
            let id = item["id"].as_str().unwrap_or("");
            let project_id: Option<&str> = item["project_id"].as_str();
            let itype = item["type"].as_str().unwrap_or("issue");
            let title = item["title"].as_str().unwrap_or("");
            let body = item["body"].as_str().unwrap_or("");
            let priority = item["priority"].as_str().unwrap_or("p2");
            let status = item["status"].as_str().unwrap_or("open");
            let assignee = item["assignee"].as_str().unwrap_or("");
            let due_at: Option<&str> = item["due_at"].as_str();
            let created_at = item["created_at"].as_str().unwrap_or(&now);
            let updated_at = item["updated_at"].as_str().unwrap_or(&now);
            let category = item["category"].as_str().unwrap_or("");
            if id.is_empty() || title.is_empty() { continue; }
            let affected = conn.execute(
                "INSERT OR IGNORE INTO items (id, project_id, type, title, body, priority, status, assignee, due_at, created_at, updated_at, category) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                rusqlite::params![id, project_id, itype, title, body, priority, status, assignee, due_at, created_at, updated_at, category],
            ).unwrap_or(0);
            *imported.get_mut("items").unwrap() += affected;
            // Restore item tags
            if let Some(tag_ids) = item["tag_ids"].as_array() {
                for tid in tag_ids {
                    if let Some(tid_str) = tid.as_str() {
                        conn.execute(
                            "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1,?2)",
                            rusqlite::params![id, tid_str],
                        ).ok();
                    }
                }
            }
        }
    }

    // Import tasks
    if let Some(tasks) = parsed["tasks"].as_array() {
        for t in tasks {
            let id = t["id"].as_str().unwrap_or("");
            let project_id = t["project_id"].as_str().unwrap_or("");
            let title = t["title"].as_str().unwrap_or("");
            let desc = t["description"].as_str().unwrap_or("");
            let status = t["status"].as_str().unwrap_or("todo");
            let assignee_id: Option<&str> = t["assignee_id"].as_str();
            let position = t["position"].as_i64().unwrap_or(0) as i32;
            let created_at = t["created_at"].as_str().unwrap_or(&now);
            let updated_at = t["updated_at"].as_str().unwrap_or(&now);
            if id.is_empty() || project_id.is_empty() || title.is_empty() { continue; }
            let affected = conn.execute(
                "INSERT OR IGNORE INTO tasks (id, project_id, title, description, status, assignee_id, position, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                rusqlite::params![id, project_id, title, desc, status, assignee_id, position, created_at, updated_at],
            ).unwrap_or(0);
            *imported.get_mut("tasks").unwrap() += affected;
        }
    }

    Ok(format!(
        "Import complete: {} projects, {} items, {} tasks, {} members, {} tags",
        imported["projects"], imported["items"], imported["tasks"],
        imported["members"], imported["tags"]
    ))
}
