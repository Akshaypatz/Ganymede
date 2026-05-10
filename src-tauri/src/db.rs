use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self, rusqlite::Error> {
        fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("ganymede.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                stage       TEXT NOT NULL DEFAULT 'solutioning_pending',
                health      TEXT NOT NULL DEFAULT 'green',
                progress    INTEGER NOT NULL DEFAULT 0,
                due_date    TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS items (
                id          TEXT PRIMARY KEY,
                project_id  TEXT,
                type        TEXT NOT NULL DEFAULT 'issue',
                title       TEXT NOT NULL,
                body        TEXT NOT NULL DEFAULT '',
                priority    TEXT NOT NULL DEFAULT 'p2',
                status      TEXT NOT NULL DEFAULT 'open',
                assignee    TEXT NOT NULL DEFAULT '',
                due_at      TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS tags (
                id       TEXT PRIMARY KEY,
                name     TEXT UNIQUE NOT NULL,
                color    TEXT NOT NULL DEFAULT '#737373',
                category TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS item_tags (
                item_id  TEXT NOT NULL,
                tag_id   TEXT NOT NULL,
                PRIMARY KEY (item_id, tag_id),
                FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS boards (
                id         TEXT PRIMARY KEY,
                name       TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS board_columns (
                id        TEXT PRIMARY KEY,
                board_id  TEXT NOT NULL,
                name      TEXT NOT NULL,
                color     TEXT NOT NULL DEFAULT '',
                position  INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS board_cards (
                id         TEXT PRIMARY KEY,
                column_id  TEXT NOT NULL,
                item_id    TEXT,
                title      TEXT NOT NULL,
                body       TEXT NOT NULL DEFAULT '',
                position   INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (column_id) REFERENCES board_columns(id) ON DELETE CASCADE,
                FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS card_tags (
                card_id  TEXT NOT NULL,
                tag_id   TEXT NOT NULL,
                PRIMARY KEY (card_id, tag_id),
                FOREIGN KEY (card_id) REFERENCES board_cards(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS reminders (
                id         TEXT PRIMARY KEY,
                item_id    TEXT NOT NULL,
                due_at     TEXT NOT NULL,
                recurrence TEXT,
                completed  INTEGER NOT NULL DEFAULT 0,
                snoozed_to TEXT,
                FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS activity_log (
                id         TEXT PRIMARY KEY,
                item_id    TEXT,
                project_id TEXT,
                action     TEXT NOT NULL,
                detail     TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS members (
                id         TEXT PRIMARY KEY,
                name       TEXT NOT NULL,
                email      TEXT NOT NULL DEFAULT '',
                color      TEXT NOT NULL DEFAULT '#f97316',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project_members (
                project_id TEXT NOT NULL,
                member_id  TEXT NOT NULL,
                role       TEXT NOT NULL DEFAULT 'member',
                PRIMARY KEY (project_id, member_id),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (member_id) REFERENCES members(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id          TEXT PRIMARY KEY,
                project_id  TEXT NOT NULL,
                title       TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status      TEXT NOT NULL DEFAULT 'todo',
                assignee_id TEXT,
                position    INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (assignee_id) REFERENCES members(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS ai_conversations (
                id         TEXT PRIMARY KEY,
                title      TEXT NOT NULL DEFAULT 'New conversation',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ai_messages (
                id              TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role            TEXT NOT NULL,
                content         TEXT NOT NULL,
                actions_json    TEXT,
                followups_json  TEXT,
                created_at      TEXT NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_items_project_status ON items(project_id, status, priority);
            CREATE INDEX IF NOT EXISTS idx_items_type_status ON items(type, status);
            CREATE INDEX IF NOT EXISTS idx_items_due ON items(due_at);
            CREATE INDEX IF NOT EXISTS idx_reminders_due ON reminders(due_at, completed);
            CREATE INDEX IF NOT EXISTS idx_board_cards_col ON board_cards(column_id, position);
            CREATE INDEX IF NOT EXISTS idx_activity_created ON activity_log(created_at);
            CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id, status);
            CREATE INDEX IF NOT EXISTS idx_ai_messages_conv ON ai_messages(conversation_id, created_at);
            ",
        )?;
        // Safe column additions for existing databases
        conn.execute("ALTER TABLE projects ADD COLUMN owner_id TEXT", []).ok();
        conn.execute("ALTER TABLE projects ADD COLUMN description TEXT NOT NULL DEFAULT ''", []).ok();
        Ok(())
    }
}
