use serde::{Deserialize, Serialize};

// ─── Projects ───────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub stage: String,
    pub health: String,
    pub progress: i32,
    pub due_date: Option<String>,
    pub owner_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub owner_name: Option<String>,
    #[serde(default)]
    pub members: Vec<Member>,
    #[serde(default)]
    pub tasks: Vec<Task>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub description: Option<String>,
    pub stage: Option<String>,
    pub health: Option<String>,
    pub progress: Option<i32>,
    pub due_date: Option<String>,
    pub owner_id: Option<String>,
    pub member_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub stage: Option<String>,
    pub health: Option<String>,
    pub progress: Option<i32>,
    pub due_date: Option<String>,
    pub owner_id: Option<String>,
    pub member_ids: Option<Vec<String>>,
}

// ─── Items ──────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: String,
    pub project_id: Option<String>,
    pub r#type: String,
    pub title: String,
    pub body: String,
    pub priority: String,
    pub status: String,
    pub assignee: String,
    pub due_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub project_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub project_id: Option<String>,
    pub r#type: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub due_at: Option<String>,
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItem {
    pub id: String,
    pub project_id: Option<String>,
    pub r#type: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub due_at: Option<String>,
    pub tag_ids: Option<Vec<String>>,
}

// ─── Tags ───────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTag {
    pub name: String,
    pub color: Option<String>,
    pub category: Option<String>,
}

// ─── Boards ─────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub created_at: String,
    #[serde(default)]
    pub columns: Vec<BoardColumn>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardColumn {
    pub id: String,
    pub board_id: String,
    pub name: String,
    pub color: String,
    pub position: i32,
    #[serde(default)]
    pub cards: Vec<BoardCard>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardCard {
    pub id: String,
    pub column_id: String,
    pub item_id: Option<String>,
    pub title: String,
    pub body: String,
    pub position: i32,
    pub created_at: String,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBoard {
    pub name: String,
    pub columns: Option<Vec<CreateBoardColumn>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBoardColumn {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBoardCard {
    pub column_id: String,
    pub title: String,
    pub body: Option<String>,
    pub item_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct MoveCard {
    pub card_id: String,
    pub target_column_id: String,
    pub target_position: i32,
}

// ─── Activity ───────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityEntry {
    pub id: String,
    pub item_id: Option<String>,
    pub project_id: Option<String>,
    pub action: String,
    pub detail: String,
    pub created_at: String,
}

// ─── Dashboard ──────────────────────────────────────
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub p0_count: i32,
    pub p1_count: i32,
    pub p2_count: i32,
    pub p3_count: i32,
    pub waiting_count: i32,
    pub waiting_overdue: i32,
    pub followups_today: i32,
    pub projects_at_risk: i32,
    pub projects_on_track: i32,
    pub total_projects: i32,
}

// ─── Members ────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub email: String,
    pub color: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMember {
    pub name: String,
    pub email: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMember {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub color: Option<String>,
}

// ─── Tasks ──────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub assignee_id: Option<String>,
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub assignee_name: Option<String>,
    #[serde(default)]
    pub project_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTask {
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub assignee_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTask {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub assignee_id: Option<String>,
    pub position: Option<i32>,
}

// ─── AI ─────────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct AiChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiAction {
    pub id: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub label: String,
    pub data: serde_json::Value,
    #[serde(default = "default_proposed")]
    pub status: String,
}

fn default_proposed() -> String {
    "proposed".into()
}

#[derive(Debug, Serialize)]
pub struct AiChatResponse {
    pub conversation_id: String,
    pub message: String,
    pub actions: Vec<AiAction>,
    pub followups: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub actions: Vec<AiAction>,
    pub followups: Vec<String>,
    pub created_at: String,
}
