import { writable } from "svelte/store";
import type { ViewId, Project, Item, Tag, Board, DashboardStats, ActivityEntry, Member, AiMessage, CheckinReport, Task, Reminder } from "../types";
import { FOLLOWUP_TAGS } from "../types";

// Active view
export const activeView = writable<ViewId>("dashboard");

// Theme
export const isDark = writable(true);

// Sidebar
export const sidebarCollapsed = writable(false);

// AI panel
export const aiPanelOpen = writable(false);

// Data stores
export const projects = writable<Project[]>([]);
export const items = writable<Item[]>([]);
export const tags = writable<Tag[]>([]);
export const boards = writable<Board[]>([]);
export const dashboardStats = writable<DashboardStats | null>(null);
export const activityLog = writable<ActivityEntry[]>([]);
export const members = writable<Member[]>([]);
// Cross-entity pending-on-me tasks (kept in sync across ProjectDetailView & PendingOnMeView)
export const pendingMeTasks = writable<Task[]>([]);

// Modals
export const showQuickAdd = writable(false);
export const showCommandPalette = writable(false);

// Active board for the board view
export const activeBoardId = writable<string | null>(null);

// Active project for the detail view
export const activeProjectId = writable<string | null>(null);

// Issues view filter
export const issueFilter = writable<"all">("all");

// Follow-up categories (configurable, defaults to FOLLOWUP_TAGS)
export const followupCategories = writable<{ value: string; label: string; color: string }[]>([...FOLLOWUP_TAGS]);

// Needs Attention rules (configurable)
export const attentionRules = writable({
  p0: true,
  blocked: true,
  stale_p1_days: 7,
  overdue_followups: true,
});

// AI conversation
export const aiConversationId = writable<string | null>(null);
export const aiMessages = writable<AiMessage[]>([]);
export const aiLoading = writable(false);

// Check-in report
export const checkinReport = writable<CheckinReport | null>(null);

// Auth / session
export const isLocked = writable(false);
export const currentUser = writable<{ name: string } | null>(null);
export const showOnboarding = writable(false);

// Due reminders (shared so SettingsView can inject test reminders)
export const dueReminders = writable<Reminder[]>([]);

// Toast messages
export const toasts = writable<{ id: number; message: string; icon?: string }[]>([]);
let toastId = 0;
export function addToast(message: string, icon?: string) {
  const id = ++toastId;
  toasts.update((t) => [...t, { id, message, icon }]);
  setTimeout(() => {
    toasts.update((t) => t.filter((x) => x.id !== id));
  }, 3000);
}
