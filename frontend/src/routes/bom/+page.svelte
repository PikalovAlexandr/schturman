<script lang="ts">
  import { onMount } from 'svelte';
  import { bom } from '$lib/api';
  import type { BOM } from '$lib/api';

  let boms: BOM[] = [];
  let loading = true;

  onMount(async () => {
    try {
      boms = await bom.list();
    } catch (error) {
      console.error('Failed to load BOMs:', error);
    } finally {
      loading = false;
    }
  });

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString('ru-RU');
  }
</script>

<div class="bom-page">
  <header class="page-header">
    <div>
      <h2>Спецификации (BOM)</h2>
      <p class="subtitle">Управление сборочными единицами</p>
    </div>
    <button class="primary">+ Создать спецификацию</button>
  </header>

  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Загрузка...</p>
    </div>
  {:else if boms.length === 0}
    <div class="empty-state card">
      <div class="empty-icon">📋</div>
      <h3>Нет спецификаций</h3>
      <p>Создайте первую спецификацию для начала работы</p>
    </div>
  {:else}
    <div class="bom-list">
      {#each boms as bomItem}
        <article class="bom-card card">
          <div class="bom-header">
            <h3>{bomItem.name}</h3>
            <span class="badge info">{bomItem.status}</span>
          </div>
          <div class="bom-meta">
            <span>Версия: {bomItem.version}</span>
            <span>Создано: {formatDate(bomItem.created_at)}</span>
          </div>
          <div class="bom-actions">
            <button class="secondary">Просмотр</button>
            <button class="secondary">Редактировать</button>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .bom-page {
    padding: var(--spacing-xl);
    max-width: 1600px;
    margin: 0 auto;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--spacing-xl);
  }

  .page-header h2 {
    font-size: clamp(24px, 3vw, 32px);
    margin-bottom: var(--spacing-xs);
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 14px;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-lg);
    padding: var(--spacing-xl) * 2;
    color: var(--text-secondary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--surface-3);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-xl) * 2;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: var(--spacing-lg);
    opacity: 0.5;
  }

  .bom-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: var(--spacing-lg);
  }

  .bom-card {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .bom-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .bom-meta {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    font-size: 13px;
    color: var(--text-secondary);
  }

  .bom-actions {
    display: flex;
    gap: var(--spacing-sm);
  }

  .bom-actions button {
    flex: 1;
  }
</style>
