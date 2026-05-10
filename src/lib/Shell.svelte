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
  import QuickAdd from "./components/QuickAdd.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import Toaster from "./components/Toaster.svelte";
  import {
    activeView,
    isDark,
    sidebarCollapsed,
    showQuickAdd,
    showCommandPalette,
  } from "./stores/app";
  import { onMount } from "svelte";
  import { getSetting } from "./api";

  onMount(async () => {
    // Load theme preference
    const theme = await getSetting("theme").catch(() => null);
    if (theme === "light") {
      isDark.set(false);
      document.documentElement.setAttribute("data-theme", "light");
    }
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
      if (e.key === "Escape") {
        showCommandPalette.set(false);
        showQuickAdd.set(false);
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  });
</script>

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
        {/if}
      </div>
    </div>
  </div>
</div>

{#if $showQuickAdd}
  <QuickAdd />
{/if}

{#if $showCommandPalette}
  <CommandPalette />
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
