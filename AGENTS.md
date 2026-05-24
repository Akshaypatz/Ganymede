# Ganymede – AI Agent / Coding Agent Guide

> This file is the authoritative reference for any AI coding agent (Claude, GitHub Copilot, etc.) working in this repository.

---

## Project Overview

**Ganymede** is a cross-platform desktop app built with **Tauri 2 + Svelte 5 + TypeScript** (frontend) and **Rust** (backend). It is an operations co-pilot for engineering/product teams: it manages projects, issues, tasks, members, boards, check-ins, reminders, and an AI chat assistant.

### Stack
| Layer | Tech |
|---|---|
| Desktop shell | Tauri 2 (Rust) |
| Frontend | Svelte 5, TypeScript, Vite 8 |
| Backend / DB | Rust, SQLite via `rusqlite` (bundled) |
| AI integration | OpenAI-compatible REST API (configurable provider) |
| Styling | CSS variables in `src/styles/global.css` |

---

## Repository Structure

```
Ganymede/
├── src/                        # Svelte/TypeScript frontend
│   ├── App.svelte              # Root component, routing
│   ├── main.ts                 # Entry point
│   ├── lib/
│   │   ├── api.ts              # All Tauri invoke() wrappers
│   │   ├── types.ts            # Shared TypeScript types
│   │   ├── Shell.svelte        # App shell (sidebar + topbar)
│   │   ├── Sidebar.svelte      # Navigation sidebar
│   │   ├── Topbar.svelte       # Top bar
│   │   ├── tunes.ts            # Audio notification system
│   │   ├── components/         # Reusable UI components
│   │   ├── stores/app.ts       # Svelte stores (global state)
│   │   └── views/              # Page-level views
│   └── styles/global.css       # CSS custom properties / design tokens
├── src-tauri/
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri config (app name, version, bundle IDs)
│   └── src/
│       ├── lib.rs              # Tauri builder + command registration
│       ├── main.rs             # Entry point (calls lib::run)
│       ├── models.rs           # Rust structs (Serialize/Deserialize)
│       ├── commands.rs         # All Tauri commands (CRUD)
│       ├── ai.rs               # AI chat engine + apply_ai_action
│       ├── db.rs               # Database init + migrations
│       ├── auth.rs             # Biometric / password auth
│       ├── checkin.rs          # Check-in engine
│       └── reminders.rs        # Reminder system
├── package.json                # Node deps (Svelte, Vite, xlsx)
├── BUILD.md                    # Build & release commands (macOS + Windows)
├── AGENTS.md                   # This file
└── beta-preview/               # Pre-built DMG for testing
```

---

## Key Architectural Decisions

1. **All data lives in SQLite** – `db.rs` handles migrations. The DB path is `app_data_dir/ganymede.db`.
2. **Frontend calls Rust via `invoke()`** – every API call in `api.ts` corresponds to a `#[tauri::command]` function registered in `lib.rs`.
3. **AI actions are two-step**: The frontend calls `chat_with_ai` (gets message + proposed actions), then the user clicks "Apply" → `apply_ai_action` executes the action.
4. **No external state management library** – everything is plain Svelte stores in `stores/app.ts`.
5. **Design tokens** – all colors/spacing use CSS variables. Never hardcode colors.

---

## Data Model Summary

### Projects
`projects` table: `id, name, description, stage, health, progress, due_date, owner_id, created_at, updated_at`
- `stage`: `solutioning_pending | in_development | released | live`
- `health`: `green | amber | red`
- Members linked via `project_members(project_id, member_id, role)`

### Items
`items` table: `id, project_id, type, title, body, priority, status, assignee, due_at, created_at, updated_at, category`
- `type`: `issue | followup | note | decision | waiting | idea | initiative`
- `priority`: `p0 | p1 | p2 | p3`
- `status`: `open | in_progress | blocked | waiting | done | snoozed | brainstorm | exploring | planned | ready | pending_me`
- Tags linked via `item_tags(item_id, tag_id)`

### Members
`members` table: `id, name, email, color, created_at`

### Tasks
`tasks` table: `id, project_id, title, description, status, assignee_id, position, created_at, updated_at`
- `status`: `todo | in_progress | review | blocked | done`

### Boards
`boards` → `board_columns` → `board_cards`

---

## AI Action Types

The `apply_ai_action` function in `ai.rs` handles these action types. The system prompt in `ai.rs` teaches the LLM to generate them:

| Action type | Description |
|---|---|
| `create_project` | Creates a new project (deduplicates by name) |
| `update_project` | Renames or updates a project's fields |
| `create_item` | Creates an issue/followup/idea/etc. |
| `create_task` | Creates a task within a project |
| `create_member` | Creates a new team member |
| `add_member_to_project` | Adds an existing member to a project |

### Key Rules (enforced in system prompt)
- `create_project` is idempotent: if a project with the same name exists, it returns the existing one
- `project_id` in `create_item` is always left empty (AI doesn't know UUIDs)
- The AI must return valid JSON `{ message, actions, followups }`

---

## Adding a New Tauri Command

1. Add struct to `models.rs` if needed
2. Implement the command in `commands.rs` (or relevant module)
3. Register it in `lib.rs` inside `tauri::generate_handler![...]`
4. Add the TypeScript wrapper in `api.ts`
5. Add the TypeScript type in `types.ts` if needed

---

## Frontend Conventions

- **Views** go in `src/lib/views/`. They are loaded via a store-driven router in `App.svelte`.
- **Components** go in `src/lib/components/`. Keep them self-contained.
- **Stores** are all in `stores/app.ts`. Avoid creating new store files unless the concern is clearly separate.
- **CSS**: use `var(--orange)`, `var(--fg)`, `var(--surface)`, `var(--border)`, `var(--r)`, `var(--r-lg)` etc. — all defined in `global.css`.
- **Svelte 5 syntax**: use `$props()`, `$state()` where applicable in new components. Existing components use Svelte 4 syntax — be consistent within a file.

---

## Development Commands

```bash
# Install dependencies
npm install

# Run in development mode (hot reload)
npm run tauri dev

# Build production app (macOS)
npm run tauri build

# Type-check frontend
npx tsc --noEmit

# Run Rust tests
cd src-tauri && cargo test
```

See `BUILD.md` for full platform-specific build and release instructions.

---

## Common Pitfalls

- **Never hold a `MutexGuard` across an `await`** – Rust will deadlock. The `chat_with_ai` function demonstrates the correct pattern: acquire lock in a scoped block, drop it, then `await`.
- **SQLite is synchronous** – all DB calls are blocking. The `async` Tauri commands must release the mutex before any `await`.
- **`project_id` from AI is unreliable** – the AI doesn't know real UUIDs. Always validate `project_id` before inserting, as done in `apply_ai_action`.
- **Svelte reactivity** – when mutating stores, always return a new array/object to trigger updates. Mutating in place won't re-render.

---

## Testing

Rust unit tests live in `src-tauri/src/tests.rs` (included via `#[cfg(test)]` in `lib.rs`). Run with:
```bash
cd src-tauri && cargo test -- --nocapture
```

Frontend has no automated test suite yet. Manual testing is required after changes to:
- `apply_ai_action` in `ai.rs`
- Column mapping in `SettingsView.svelte`
- Any new Tauri command

---

## Export / Import Format

The native Ganymede JSON export (`export_json` command) produces:
```json
{
  "version": 1,
  "exported_at": "<ISO timestamp>",
  "projects": [...],
  "items": [...],
  "members": [...],
  "tasks": [...],
  "tags": [...],
  "boards": [...]
}
```
This format is accepted by the `import_json` command for full data restoration.
