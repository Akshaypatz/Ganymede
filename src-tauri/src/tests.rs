/// Integration tests for Ganymede backend logic.
/// Run with: `cargo test -- --nocapture`
///
/// These tests use an in-memory SQLite database so they don't touch user data.
#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use uuid::Uuid;
    use chrono::Utc;

    /// Creates an in-memory SQLite DB with the full schema applied.
    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory DB");
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                stage TEXT NOT NULL DEFAULT 'solutioning_pending',
                health TEXT NOT NULL DEFAULT 'green',
                progress INTEGER NOT NULL DEFAULT 0,
                due_date TEXT,
                owner_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS project_members (
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                member_id  TEXT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
                role TEXT NOT NULL DEFAULT 'member',
                PRIMARY KEY (project_id, member_id)
            );
            CREATE TABLE IF NOT EXISTS members (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                color TEXT NOT NULL DEFAULT '#f97316',
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS items (
                id TEXT PRIMARY KEY,
                project_id TEXT,
                type TEXT NOT NULL DEFAULT 'issue',
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                priority TEXT NOT NULL DEFAULT 'p2',
                status TEXT NOT NULL DEFAULT 'open',
                assignee TEXT NOT NULL DEFAULT '',
                due_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                category TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                color TEXT NOT NULL DEFAULT '#737373',
                category TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS item_tags (
                item_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (item_id, tag_id)
            );
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'todo',
                assignee_id TEXT,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS activity_log (
                id TEXT PRIMARY KEY,
                item_id TEXT,
                project_id TEXT,
                action TEXT NOT NULL,
                detail TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "
        ).expect("schema creation");
        conn
    }

    // ── Helper: insert a project ──────────────────────────────────────────

    fn insert_project(conn: &Connection, name: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO projects (id, name, description, stage, health, progress, created_at, updated_at) VALUES (?1,?2,'',?3,'green',0,?4,?5)",
            rusqlite::params![id, name, "solutioning_pending", now, now],
        ).unwrap();
        id
    }

    // ── Helper: insert a member ──────────────────────────────────────────

    fn insert_member(conn: &Connection, name: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO members (id, name, email, color, created_at) VALUES (?1,?2,'','#f97316',?3)",
            rusqlite::params![id, name, now],
        ).unwrap();
        id
    }

    // ─────────────────────────────────────────────────────────────────────
    // Project CRUD tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_project() {
        let conn = test_db();
        let id = insert_project(&conn, "Alpha");
        let name: String = conn.query_row("SELECT name FROM projects WHERE id=?1", [&id], |r| r.get(0)).unwrap();
        assert_eq!(name, "Alpha");
    }

    #[test]
    fn test_project_name_deduplication() {
        let conn = test_db();
        let id1 = insert_project(&conn, "Beta");

        // Simulate what apply_ai_action does: check before inserting
        let existing_id: Option<String> = conn.query_row(
            "SELECT id FROM projects WHERE lower(name) = lower(?1)",
            rusqlite::params!["beta"],
            |r| r.get(0),
        ).ok();
        assert!(existing_id.is_some(), "Should find existing project case-insensitively");
        assert_eq!(existing_id.unwrap(), id1);

        // Count should remain 1 (we wouldn't insert)
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_update_project_rename() {
        let conn = test_db();
        let id = insert_project(&conn, "Old Name");
        let now = Utc::now().to_rfc3339();

        // Simulate update_project action
        conn.execute(
            "UPDATE projects SET name=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params!["New Name", now, id],
        ).unwrap();

        let name: String = conn.query_row("SELECT name FROM projects WHERE id=?1", [&id], |r| r.get(0)).unwrap();
        assert_eq!(name, "New Name");
    }

    #[test]
    fn test_find_project_by_name_case_insensitive() {
        let conn = test_db();
        insert_project(&conn, "Gamma Project");

        let pid: Option<String> = conn.query_row(
            "SELECT id FROM projects WHERE lower(name)=lower(?1)",
            rusqlite::params!["gamma project"],
            |r| r.get(0),
        ).ok();
        assert!(pid.is_some(), "Should find project case-insensitively");

        let pid2: Option<String> = conn.query_row(
            "SELECT id FROM projects WHERE lower(name)=lower(?1)",
            rusqlite::params!["GAMMA PROJECT"],
            |r| r.get(0),
        ).ok();
        assert!(pid2.is_some());
    }

    // ─────────────────────────────────────────────────────────────────────
    // Member tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_member() {
        let conn = test_db();
        let id = insert_member(&conn, "Alice");
        let name: String = conn.query_row("SELECT name FROM members WHERE id=?1", [&id], |r| r.get(0)).unwrap();
        assert_eq!(name, "Alice");
    }

    #[test]
    fn test_member_deduplication() {
        let conn = test_db();
        let id1 = insert_member(&conn, "Bob");

        // Check dedup logic
        let existing: Option<String> = conn.query_row(
            "SELECT id FROM members WHERE lower(name)=lower(?1)",
            rusqlite::params!["bob"],
            |r| r.get(0),
        ).ok();
        assert_eq!(existing, Some(id1));
    }

    #[test]
    fn test_add_member_to_project() {
        let conn = test_db();
        let pid = insert_project(&conn, "Delta");
        let mid = insert_member(&conn, "Carol");

        conn.execute(
            "INSERT OR IGNORE INTO project_members (project_id, member_id, role) VALUES (?1,?2,'member')",
            rusqlite::params![pid, mid],
        ).unwrap();

        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM project_members WHERE project_id=?1 AND member_id=?2",
            rusqlite::params![pid, mid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);

        // Idempotent: insert again should not duplicate (INSERT OR IGNORE)
        conn.execute(
            "INSERT OR IGNORE INTO project_members (project_id, member_id, role) VALUES (?1,?2,'member')",
            rusqlite::params![pid, mid],
        ).unwrap();
        let count2: i32 = conn.query_row(
            "SELECT COUNT(*) FROM project_members WHERE project_id=?1 AND member_id=?2",
            rusqlite::params![pid, mid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count2, 1, "INSERT OR IGNORE should not create duplicate member link");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Item tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_item() {
        let conn = test_db();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO items (id, type, title, body, priority, status, assignee, created_at, updated_at) VALUES (?1,'issue','Test Issue','',?2,'open','',?3,?4)",
            rusqlite::params![id, "p2", now, now],
        ).unwrap();
        let title: String = conn.query_row("SELECT title FROM items WHERE id=?1", [&id], |r| r.get(0)).unwrap();
        assert_eq!(title, "Test Issue");
    }

    #[test]
    fn test_invalid_project_id_is_rejected() {
        let conn = test_db();
        // Simulate the project_id validation: only use a pid if it actually exists
        let fake_pid = "non-existent-uuid-123";
        let exists: bool = conn.query_row(
            "SELECT 1 FROM projects WHERE id=?1",
            rusqlite::params![fake_pid],
            |_| Ok(true),
        ).unwrap_or(false);
        assert!(!exists, "Fake project_id should not exist");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Task tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_task() {
        let conn = test_db();
        let pid = insert_project(&conn, "Epsilon");
        let tid = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO tasks (id, project_id, title, description, status, position, created_at, updated_at) VALUES (?1,?2,'Fix bug','',?3,1,?4,?5)",
            rusqlite::params![tid, pid, "todo", now, now],
        ).unwrap();
        let title: String = conn.query_row("SELECT title FROM tasks WHERE id=?1", [&tid], |r| r.get(0)).unwrap();
        assert_eq!(title, "Fix bug");
    }

    #[test]
    fn test_task_position_ordering() {
        let conn = test_db();
        let pid = insert_project(&conn, "Zeta");
        let now = Utc::now().to_rfc3339();
        for i in 1..=5_i32 {
            let tid = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO tasks (id, project_id, title, description, status, position, created_at, updated_at) VALUES (?1,?2,?3,'','todo',?4,?5,?6)",
                rusqlite::params![tid, pid, format!("Task {}", i), i, now, now],
            ).unwrap();
        }
        let max_pos: i32 = conn.query_row(
            "SELECT COALESCE(MAX(position),0) FROM tasks WHERE project_id=?1",
            [&pid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(max_pos, 5);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Export JSON tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_export_json_structure() {
        let conn = test_db();
        // Insert some data
        let pid = insert_project(&conn, "Export Test Project");
        let _mid = insert_member(&conn, "Export Tester");
        let now = Utc::now().to_rfc3339();
        let iid = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO items (id, type, title, body, priority, status, assignee, created_at, updated_at) VALUES (?1,'issue','Export Item','','p2','open','',?2,?3)",
            rusqlite::params![iid, now, now],
        ).unwrap();

        // Re-implement the core export query to validate structure
        let project_count: i32 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0)).unwrap();
        let member_count: i32 = conn.query_row("SELECT COUNT(*) FROM members", [], |r| r.get(0)).unwrap();
        let item_count: i32 = conn.query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0)).unwrap();

        assert_eq!(project_count, 1);
        assert_eq!(member_count, 1);
        assert_eq!(item_count, 1);

        // Verify exported project name
        let name: String = conn.query_row("SELECT name FROM projects WHERE id=?1", [&pid], |r| r.get(0)).unwrap();
        assert_eq!(name, "Export Test Project");
    }

    #[test]
    fn test_import_json_skip_duplicate() {
        let conn = test_db();
        let pid = insert_project(&conn, "Duplicate Check");

        // Try to INSERT OR IGNORE the same project again
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR IGNORE INTO projects (id, name, description, stage, health, progress, created_at, updated_at) VALUES (?1,'Duplicate Check','','solutioning_pending','green',0,?2,?3)",
            rusqlite::params![pid, now, now],
        ).unwrap();

        let count: i32 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1, "Duplicate project should be skipped by INSERT OR IGNORE");
    }

    // ─────────────────────────────────────────────────────────────────────
    // Settings tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_settings_upsert() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('ai_provider', 'openai') ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [],
        ).unwrap();
        let v: String = conn.query_row("SELECT value FROM settings WHERE key='ai_provider'", [], |r| r.get(0)).unwrap();
        assert_eq!(v, "openai");

        // Update
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('ai_provider', 'groq') ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [],
        ).unwrap();
        let v2: String = conn.query_row("SELECT value FROM settings WHERE key='ai_provider'", [], |r| r.get(0)).unwrap();
        assert_eq!(v2, "groq");
    }
}
