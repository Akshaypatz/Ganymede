<script lang="ts">
  import Sidebar from "./Sidebar.svelte";
  import Topbar from "./Topbar.svelte";
  import DashboardView from "./views/DashboardView.svelte";
  import IssuesView from "./views/IssuesView.svelte";
  import FollowupsView from "./views/FollowupsView.svelte";
  import IdeasView from "./views/IdeasView.svelte";
  import InitiativesView from "./views/InitiativesView.svelte";
  import ProjectsView from "./views/ProjectsView.svelte";
  import ProjectDetailView from "./views/ProjectDetailView.svelte";
  import BoardView from "./views/BoardView.svelte";
  import SettingsView from "./views/SettingsView.svelte";
  import AiView from "./views/AiView.svelte";
  import CheckinView from "./views/CheckinView.svelte";
  import PendingOnMeView from "./views/PendingOnMeView.svelte";
  import QuickAdd from "./components/QuickAdd.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import Toaster from "./components/Toaster.svelte";
  import ReminderNotification from "./components/ReminderNotification.svelte";
  import Onboarding from "./components/Onboarding.svelte";
  import LockScreen from "./components/LockScreen.svelte";
  import {
    activeView,
    isDark,
    sidebarCollapsed,
    showQuickAdd,
    showCommandPalette,
    checkinReport,
    isLocked,
    currentUser,
    showOnboarding,
    dueReminders,
  } from "./stores/app";
  import { onMount } from "svelte";
  import { getSetting, getCheckinReport, getDueReminders, listMembers, createMember } from "./api";
  import type { Reminder } from "./types";

  onMount(async () => {
    // Load theme preference
    const theme = await getSetting("theme").catch(() => null);
    if (theme === "light") {
      isDark.set(false);
      document.documentElement.setAttribute("data-theme", "light");
    }

    // Check if first-time onboarding is needed
    const userName = await getSetting("user_name").catch(() => null);
    if (!userName) {
      showOnboarding.set(true);
    } else {
      currentUser.set({ name: userName });
      // Always ensure the current user exists as a team member
      try {
        const memberList = await listMembers();
        if (!memberList.some(m => m.name === userName)) {
          await createMember({ name: userName, color: "#f97316" }).catch(() => {});
        }
      } catch {}
    }

    // Screen lock on visibility change (covers screen sleep / OS lock)
    let hiddenAt = 0;
    const onVisChange = async () => {
      if (document.hidden) {
        hiddenAt = Date.now();
      } else if (hiddenAt > 0) {
        const hiddenMs = Date.now() - hiddenAt;
        hiddenAt = 0;
        const lockEnabled = await getSetting("lock_enabled").catch(() => "true");
        // Only lock after 5 minutes away — prevents locking on brief app switches
        if (lockEnabled !== "false" && hiddenMs > 300_000) {
          isLocked.set(true);
        }
      }
    };
    document.addEventListener("visibilitychange", onVisChange);

    // Initial check-in report (silent)
    getCheckinReport().then(r => checkinReport.set(r)).catch(() => {});
    const interval = setInterval(() => {
      getCheckinReport().then(r => checkinReport.set(r)).catch(() => {});
    }, 30 * 60 * 1000);

    // Reminder polling every 30 seconds
    async function checkReminders() {
      try {
        const due = await getDueReminders();
        if (due.length > 0) dueReminders.update(existing => {
          const existingIds = new Set(existing.map(r => r.id));
          return [...existing, ...due.filter(r => !existingIds.has(r.id))];
        });
      } catch {}
    }
    checkReminders();
    const reminderInterval = setInterval(checkReminders, 30_000);

    // Global keyboard shortcuts
    const handleKey = (e: KeyboardEvent) => {
      const meta = e.metaKey || e.ctrlKey;
      if (meta && e.key === "k") {
        e.preventDefault();
        showCommandPalette.update((v) => !v);
      }
      if (meta && e.key === "n") {
        e.preventDefault();
        showQuickAdd.update((v) => !v);
      }
      if (meta && e.key === "l") {
        e.preventDefault();
        isLocked.set(true);
      }
      if (e.key === "Escape") {
        showCommandPalette.set(false);
        showQuickAdd.set(false);
      }
    };
    window.addEventListener("keydown", handleKey);

    return () => {
      window.removeEventListener("keydown", handleKey);
      document.removeEventListener("visibilitychange", onVisChange);
      clearInterval(interval);
      clearInterval(reminderInterval);
    };
  });
</script>

{#if $showOnboarding}
  <Onboarding />
{:else if $isLocked}
  <LockScreen />
{:else}
  <div class="shell" class:sidebar-collapsed={$sidebarCollapsed}>
    <Sidebar />
    <div class="main-wrap">
      <Topbar />
      <div class="view-wrap">
        <div class="content-scroll">
          {#if $activeView === "dashboard"}
            <DashboardView />
          {:else if $activeView === "issues"}
            <IssuesView />
          {:else if $activeView === "followups"}
            <FollowupsView />
          {:else if $activeView === "ideas"}
            <IdeasView />
          {:else if $activeView === "initiatives"}
            <InitiativesView />
          {:else if $activeView === "projects"}
            <ProjectsView />
          {:else if $activeView === "project_detail"}
            <ProjectDetailView />
          {:else if $activeView === "board"}
            <BoardView />
          {:else if $activeView === "settings"}
            <SettingsView />
          {:else if $activeView === "ai"}
            <AiView />
          {:else if $activeView === "checkin"}
            <CheckinView />
          {:else if $activeView === "pending_on_me"}
            <PendingOnMeView />
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

{#if !$showOnboarding && !$isLocked}
  {#if $showQuickAdd}
    <QuickAdd />
  {/if}

  {#if $showCommandPalette}
    <CommandPalette />
  {/if}

  {#if $dueReminders.length > 0}
    <ReminderNotification
      reminders={$dueReminders}
      on:dismiss={(e) => { dueReminders.update(rs => rs.filter(r => r.id !== e.detail)); }}
    />
  {/if}
{/if}

<Toaster />

<style>
  .shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
    position: relative;
  }
  .main-wrap {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }
  .view-wrap {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
  .content-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 20px 22px;
    min-width: 0;
  }
  /* AI view gets no padding — it manages its own layout */
  :global(.content-scroll:has(.ai-view)) {
    padding: 0;
    overflow: hidden;
  }
</style>
