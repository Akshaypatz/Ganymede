<script lang="ts">
  import { toasts } from "../stores/app";
</script>

<div class="toaster">
  {#each $toasts as toast (toast.id)}
    <div class="toast">
      {#if toast.icon}<span>{toast.icon}</span>{/if}
      {toast.message}
    </div>
  {/each}
</div>
