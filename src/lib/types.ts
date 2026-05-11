// ─── Projects ───────────────────────────────────────
export interface Project {
  id: string;
  name: string;
  description: string;
  stage: string;
  health: string;
  progress: number;
  due_date: string | null;
  owner_id: string | null;
  created_at: string;
  updated_at: string;
  owner_name: string | null;
  members: Member[];
  tasks: Task[];
}

export interface CreateProject {
  name: string;
  description?: string;
  stage?: string;
  health?: string;
  progress?: number;
  due_date?: string;
  owner_id?: string;
  member_ids?: string[];
}

export interface UpdateProject {
  id: string;
  name?: string;
  description?: string;
  stage?: string;
  health?: string;
  progress?: number;
  due_date?: string;
  owner_id?: string;
  member_ids?: string[];
}

// ─── Items ──────────────────────────────────────────
export type ItemType = "issue" | "followup" | "note" | "decision" | "waiting" | "idea" | "initiative";
export type Priority = "p0" | "p1" | "p2" | "p3";
export type Status = "open" | "in_progress" | "blocked" | "waiting" | "done" | "snoozed" | "brainstorm" | "exploring" | "planned" | "ready" | "pending_me";

// Follow-up tag categories
export type FollowupTag = "vapt" | "sca" | "bank_dependency" | "network_issue" | "infra" | "compliance" | "other";
export const FOLLOWUP_TAGS: { value: FollowupTag; label: string; color: string }[] = [
  { value: "vapt", label: "VAPT", color: "#ef4444" },
  { value: "sca", label: "SCA", color: "#f97316" },
  { value: "bank_dependency", label: "Bank Dependency", color: "#eab308" },
  { value: "network_issue", label: "Network Issue", color: "#3b82f6" },
  { value: "infra", label: "Infra", color: "#8b5cf6" },
  { value: "compliance", label: "Compliance", color: "#06b6d4" },
  { value: "other", label: "Other", color: "#737373" },
];

export interface Item {
  id: string;
  project_id: string | null;
  type: ItemType;
  title: string;
  body: string;
  priority: Priority;
  status: Status;
  assignee: string;
  due_at: string | null;
  created_at: string;
  updated_at: string;
  category: string;
  tags: Tag[];
  project_name: string | null;
}

export interface CreateItem {
  project_id?: string;
  type?: ItemType;
  title: string;
  body?: string;
  priority?: Priority;
  status?: Status;
  assignee?: string;
  due_at?: string;
  tag_ids?: string[];
  category?: string;
}

export interface UpdateItem {
  id: string;
  project_id?: string;
  type?: ItemType;
  title?: string;
  body?: string;
  priority?: Priority;
  status?: Status;
  assignee?: string;
  due_at?: string;
  tag_ids?: string[];
  category?: string;
}

// ─── Tags ───────────────────────────────────────────
export interface Tag {
  id: string;
  name: string;
  color: string;
  category: string;
}

export interface CreateTag {
  name: string;
  color?: string;
  category?: string;
}

// ─── Reminders ─────────────────────────────────────
export interface Reminder {
  id: string;
  item_id: string;
  item_title: string;
  due_at: string;
  recurrence: string | null;
  completed: boolean;
  snoozed_to: string | null;
  tune: string;
  label: string;
}

// ─── Boards ─────────────────────────────────────────
export interface Board {
  id: string;
  name: string;
  created_at: string;
  columns: BoardColumn[];
}

export interface BoardColumn {
  id: string;
  board_id: string;
  name: string;
  color: string;
  position: number;
  cards: BoardCard[];
}

export interface BoardCard {
  id: string;
  column_id: string;
  item_id: string | null;
  title: string;
  body: string;
  position: number;
  created_at: string;
  tags: Tag[];
}

export interface CreateBoard {
  name: string;
  columns?: { name: string; color?: string }[];
}

export interface CreateBoardCard {
  column_id: string;
  title: string;
  body?: string;
  item_id?: string;
  tag_ids?: string[];
}

export interface MoveCard {
  card_id: string;
  target_column_id: string;
  target_position: number;
}

// ─── Activity ───────────────────────────────────────
export interface ActivityEntry {
  id: string;
  item_id: string | null;
  project_id: string | null;
  action: string;
  detail: string;
  created_at: string;
}

// ─── Dashboard ──────────────────────────────────────
export interface DashboardStats {
  p0_count: number;
  p1_count: number;
  p2_count: number;
  p3_count: number;
  waiting_count: number;
  waiting_overdue: number;
  followups_today: number;
  projects_at_risk: number;
  projects_on_track: number;
  total_projects: number;
}

// ─── Check-in Engine ────────────────────────────────
export interface CheckinItem {
  id: string;
  title: string;
  item_type: string;
  priority: string;
  status: string;
  assignee: string;
  project_name: string;
  due_at: string | null;
  updated_at: string;
  age_days: number;
  reason: "overdue" | "stale" | "blocked" | "due_soon" | "pending_me";
}

export interface CheckinProject {
  id: string;
  name: string;
  health: string;
  stage: string;
  last_activity_days: number;
}

export interface CheckinReport {
  overdue: CheckinItem[];
  stale: CheckinItem[];
  blocked: CheckinItem[];
  due_soon: CheckinItem[];
  pending_me: CheckinItem[];
  at_risk_projects: CheckinProject[];
  total_attention: number;
  generated_at: string;
}

export type ViewId =
  | "dashboard"
  | "issues"
  | "followups"
  | "ideas"
  | "initiatives"
  | "projects"
  | "project_detail"
  | "board"
  | "today"
  | "week"
  | "ai"
  | "checkin"
  | "settings"
  | "pending_on_me";

// ─── Members ────────────────────────────────────────
export interface Member {
  id: string;
  name: string;
  email: string;
  color: string;
  created_at: string;
}

export interface CreateMember {
  name: string;
  email?: string;
  color?: string;
}

export interface UpdateMember {
  id: string;
  name?: string;
  email?: string;
  color?: string;
}

// ─── Tasks ──────────────────────────────────────────
export type TaskStatus = "todo" | "in_progress" | "review" | "blocked" | "done" | "pending_me";

export interface Task {
  id: string;
  project_id: string;
  title: string;
  description: string;
  status: TaskStatus;
  assignee_id: string | null;
  position: number;
  created_at: string;
  updated_at: string;
  assignee_name: string | null;
  project_name?: string | null;
}

export interface CreateTask {
  project_id: string;
  title: string;
  description?: string;
  status?: TaskStatus;
  assignee_id?: string;
}

export interface UpdateTask {
  id: string;
  title?: string;
  description?: string;
  status?: TaskStatus;
  assignee_id?: string;
  position?: number;
}

// ─── AI ─────────────────────────────────────────────
export interface AiChatRequest {
  message: string;
  conversation_id?: string;
}

export interface AiAction {
  id: string;
  type: string;
  label: string;
  data: Record<string, any>;
  status: "proposed" | "applied" | "rejected";
}

export interface AiChatResponse {
  conversation_id: string;
  message: string;
  actions: AiAction[];
  followups: string[];
}

export interface AiConversation {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface AiMessage {
  id: string;
  conversation_id: string;
  role: "user" | "assistant";
  content: string;
  actions: AiAction[];
  applied_summary?: string[];
  followups: string[];
  created_at: string;
}
