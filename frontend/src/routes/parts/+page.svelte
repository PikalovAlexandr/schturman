<script lang="ts">
  import { onMount } from 'svelte';
  import { parts, ai } from '$lib/api';
  import type { Part } from '$lib/api';

  let allParts: Part[] = [];
  let filteredParts: Part[] = [];
  let loading = true;
  let searchQuery = '';
  let useAI = false;
  let aiSearching = false;

  onMount(async () => {
    try {
      allParts = await parts.list();
      filteredParts = allParts;
    } catch (error) {
      console.error('Failed to load parts:', error);
    } finally {
      loading = false;
    }
  });

  async function handleSearch() {
    if (!searchQuery.trim()) {
      filteredParts = allParts;
      return;
    }

    if (useAI) {
      aiSearching = true;
      try {
        filteredParts = await ai.semanticSearch(searchQuery);
      } catch (error) {
        console.error('AI search failed:', error);
        // Fallback to regular search
        filteredParts = await parts.search(searchQuery);
      } finally {
        aiSearching = false;
      }
    } else {
      try {
        filteredParts = await parts.search(searchQuery);
      } catch (error) {
        console.error('Search failed:', error);
      }
    }
  }

  function getStatusBadge(status: string) {
    const map: Record<string, string> = {
      draft: 'info',
      released: 'success',
      obsolete: 'error',
    };
    return map[status] || 'info';
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString('ru-RU', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  }
</script>

<div class="parts-page">
  <header class="page-header">
    <div>
      <h2>Части</h2>
      <p class="subtitle">Управление компонентами и деталями</p>
    </div>
    <button class="primary">+ Создать часть</button>
  </header>

  <div class="search-section card">
    <div class="search-bar">
      <input
        type="text"
        placeholder="Поиск частей..."
        bind:value={searchQuery}
        on:keydown={(e) => e.key === 'Enter' && handleSearch()}
      />
      <button class="primary" on:click={handleSearch} disabled={aiSearching}>
        {aiSearching ? '⏳' : '🔍'} {useAI ? 'AI Поиск' : 'Поиск'}
      </button>
    </div>
    
    <label class="ai-toggle">
      <input type="checkbox" bind:checked={useAI} />
      <span>Использовать AI поиск (семантический)</span>
    </label>
  </div>

  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Загрузка частей...</p>
    </div>
  {:else if filteredParts.length === 0}
    <div class="empty-state card">
      <div class="empty-icon">📦</div>
      <h3>Части не найдены</h3>
      <p>Попробуйте изменить параметры поиска или создайте новую часть</p>
    </div>
  {:else}
    <div class="parts-grid">
      {#each filteredParts as part}
        <article class="part-card card">
          <div class="part-header">
            <div>
              <h3 class="part-name">{part.name}</h3>
              <div class="part-number">{part.part_number}</div>
            </div>
            <span class="badge {getStatusBadge(part.status)}">{part.status}</span>
          </div>

          {#if part.description}
            <p class="part-description">{part.description}</p>
          {/if}

          <div class="part-meta">
            <div class="meta-item">
              <span class="meta-label">Тип:</span>
              <span class="meta-value">{part.type}</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">Создано:</span>
              <span class="meta-value">{formatDate(part.created_at)}</span>
            </div>
          </div>

          <div class="part-actions">
            <button class="secondary">Просмотр</button>
            <button class="secondary">Редактировать</button>
          </div>
        </article>
      {/each}
    </div>

    <div class="results-info">
      Найдено: {filteredParts.length} {filteredParts.length === 1 ? 'часть' : 'частей'}
    </div>
  {/if}
</div>

<style>
  .parts-page {
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

  .search-section {
    margin-bottom: var(--spacing-xl);
  }

  .search-bar {
    display: flex;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .search-bar input {
    flex: 1;
  }

  .ai-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 14px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .ai-toggle input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
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

  .empty-state h3 {
    font-size: 20px;
    margin-bottom: var(--spacing-sm);
    color: var(--text-primary);
  }

  .empty-state p {
    color: var(--text-secondary);
  }

  .parts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
  }

  .part-card {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .part-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }

  .part-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--spacing-md);
  }

  .part-name {
    font-size: 18px;
    color: var(--text-primary);
    margin-bottom: 4px;
  }

  .part-number {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .part-description {
    color: var(--text-secondary);
    font-size: 14px;
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .part-meta {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding-top: var(--spacing-sm);
    border-top: 1px solid var(--surface-3);
  }

  .meta-item {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }

  .meta-label {
    color: var(--text-tertiary);
  }

  .meta-value {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .part-actions {
    display: flex;
    gap: var(--spacing-sm);
    margin-top: auto;
  }

  .part-actions button {
    flex: 1;
  }

  .results-info {
    text-align: center;
    padding: var(--spacing-lg);
    color: var(--text-tertiary);
    font-size: 14px;
  }
</style>
