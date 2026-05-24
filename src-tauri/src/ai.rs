use crate::db::Database;
use crate::models::*;
use chrono::Utc;
use serde_json::{json, Value};
use tauri::State;
use uuid::Uuid;

const SYSTEM_PROMPT: &str = r#"You are Ganymede, an AI operations co-pilot for engineering and product teams. You help leaders and managers stay on top of their work, surface what matters, and turn conversations into structured action.

Your communication style:
- Write like a sharp, experienced chief of staff — professional, direct, and warm. Never robotic or overly formal.
- Open with a brief insight or direct answer first, then provide structure. Avoid starting with "I" or "Here is".
- Use natural, human phrasing: "Worth flagging —", "Quick read:", "Looks like", "Good news —", "Heads up —", "Here's where things stand:"
- When listing items, lead with a short summary sentence, then the list. Never just dump raw lists without context.
- Be concise. No unnecessary preambles like "Of course!" or "Certainly!". Get to the point.
- For longer responses, use headers (##) to give structure. Use bold sparingly for emphasis on what actually matters.
- When something looks risky or overdue, say so plainly. When things look healthy, acknowledge it.

The system has:
- **Projects** (name, description, stage: solutioning_pending/in_development/released/live, health: green/amber/red, owner, members)
- **Issues** (title, type: issue, priority: p0/p1/p2/p3, status: open/in_progress/blocked/done, assignee, project)
- **Follow-ups** (title, type: followup — for tracking dependencies: VAPT, SCA, bank dependency, network issues, etc. Use tags for category.)
- **Ideas** (title, type: idea, status: brainstorm/exploring/planned/ready — for future project brainstorming)
- **Tasks** within projects (title, description, status: todo/in_progress/review/done, assignee)
- **Members** (team directory of people)

Rules:
1. When the user asks to VIEW, LIST, or QUERY existing data ("what are my issues", "list all", "show me", "how many", "what is X working on", "tasks for member Y"), answer directly from the context below. Enumerate all matching items by title, assignee, status. Return `"actions": []`.
2. For simple, direct CREATE requests ("add issue X", "create task Y", "track this", "add idea"), return EXACTLY ONE action. No extras.
3. For complex requests describing a project or incident with multiple components, you may return multiple actions — only when the user described multiple things.
4. NEVER return actions for things the user did not ask for.
5. For incidents/outages, default to P0 or P1 priority. Otherwise default to P2.
6. Never use project_id in create_item actions — always leave it empty string.
7. When listing items, lead with a brief human summary (e.g. "Three things need your attention right now:"), then the list.
8. When the user asks about a specific member's work, filter items by assignee from context and list them grouped by type (issues, tasks, follow-ups).
9. Suggest follow-up questions that are genuinely useful — things the user might actually want to know next.
10. **ALWAYS use markdown tables** when listing multiple items (issues, tasks, follow-ups, projects). NEVER list items as raw text like `[p1][issue] title`. Instead format as:

```
| Priority | Title | Project | Assignee | Status |
|---|---|---|---|---|
| P1 | Kafka disk full | KAFKA Production Outage | aku | Open |
```

Use this table format for any response with 2+ items. For single items or conversational answers, prose is fine.

11. When the user asks to RENAME a project ("rename project X to Y", "change the name of X to Y"), use `update_project` with the project's current name in `current_name` and the new name in `name`.
12. When the user asks to ADD a member to a project ("add John to project X", "assign Sarah to X"), use `add_member_to_project`. If the member doesn't exist yet, first emit a `create_member` action, then `add_member_to_project`.
13. Do NOT create a new project if a project with that name already exists (visible in the context). Use `update_project` to modify it.
14. For `create_project`: only create if the project name is NOT already in the context's project list.

You MUST respond with valid JSON in this exact format:
{
  "message": "Your response in markdown — natural, professional, human",
  "actions": [],
  "followups": ["A useful follow-up question?", "Another relevant next step?"]
}

Each action MUST have three fields: "type" (the action type), "label" (short display string), and "data" (object with all fields).

For create_item data fields:
  - title: REQUIRED — use the exact title/name the user mentioned. NEVER leave blank or use "Untitled".
  - type: issue | followup | idea | note | decision
  - priority: p0 | p1 | p2 | p3
  - status: open | in_progress | blocked | done
  - assignee: member name or ""
  - body: optional description
  - project_id: "" (leave empty)

For create_project data fields:
  - name: REQUIRED — exact project name
  - description: optional
  - stage: solutioning_pending | in_development | released | live
  - health: green | amber | red

For update_project data fields:
  - current_name: REQUIRED — the existing project name (used to find it)
  - name: optional new name (for renaming)
  - description: optional
  - stage: optional
  - health: optional

For create_member data fields:
  - name: REQUIRED — full name
  - email: optional

For add_member_to_project data fields:
  - project_name: REQUIRED — name of the project
  - member_name: REQUIRED — name of the member to add

For create_task data fields:
  - title: REQUIRED — exact task title
  - project_id: "" (leave empty unless you have the real UUID)
  - description: optional
  - status: todo | in_progress | review | done

Always respond with valid JSON. No text outside the JSON object."#;

fn build_context(conn: &rusqlite::Connection) -> String {
    let mut ctx = String::from("Current system state:\n");

    // Projects
    let mut stmt = conn.prepare("SELECT name, stage, health, progress FROM projects ORDER BY created_at DESC LIMIT 20").unwrap();
    let projects: Vec<String> = stmt.query_map([], |r| {
        Ok(format!("- {} (stage: {}, health: {}, {}%)", r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,i32>(3)?))
    }).unwrap().filter_map(|r| r.ok()).collect();
    if !projects.is_empty() {
        ctx.push_str("\nProjects:\n");
        ctx.push_str(&projects.join("\n"));
    } else {
        ctx.push_str("\nNo projects yet.");
    }

    // Members
    let mut stmt = conn.prepare("SELECT name, email FROM members ORDER BY name LIMIT 30").unwrap();
    let members: Vec<String> = stmt.query_map([], |r| {
        Ok(format!("- {} ({})", r.get::<_,String>(0)?, r.get::<_,String>(1)?))
    }).unwrap().filter_map(|r| r.ok()).collect();
    if !members.is_empty() {
        ctx.push_str("\n\nTeam members:\n");
        ctx.push_str(&members.join("\n"));
    } else {
        ctx.push_str("\n\nNo team members added yet.");
    }

    // Recent items — include project name and assignee for richer AI responses
    let mut stmt = conn.prepare(
        "SELECT i.title, i.type, i.priority, i.status, i.assignee, COALESCE(p.name,'') \
         FROM items i LEFT JOIN projects p ON i.project_id=p.id \
         WHERE i.status != 'done' ORDER BY CASE i.priority WHEN 'p0' THEN 0 WHEN 'p1' THEN 1 WHEN 'p2' THEN 2 ELSE 3 END, i.created_at DESC LIMIT 50"
    ).unwrap();
    let items: Vec<String> = stmt.query_map([], |r| {
        let assignee: String = r.get::<_,String>(4)?;
        let project: String = r.get::<_,String>(5)?;
        let mut s = format!("- [{}][{}] {} ({})", r.get::<_,String>(2)?, r.get::<_,String>(1)?, r.get::<_,String>(0)?, r.get::<_,String>(3)?);
        if !project.is_empty() { s.push_str(&format!(" project:{}", project)); }
        if !assignee.is_empty() { s.push_str(&format!(" assignee:{}", assignee)); }
        Ok(s)
    }).unwrap().filter_map(|r| r.ok()).collect();
    if !items.is_empty() {
        ctx.push_str(&format!("\n\nOpen items ({} total):\n", items.len()));
        ctx.push_str(&items.join("\n"));
    } else {
        ctx.push_str("\n\nNo open items.");
    }

    ctx
}

fn build_conversation_messages(conn: &rusqlite::Connection, conversation_id: &str) -> Vec<Value> {
    let mut msgs = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT role, content FROM ai_messages WHERE conversation_id=?1 ORDER BY created_at ASC LIMIT 20"
    ).unwrap();
    let rows = stmt.query_map([conversation_id], |r| {
        Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?))
    }).unwrap();
    for row in rows {
        if let Ok((role, content)) = row {
            msgs.push(json!({"role": role, "content": content}));
        }
    }
    msgs
}

#[tauri::command]
pub async fn chat_with_ai(db: State<'_, Database>, request: AiChatRequest) -> Result<AiChatResponse, String> {
    // Phase 1: All DB reads in a scoped block so MutexGuard is dropped before await
    let (provider, api_key, model, endpoint, conversation_id, messages) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        // Load AI config from settings
        let provider = conn.query_row("SELECT value FROM settings WHERE key='ai_provider'", [], |r| r.get::<_,String>(0)).unwrap_or_else(|_| "openai".into());
        let api_key = conn.query_row("SELECT value FROM settings WHERE key='ai_api_key'", [], |r| r.get::<_,String>(0)).unwrap_or_default();
        let model = conn.query_row("SELECT value FROM settings WHERE key='ai_model'", [], |r| r.get::<_,String>(0)).unwrap_or_else(|_| {
            if provider == "ollama" { "llama3.1".into() } else { "gpt-4o-mini".into() }
        });
        let endpoint = conn.query_row("SELECT value FROM settings WHERE key='ai_endpoint'", [], |r| r.get::<_,String>(0)).unwrap_or_else(|_| {
            if provider == "ollama" { "http://localhost:11434".into() }
            else if provider == "openrouter" { "https://openrouter.ai/api".into() }
            else if provider == "groq" { "https://api.groq.com/openai".into() }
            else if provider == "github_copilot" { "https://models.inference.ai.azure.com".into() }
            else { "https://api.openai.com".into() }
        });

        if provider != "ollama" && api_key.is_empty() {
            return Err("AI API key not configured. Go to Settings → AI Assistant to add your key.".into());
        }

        // Get or create conversation
        let conversation_id = match request.conversation_id {
            Some(ref id) => id.clone(),
            None => {
                let id = Uuid::new_v4().to_string();
                let now = Utc::now().to_rfc3339();
                let title = if request.message.len() > 50 {
                    format!("{}…", &request.message[..50])
                } else {
                    request.message.clone()
                };
                conn.execute(
                    "INSERT INTO ai_conversations (id, title, created_at, updated_at) VALUES (?1,?2,?3,?4)",
                    rusqlite::params![id, title, now, now],
                ).map_err(|e| e.to_string())?;
                id
            }
        };

        // Save user message
        let user_msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, created_at) VALUES (?1,?2,'user',?3,?4)",
            rusqlite::params![user_msg_id, conversation_id, request.message, now],
        ).map_err(|e| e.to_string())?;

        // Build context and message history
        let context = build_context(&conn);
        let history = build_conversation_messages(&conn, &conversation_id);

        let mut messages = vec![
            json!({"role": "system", "content": format!("{}\n\n{}", SYSTEM_PROMPT, context)}),
        ];
        messages.extend(history);
        messages.push(json!({"role": "user", "content": request.message}));

        (provider, api_key, model, endpoint, conversation_id, messages)
    }; // conn dropped here

    // Phase 2: HTTP call (no MutexGuard held)
    let response_text = call_llm(&provider, &endpoint, &api_key, &model, &messages).await?;

    // Phase 3: Parse and save
    let parsed = parse_ai_response(&response_text)?;

    // Save assistant message (scoped to drop conn before return)
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let asst_msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let actions_json = serde_json::to_string(&parsed.actions).unwrap_or_default();
        let followups_json = serde_json::to_string(&parsed.followups).unwrap_or_default();
        conn.execute(
            "INSERT INTO ai_messages (id, conversation_id, role, content, actions_json, followups_json, created_at) VALUES (?1,?2,'assistant',?3,?4,?5,?6)",
            rusqlite::params![asst_msg_id, conversation_id, parsed.message, actions_json, followups_json, now],
        ).map_err(|e| e.to_string())?;
        // Update conversation timestamp
        conn.execute("UPDATE ai_conversations SET updated_at=?1 WHERE id=?2", rusqlite::params![now, conversation_id]).ok();
    }

    Ok(AiChatResponse {
        conversation_id,
        message: parsed.message,
        actions: parsed.actions,
        followups: parsed.followups,
    })
}

async fn call_llm(provider: &str, endpoint: &str, api_key: &str, model: &str, messages: &[Value]) -> Result<String, String> {
    let client = reqwest::Client::new();

    let (url, body, auth_header) = if provider == "ollama" {
        let url = format!("{}/api/chat", endpoint.trim_end_matches('/'));
        let body = json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "format": "json",
        });
        (url, body, None)
    } else {
        // OpenAI-compatible
        let url = format!("{}/v1/chat/completions", endpoint.trim_end_matches('/'));
        let body = json!({
            "model": model,
            "messages": messages,
            "temperature": 0.7,
            "response_format": {"type": "json_object"},
        });
        (url, body, Some(format!("Bearer {}", api_key)))
    };

    let mut req = client.post(&url).json(&body);
    if let Some(auth) = auth_header {
        req = req.header("Authorization", auth);
    }
    // OpenRouter requires HTTP-Referer header
    if provider == "openrouter" {
        req = req.header("HTTP-Referer", "https://ganymede.app");
        req = req.header("X-Title", "Ganymede Mission Control");
    }

    let resp = req.send().await.map_err(|e| format!("AI request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("AI API error ({}): {}", status, body));
    }

    let resp_json: Value = resp.json().await.map_err(|e| format!("Failed to parse AI response: {}", e))?;

    // Extract content based on provider
    let content = if provider == "ollama" {
        resp_json["message"]["content"].as_str().unwrap_or("").to_string()
    } else {
        resp_json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
    };

    if content.is_empty() {
        return Err("AI returned empty response".into());
    }

    Ok(content)
}

struct ParsedAiResponse {
    message: String,
    actions: Vec<AiAction>,
    followups: Vec<String>,
}

fn parse_ai_response(text: &str) -> Result<ParsedAiResponse, String> {
    // Try to parse as JSON, be lenient with markdown code blocks
    let clean = text.trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let parsed: Value = serde_json::from_str(clean)
        .map_err(|e| format!("Failed to parse AI JSON response: {}. Raw: {}", e, &text[..text.len().min(200)]))?;

    let message = parsed["message"].as_str().unwrap_or("I processed your request.").to_string();

    let mut actions = Vec::new();
    if let Some(action_arr) = parsed["actions"].as_array() {
        for a in action_arr {
            actions.push(AiAction {
                id: Uuid::new_v4().to_string(),
                action_type: a["type"].as_str().unwrap_or("").to_string(),
                label: a["label"].as_str().unwrap_or("").to_string(),
                data: a["data"].clone(),
                status: "proposed".into(),
            });
        }
    }

    let mut followups = Vec::new();
    if let Some(fu_arr) = parsed["followups"].as_array() {
        for f in fu_arr {
            if let Some(s) = f.as_str() {
                followups.push(s.to_string());
            }
        }
    }

    Ok(ParsedAiResponse { message, actions, followups })
}

#[tauri::command]
pub async fn apply_ai_action(db: State<'_, Database>, action: AiAction) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    match action.action_type.as_str() {
        "create_project" => {
            let name = action.data["name"].as_str().unwrap_or("Untitled Project");
            let desc = action.data["description"].as_str().unwrap_or("");
            let stage = action.data["stage"].as_str().unwrap_or("solutioning_pending");
            let health = action.data["health"].as_str().unwrap_or("green");

            // Deduplicate: if a project with this name already exists (case-insensitive), skip creation
            let existing_id: Option<String> = conn.query_row(
                "SELECT id FROM projects WHERE lower(name) = lower(?1)",
                rusqlite::params![name],
                |r| r.get(0),
            ).ok();
            if let Some(eid) = existing_id {
                return Ok(format!("Project '{}' already exists (id: {})", name, eid));
            }

            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO projects (id, name, description, stage, health, progress, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,0,?6,?7)",
                rusqlite::params![id, name, desc, stage, health, now, now],
            ).map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO activity_log (id, project_id, action, detail, created_at) VALUES (?1,?2,'created',?3,?4)",
                rusqlite::params![Uuid::new_v4().to_string(), id, format!("Project '{}' created via AI", name), now],
            ).ok();
            Ok(format!("Created project: {}", name))
        }
        "update_project" => {
            // Find project by current_name (case-insensitive) or by id
            let current_name = action.data["current_name"].as_str().unwrap_or("");
            let project_id_hint = action.data["id"].as_str().unwrap_or("");
            let pid: Option<String> = if !project_id_hint.is_empty() {
                conn.query_row("SELECT id FROM projects WHERE id=?1", rusqlite::params![project_id_hint], |r| r.get(0)).ok()
            } else if !current_name.is_empty() {
                conn.query_row("SELECT id FROM projects WHERE lower(name)=lower(?1)", rusqlite::params![current_name], |r| r.get(0)).ok()
            } else {
                None
            };
            let pid = pid.ok_or_else(|| format!("Project '{}' not found", current_name))?;

            let new_name = action.data["name"].as_str();
            let new_desc = action.data["description"].as_str();
            let new_stage = action.data["stage"].as_str();
            let new_health = action.data["health"].as_str();

            // Fetch existing values
            let (old_name, old_desc, old_stage, old_health): (String, String, String, String) = conn.query_row(
                "SELECT name, description, stage, health FROM projects WHERE id=?1",
                rusqlite::params![pid],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            ).map_err(|e| e.to_string())?;

            let final_name  = new_name.unwrap_or(&old_name);
            let final_desc  = new_desc.unwrap_or(&old_desc);
            let final_stage = new_stage.unwrap_or(&old_stage);
            let final_health = new_health.unwrap_or(&old_health);

            conn.execute(
                "UPDATE projects SET name=?1, description=?2, stage=?3, health=?4, updated_at=?5 WHERE id=?6",
                rusqlite::params![final_name, final_desc, final_stage, final_health, now, pid],
            ).map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO activity_log (id, project_id, action, detail, created_at) VALUES (?1,?2,'updated',?3,?4)",
                rusqlite::params![Uuid::new_v4().to_string(), pid, format!("Project updated via AI: '{}' → '{}'", old_name, final_name), now],
            ).ok();
            Ok(format!("Updated project: {} → {}", old_name, final_name))
        }
        "create_member" => {
            let name = action.data["name"].as_str().unwrap_or("").trim().to_string();
            if name.is_empty() {
                return Err("Member name is required".into());
            }
            // Deduplicate by name (case-insensitive)
            let existing: Option<String> = conn.query_row(
                "SELECT id FROM members WHERE lower(name)=lower(?1)",
                rusqlite::params![name],
                |r| r.get(0),
            ).ok();
            if let Some(eid) = existing {
                return Ok(format!("Member '{}' already exists (id: {})", name, eid));
            }
            let id = Uuid::new_v4().to_string();
            let email = action.data["email"].as_str().unwrap_or("");
            let colors = ["#f97316","#3b82f6","#8b5cf6","#ef4444","#10b981","#f59e0b","#ec4899","#06b6d4"];
            let color_idx: usize = conn.query_row("SELECT COUNT(*) FROM members", [], |r| r.get::<_,usize>(0)).unwrap_or(0) % colors.len();
            let color = colors[color_idx];
            conn.execute(
                "INSERT INTO members (id, name, email, color, created_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![id, name, email, color, now],
            ).map_err(|e| e.to_string())?;
            Ok(format!("Created member: {}", name))
        }
        "add_member_to_project" => {
            let project_name = action.data["project_name"].as_str().unwrap_or("");
            let member_name  = action.data["member_name"].as_str().unwrap_or("");
            if project_name.is_empty() || member_name.is_empty() {
                return Err("project_name and member_name are required for add_member_to_project".into());
            }
            let project_id: String = conn.query_row(
                "SELECT id FROM projects WHERE lower(name)=lower(?1)",
                rusqlite::params![project_name],
                |r| r.get(0),
            ).map_err(|_| format!("Project '{}' not found", project_name))?;
            let member_id: String = conn.query_row(
                "SELECT id FROM members WHERE lower(name)=lower(?1)",
                rusqlite::params![member_name],
                |r| r.get(0),
            ).map_err(|_| format!("Member '{}' not found — create the member first", member_name))?;
            conn.execute(
                "INSERT OR IGNORE INTO project_members (project_id, member_id, role) VALUES (?1,?2,'member')",
                rusqlite::params![project_id, member_id],
            ).map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO activity_log (id, project_id, action, detail, created_at) VALUES (?1,?2,'member_added',?3,?4)",
                rusqlite::params![Uuid::new_v4().to_string(), project_id, format!("Member '{}' added to project '{}' via AI", member_name, project_name), now],
            ).ok();
            Ok(format!("Added {} to project {}", member_name, project_name))
        }
        "create_item" => {
            let id = Uuid::new_v4().to_string();
            let title = action.data["title"].as_str().unwrap_or("Untitled");
            let item_type = action.data["type"].as_str().unwrap_or("issue");
            let priority = action.data["priority"].as_str().unwrap_or("p2");
            let status = action.data["status"].as_str().unwrap_or("open");
            let assignee = action.data["assignee"].as_str().unwrap_or("");
            let body = action.data["body"].as_str().unwrap_or("");
            // Validate project_id: only use it if it matches an actual project UUID in the DB
            let raw_pid = action.data["project_id"].as_str()
                .filter(|s| !s.is_empty() && !s.contains("PLACEHOLDER") && !s.contains(" "));
            let project_id: Option<&str> = if let Some(pid) = raw_pid {
                let exists: bool = conn.query_row(
                    "SELECT 1 FROM projects WHERE id=?1",
                    rusqlite::params![pid],
                    |_| Ok(true)
                ).unwrap_or(false);
                if exists { Some(pid) } else { None }
            } else {
                None
            };
            conn.execute(
                "INSERT INTO items (id, project_id, type, title, body, priority, status, assignee, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                rusqlite::params![id, project_id, item_type, title, body, priority, status, assignee, now, now],
            ).map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO activity_log (id, item_id, action, detail, created_at) VALUES (?1,?2,'created',?3,?4)",
                rusqlite::params![Uuid::new_v4().to_string(), id, format!("{} '{}' created via AI", item_type, title), now],
            ).ok();
            Ok(format!("Created {}: {}", item_type, title))
        }
        "create_task" => {
            let id = Uuid::new_v4().to_string();
            let project_id = action.data["project_id"].as_str().unwrap_or("");
            let title = action.data["title"].as_str().unwrap_or("Untitled task");
            let desc = action.data["description"].as_str().unwrap_or("");
            let status = action.data["status"].as_str().unwrap_or("todo");
            if project_id.is_empty() || project_id.contains("PLACEHOLDER") {
                return Err("Task needs a valid project_id. Create or select a project first.".into());
            }
            let max_pos: i32 = conn.query_row("SELECT COALESCE(MAX(position),0) FROM tasks WHERE project_id=?1", [project_id], |r| r.get(0)).unwrap_or(0);
            conn.execute(
                "INSERT INTO tasks (id, project_id, title, description, status, position, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                rusqlite::params![id, project_id, title, desc, status, max_pos + 1, now, now],
            ).map_err(|e| e.to_string())?;
            Ok(format!("Created task: {}", title))
        }
        _ => Err(format!("Unknown action type: {}", action.action_type)),
    }
}

#[tauri::command]
pub fn list_ai_conversations(db: State<Database>) -> Result<Vec<AiConversation>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, title, created_at, updated_at FROM ai_conversations ORDER BY updated_at DESC LIMIT 50").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(AiConversation {
        id: r.get(0)?, title: r.get(1)?, created_at: r.get(2)?, updated_at: r.get(3)?,
    })).map_err(|e| e.to_string())?;
    let mut convos = Vec::new();
    for r in rows { convos.push(r.map_err(|e| e.to_string())?); }
    Ok(convos)
}

#[tauri::command]
pub fn get_ai_messages(db: State<Database>, conversation_id: String) -> Result<Vec<AiMessage>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, actions_json, followups_json, created_at FROM ai_messages WHERE conversation_id=?1 ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([&conversation_id], |r| {
        let actions_str: String = r.get::<_, Option<String>>(4)?.unwrap_or_default();
        let followups_str: String = r.get::<_, Option<String>>(5)?.unwrap_or_default();
        Ok(AiMessage {
            id: r.get(0)?,
            conversation_id: r.get(1)?,
            role: r.get(2)?,
            content: r.get(3)?,
            actions: serde_json::from_str(&actions_str).unwrap_or_default(),
            followups: serde_json::from_str(&followups_str).unwrap_or_default(),
            created_at: r.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut messages = Vec::new();
    for r in rows { messages.push(r.map_err(|e| e.to_string())?); }
    Ok(messages)
}
