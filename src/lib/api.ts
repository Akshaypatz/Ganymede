import { invoke } from "@tauri-apps/api/core";
import type {
  Project, CreateProject, UpdateProject,
  Item, CreateItem, UpdateItem,
  Tag, CreateTag,
  Board, CreateBoard, CreateBoardCard, MoveCard,
  ActivityEntry, DashboardStats, BoardColumn,
  Member, CreateMember, UpdateMember,
  Task, CreateTask, UpdateTask,
  AiChatRequest, AiChatResponse, AiAction, AiConversation, AiMessage,
} from "./types";

// ─── Projects ───────────────────────────────────────
export const listProjects = () => invoke<Project[]>("list_projects");
export const createProject = (data: CreateProject) => invoke<Project>("create_project", { data });
export const updateProject = (data: UpdateProject) => invoke<Project>("update_project", { data });
export const deleteProject = (id: string) => invoke<void>("delete_project", { id });
export const getProject = (id: string) => invoke<Project>("get_project", { id });

// ─── Items ──────────────────────────────────────────
export const listItems = (filters?: {
  itemType?: string;
  status?: string;
  projectId?: string;
  priority?: string;
}) =>
  invoke<Item[]>("list_items", {
    itemType: filters?.itemType ?? null,
    status: filters?.status ?? null,
    projectId: filters?.projectId ?? null,
    priority: filters?.priority ?? null,
  });

export const createItem = (data: CreateItem) => invoke<Item>("create_item", { data });
export const updateItem = (data: UpdateItem) => invoke<Item>("update_item", { data });
export const deleteItem = (id: string) => invoke<void>("delete_item", { id });

// ─── Tags ───────────────────────────────────────────
export const listTags = () => invoke<Tag[]>("list_tags");
export const createTag = (data: CreateTag) => invoke<Tag>("create_tag", { data });
export const deleteTag = (id: string) => invoke<void>("delete_tag", { id });

// ─── Boards ─────────────────────────────────────────
export const listBoards = () => invoke<Board[]>("list_boards");
export const getBoard = (id: string) => invoke<Board>("get_board", { id });
export const createBoard = (data: CreateBoard) => invoke<Board>("create_board", { data });
export const addBoardColumn = (boardId: string, name: string, color?: string) =>
  invoke<BoardColumn>("add_board_column", { boardId, name, color: color ?? null });
export const addBoardCard = (data: CreateBoardCard) => invoke<any>("add_board_card", { data });
export const moveCard = (data: MoveCard) => invoke<void>("move_card", { data });
export const deleteBoard = (id: string) => invoke<void>("delete_board", { id });
export const deleteBoardColumn = (id: string) => invoke<void>("delete_board_column", { id });
export const deleteBoardCard = (id: string) => invoke<void>("delete_board_card", { id });

// ─── Activity ───────────────────────────────────────
export const listActivity = (limit?: number) =>
  invoke<ActivityEntry[]>("list_activity", { limit: limit ?? 20 });

// ─── Dashboard ──────────────────────────────────────
export const getDashboardStats = () => invoke<DashboardStats>("get_dashboard_stats");

// ─── Settings ───────────────────────────────────────
export const getSetting = (key: string) => invoke<string | null>("get_setting", { key });
export const setSetting = (key: string, value: string) =>
  invoke<void>("set_setting", { key, value });
export const resetAppData = () => invoke<void>("reset_app_data");

// ─── Members ────────────────────────────────────────
export const listMembers = () => invoke<Member[]>("list_members");
export const createMember = (data: CreateMember) => invoke<Member>("create_member", { data });
export const updateMember = (data: UpdateMember) => invoke<Member>("update_member", { data });
export const deleteMember = (id: string) => invoke<void>("delete_member", { id });

// ─── Tasks ──────────────────────────────────────────
export const listTasks = (projectId: string) => invoke<Task[]>("list_tasks", { projectId });
export const listBlockedTasks = () => invoke<Task[]>("list_blocked_tasks");
export const createTask = (data: CreateTask) => invoke<Task>("create_task", { data });
export const updateTask = (data: UpdateTask) => invoke<Task>("update_task", { data });
export const deleteTask = (id: string) => invoke<void>("delete_task", { id });

// ─── AI ─────────────────────────────────────────────
export const chatWithAi = (request: AiChatRequest) =>
  invoke<AiChatResponse>("chat_with_ai", { request });
export const applyAiAction = (action: AiAction) =>
  invoke<string>("apply_ai_action", { action });
export const listAiConversations = () =>
  invoke<AiConversation[]>("list_ai_conversations");
export const getAiMessages = (conversationId: string) =>
  invoke<AiMessage[]>("get_ai_messages", { conversationId });
