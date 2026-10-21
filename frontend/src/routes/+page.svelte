<script lang="ts">
  import { onMount } from 'svelte';
  import { parts, bom, changes, ai } from '$lib/api';
  import type { Part, BOM, Change, AITask } from '$lib/api';

  let stats = {
    partsCount: 0,
    bomsCount: 0,
    changesCount: 0,
    aiTasksCount: 0,
  };

  let recentParts: Part[] = [];
  let recentChanges: Change[] = [];
  let activeAITasks: AITask[] = [];
  let loading = true;

  onMount(async () => {
    try {
      const [allParts, allBoms, allChanges, allTasks] = await Promise.all([
        parts.list(),
        bom.list(),
        changes.list(),
        ai.listTasks(),
      ]);

      stats = {
        partsCount: allParts.length,
        bomsCount: allBoms.length,
        changesCount: allChanges.length,
        aiTasksCount: allTasks.filter(t => t.status === 'processing').length,
      };

      recentParts = allParts.slice(0, 5);
      recentChanges = allChanges.slice(0, 5);
      activeAITasks = allTasks.filter(t => t.status === 'processing' || t.status === 'queued').slice(0, 5);
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      loading = false;
    }
  });

  function getStatusBadge(status: string) {
    const map: Record<string, string> = {
      draft: 'info',
      released: 'success',
      obsolete: 'error',
      pending: 'warning',
      approved: 'success',
      rejected: 'error',
      queued: 'info',
      processing: 'warning',
      completed: 'success',
      failed: 'error',
    };
    return map[status] || 'info';
  }
</script>

<div class="dashboard">
  <header class="page-header">
    <div>
      <h2>Панель управления</h2>
      <p class="subtitle">Обзор системы Штурман PLM</p>
    </div>
  </header>

  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Загрузка данных...</p>
    </div>
  {:else}
    <div class="stats-grid">
      <div class="stat-card card">
        <div class="stat-icon">⚙️</div>
        <div class="stat-content">
          <div class="stat-value">{stats.partsCount}</div>
          <div class="stat-label">Части</div>
        </div>
      </div>

      <div class="stat-card card">
        <div class="stat-icon">📋</div>
        <div class="stat-content">
          <div class="stat-value">{stats.bomsCount}</div>
          <div class="stat-label">Спецификации</div>
        </div>
      </div>

      <div class="stat-card card">
        <div class="stat-icon">🔄</div>
        <div class="stat-content">
          <div class="stat-value">{stats.changesCount}</div>
          <div class="stat-label">Изменения</div>
        </div>
      </div>

      <div class="stat-card card">
        <div class="stat-icon">🤖</div>
        <div class="stat-content">
          <div class="stat-value">{stats.aiTasksCount}</div>
          <div class="stat-label">AI Задачи</div>
        </div>
      </div>
    </div>

    <div class="content-grid">
      <section class="card">
        <h3>Последние части</h3>
        {#if recentParts.length > 0}
          <div class="list">
            {#each recentParts as part}
              <div class="list-item">
                <div class="item-main">
                  <a href="/parts/{part.id}" class="item-title">{part.name}</a>
                  <div class="item-meta">
                    <span class="part-number">{part.part_number}</span>
                    <span class="separator">•</span>
                    <span class="part-type">{part.type}</span>
                  </div>
                </div>
                <span class="badge {getStatusBadge(part.status)}">{part.status}</span>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">Нет частей</p>
        {/if}
        <a href="/parts" class="view-all">Все части →</a>
      </section>

      <section class="card">
        <h3>Изменения (ECO)</h3>
        {#if recentChanges.length > 0}
          <div class="list">
            {#each recentChanges as change}
              <div class="list-item">
                <div class="item-main">
                  <a href="/changes/{change.id}" class="item-title">{change.title}</a>
                  <div class="item-meta">
                    <span class="change-number">{change.number}</span>
                    <span class="separator">•</span>
                    <span class="change-type">{change.type}</span>
                  </div>
                </div>
                <span class="badge {getStatusBadge(change.status)}">{change.status}</span>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">Нет изменений</p>
        {/if}
        <a href="/changes" class="view-all">Все изменения →</a>
      </section>

      {#if activeAITasks.length > 0}
        <section class="card full-width">
          <h3>Активные AI задачи</h3>
          <div class="list">
            {#each activeAITasks as task}
              <div class="list-item">
                <div class="item-main">
                  <div class="item-title">{task.task_type.replace('_', ' ')}</div>
                  <div class="item-meta">
                    <span>ID: {task.id.slice(0, 8)}...</span>
                    <span class="separator">•</span>
                    <span>Приоритет: {task.priority}</span>
                  </div>
                </div>
                <span class="badge {getStatusBadge(task.status)}">{task.status}</span>
              </div>
            {/each}
          </div>
        </section>
      {/if}
    </div>
  {/if}
</div>

<style>
  .dashboard {
    padding: var(--spacing-xl);
    max-width: 1600px;
    margin: 0 auto;
  }

  .page-header {
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
    justify-content: center;
    gap: var(--spacing-lg);
    padding: var(--spacing-xl);
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

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-xl);
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
    padding: var(--spacing-lg);
  }

  .stat-icon {
    font-size: 40px;
    opacity: 0.8;
  }

  .stat-value {
    font-size: clamp(28px, 4vw, 36px);
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--accent);
  }

  .stat-label {
    font-size: 14px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 600;
  }

  .content-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: var(--spacing-lg);
  }

  .full-width {
    grid-column: 1 / -1;
  }

  section h3 {
    font-size: 18px;
    margin-bottom: var(--spacing-lg);
    color: var(--text-primary);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .list-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--surface-2);
    border-radius: var(--radius-md);
    transition: background 0.2s;
  }

  .list-item:hover {
    background: var(--surface-3);
  }

  .item-main {
    flex: 1;
    min-width: 0;
  }

  .item-title {
    display: block;
    font-weight: 600;
    color: var(--text-primary);
    text-decoration: none;
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-title:hover {
    color: var(--accent);
  }

  .item-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .separator {
    opacity: 0.5;
  }

  .empty-state {
    padding: var(--spacing-xl);
    text-align: center;
    color: var(--text-tertiary);
  }

  .view-all {
    display: inline-block;
    color: var(--accent);
    text-decoration: none;
    font-weight: 500;
    font-size: 14px;
  }

  .view-all:hover {
    text-decoration: underline;
  }
</style>
