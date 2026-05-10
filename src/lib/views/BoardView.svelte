<script lang="ts">
  import { onMount } from "svelte";
  import { boards, activeBoardId, tags, addToast } from "../stores/app";
  import { listBoards, getBoard, createBoard, addBoardCard, moveCard, listTags, addBoardColumn, deleteBoardCard } from "../api";
  import type { Board, BoardColumn, BoardCard, Tag } from "../types";

  let currentBoard: Board | null = null;
  let showCreateBoard = false;
  let newBoardName = "";
  let newBoardColumns = "Todo, In Progress, Done";
  let addingCardToCol: string | null = null;
  let newCardTitle = "";
  let allTags: Tag[] = [];
  let dragCard: BoardCard | null = null;
  let dragOverCol: string | null = null;

  onMount(async () => {
    try {
      allTags = await listTags();
      tags.set(allTags);
    } catch {}
    try {
      const boardList = await listBoards();
      boards.set(boardList);
      if (boardList.length > 0) {
        const bid = $activeBoardId || boardList[0].id;
        activeBoardId.set(bid);
        currentBoard = await getBoard(bid);
      }
    } catch {}
  });

  // Watch for board switches
  $: if ($activeBoardId) {
    loadBoard($activeBoardId);
  }

  async function loadBoard(id: string) {
    try {
      currentBoard = await getBoard(id);
    } catch {}
  }

  async function handleCreateBoard() {
    if (!newBoardName.trim()) return;
    const cols = newBoardColumns.split(",").map((s) => s.trim()).filter(Boolean).map((name) => ({ name }));
    const board = await createBoard({ name: newBoardName.trim(), columns: cols.length > 0 ? cols : undefined });
    boards.update((b) => [...b, board]);
    activeBoardId.set(board.id);
    currentBoard = board;
    newBoardName = "";
    newBoardColumns = "Todo, In Progress, Done";
    showCreateBoard = false;
    addToast(`Board created: "${board.name}"`, "✓");
  }

  async function handleAddCard(colId: string) {
    if (!newCardTitle.trim()) return;
    await addBoardCard({ column_id: colId, title: newCardTitle.trim() });
    if (currentBoard) currentBoard = await getBoard(currentBoard.id);
    newCardTitle = "";
    addingCardToCol = null;
    addToast("Card added", "✓");
  }

  async function handleAddColumn() {
    if (!currentBoard) return;
    const name = prompt("Column name:");
    if (!name?.trim()) return;
    await addBoardColumn(currentBoard.id, name.trim());
    currentBoard = await getBoard(currentBoard.id);
    addToast(`Column "${name}" added`, "✓");
  }

  // Drag and drop handlers
  function onDragStart(e: DragEvent, card: BoardCard) {
    dragCard = card;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", card.id);
    }
  }

  function onDragOver(e: DragEvent, colId: string) {
    e.preventDefault();
    dragOverCol = colId;
  }

  function onDragLeave() {
    dragOverCol = null;
  }

  async function onDrop(e: DragEvent, colId: string) {
    e.preventDefault();
    dragOverCol = null;
    if (!dragCard || !currentBoard) return;
    if (dragCard.column_id === colId) return;
    const targetCol = currentBoard.columns.find((c) => c.id === colId);
    const pos = targetCol ? targetCol.cards.length : 0;
    await moveCard({ card_id: dragCard.id, target_column_id: colId, target_position: pos });
    dragCard = null;
    currentBoard = await getBoard(currentBoard.id);
    addToast("Card moved", "✓");
  }

  async function handleDeleteCard(cardId: string) {
    await deleteBoardCard(cardId);
    if (currentBoard) currentBoard = await getBoard(currentBoard.id);
    addToast("Card deleted");
  }
</script>

<div class="page-header">
  <div class="page-title">{currentBoard?.name || "Boards"}</div>
  <div class="page-sub">
    {#if currentBoard}
      {currentBoard.columns.reduce((sum, col) => sum + col.cards.length, 0)} cards across {currentBoard.columns.length} columns
    {:else}
      No board selected
    {/if}
  </div>
</div>

<div class="board-toolbar">
  {#each $boards as board}
    <button class="btn" class:btn-primary={$activeBoardId === board.id} class:btn-ghost={$activeBoardId !== board.id}
      on:click={() => activeBoardId.set(board.id)}>{board.name}</button>
  {/each}
  <button class="btn btn-ghost" on:click={() => (showCreateBoard = !showCreateBoard)}>+ New Board</button>
</div>

{#if showCreateBoard}
  <div class="create-form">
    <input bind:value={newBoardName} placeholder="Board name…" class="create-input" on:keydown={(e) => e.key === "Enter" && handleCreateBoard()} />
    <input bind:value={newBoardColumns} placeholder="Columns (comma-separated)" class="create-input" />
    <button class="btn btn-primary" on:click={handleCreateBoard}>Create</button>
    <button class="btn btn-ghost" on:click={() => (showCreateBoard = false)}>Cancel</button>
  </div>
{/if}

{#if currentBoard}
  <div class="board-wrap">
    {#each currentBoard.columns as col}
      <div
        class="board-col"
        class:drag-over={dragOverCol === col.id}
        on:dragover={(e) => onDragOver(e, col.id)}
        on:dragleave={onDragLeave}
        on:drop={(e) => onDrop(e, col.id)}
        role="list"
      >
        <div class="board-col-hdr">
          <span class="board-col-name">{col.name}</span>
          <span class="board-col-cnt">{col.cards.length}</span>
        </div>
        <div class="board-cards">
          {#each col.cards as card}
            <div
              class="board-card"
              draggable="true"
              on:dragstart={(e) => onDragStart(e, card)}
              role="listitem"
            >
              <div class="bc-title">{card.title}</div>
              <div class="bc-meta">
                {#each card.tags as tag}
                  <span class="bc-tag" style="background:{tag.color}20;color:{tag.color}">{tag.name}</span>
                {/each}
                <button class="bc-delete" on:click|stopPropagation={() => handleDeleteCard(card.id)} title="Delete card">×</button>
              </div>
            </div>
          {/each}
        </div>
        <div class="board-add">
          {#if addingCardToCol === col.id}
            <input
              bind:value={newCardTitle}
              placeholder="Card title…"
              class="card-input"
              on:keydown={(e) => { if (e.key === "Enter") handleAddCard(col.id); if (e.key === "Escape") addingCardToCol = null; }}
              autofocus
            />
          {:else}
            <button class="board-add-btn" on:click={() => { addingCardToCol = col.id; newCardTitle = ""; }}>+ Add card</button>
          {/if}
        </div>
      </div>
    {/each}
    <div class="board-col add-col">
      <button class="board-add-btn full" on:click={handleAddColumn}>+ Add column</button>
    </div>
  </div>
{:else}
  <div class="empty-state">Create a board to get started.</div>
{/if}

<style>
  .page-header { margin-bottom: 18px; }
  .page-title { font-size: 17px; font-weight: 750; letter-spacing: -0.03em; }
  .page-sub { font-size: 12px; color: var(--fg-3); margin-top: 3px; }

  .board-toolbar { display: flex; gap: 6px; margin-bottom: 12px; flex-wrap: wrap; }

  .create-form {
    display: flex; align-items: center; gap: 8px; padding: 10px 12px;
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg); margin-bottom: 14px;
  }
  .create-input {
    flex: 1; padding: 6px 10px; background: var(--surface-2);
    border: 1px solid var(--border); border-radius: var(--r); font-size: 12px; color: var(--fg);
  }
  .create-input:focus { border-color: var(--orange); }

  .board-wrap { display: flex; gap: 10px; overflow-x: auto; padding-block-end: 8px; }
  .board-col {
    min-width: 240px; max-width: 260px; flex-shrink: 0;
    background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg);
    display: flex; flex-direction: column; max-height: calc(100vh - 200px);
    transition: border-color 150ms;
  }
  .board-col.drag-over { border-color: var(--orange); background: var(--orange-bg); }
  .board-col.add-col {
    background: transparent; border-style: dashed; border-color: var(--border-2);
    display: flex; align-items: center; justify-content: center; min-height: 100px;
  }
  .board-col-hdr {
    padding: 10px 12px; border-block-end: 1px solid var(--border); flex-shrink: 0;
    display: flex; align-items: center; justify-content: space-between;
  }
  .board-col-name { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; }
  .board-col-cnt { font-size: 10px; font-weight: 700; color: var(--fg-4); background: var(--surface-2); padding: 1px 6px; border-radius: 99px; }
  .board-cards { flex: 1; overflow-y: auto; padding: 8px; }
  .board-card {
    background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r);
    padding: 10px 11px; margin-bottom: 6px; cursor: grab; transition: all 120ms;
  }
  .board-card:hover { border-color: var(--border-2); box-shadow: var(--shadow); }
  .bc-title { font-size: 12px; font-weight: 550; color: var(--fg); margin-bottom: 5px; line-height: 1.35; }
  .bc-meta { display: flex; align-items: center; gap: 5px; flex-wrap: wrap; }
  .bc-tag { font-size: 9px; font-weight: 600; padding: 1px 6px; border-radius: 4px; text-transform: uppercase; letter-spacing: 0.04em; }
  .bc-delete {
    margin-inline-start: auto; width: 16px; height: 16px; border-radius: 3px;
    font-size: 12px; color: var(--fg-4); cursor: pointer; display: flex;
    align-items: center; justify-content: center; opacity: 0; transition: opacity 100ms;
    background: none; border: none;
  }
  .board-card:hover .bc-delete { opacity: 1; }
  .bc-delete:hover { color: var(--p0); }
  .board-add { padding: 6px 12px 10px; flex-shrink: 0; border-block-start: 1px solid var(--border); }
  .board-add-btn {
    width: 100%; padding: 6px; border-radius: 6px; border: 1px dashed var(--border-2);
    color: var(--fg-4); font-size: 11px; display: flex; align-items: center;
    justify-content: center; gap: 5px; cursor: pointer; transition: all 120ms; background: none;
  }
  .board-add-btn:hover { background: var(--surface-2); color: var(--fg-3); border-color: var(--border-2); }
  .board-add-btn.full { border-style: dashed; padding: 12px; }
  .card-input {
    width: 100%; padding: 6px 8px; background: var(--surface-2); border: 1px solid var(--orange);
    border-radius: 6px; font-size: 11px; color: var(--fg);
  }

  .empty-state { font-size: 12px; color: var(--fg-4); padding: 24px 0; text-align: center; }
</style>
