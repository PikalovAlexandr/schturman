<script lang="ts">
  import { ai } from '$lib/api';
  import type { AITask } from '$lib/api';
  
  let tasks: AITask[] = [];
  let selectedTaskType = 'semantic_search';
  let inputText = '';
  let result = '';
  let processing = false;

  const taskTypes = [
    { value: 'semantic_search', label: 'Семантический поиск' },
    { value: 'cad_analysis', label: 'Анализ CAD' },
    { value: 'bom_validation', label: 'Проверка BOM' },
    { value: 'change_impact', label: 'Анализ влияния ECO' },
    { value: 'document_extraction', label: 'Извлечение из документов' },
    { value: 'llm_summary', label: 'Генерация резюме' },
  ];

  async function loadTasks() {
    try {
      tasks = await ai.listTasks();
    } catch (error) {
      console.error('Failed to load tasks:', error);
    }
  }

  async function runTask() {
    processing = true;
    result = '';
    try {
      const inputData = JSON.parse(inputText || '{}');
      const task = await ai.createTask(selectedTaskType, inputData, 8);
      
      // Poll for result
      for (let i = 0; i < 30; i++) {
        await new Promise(r => setTimeout(r, 1000));
        const updated = await ai.getTask(task.id);
        if (updated.status === 'completed') {
          result = JSON.stringify(updated.result_data, null, 2);
          break;
        }
        if (updated.status === 'failed') {
          result = `Error: ${updated.error}`;
          break;
        }
      }
      
      await loadTasks();
    } catch (error: any) {
      result = `Error: ${error.message}`;
    } finally {
      processing = false;
    }
  }

  loadTasks();
</script>

<div class="ai-page">
  <header class="page-header">
    <div>
      <h2>AI Ассистент</h2>
      <p class="subtitle">Интеллектуальные возможности PLM</p>
    </div>
  </header>

  <div class="ai-grid">
    <section class="card">
      <h3>Запустить AI задачу</h3>
      
      <div class="form">
        <div class="form-group">
          <label>Тип задачи</label>
          <select bind:value={selectedTaskType}>
            {#each taskTypes as type}
              <option value={type.value}>{type.label}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label>Входные данные (JSON)</label>
          <textarea
            bind:value={inputText}
            placeholder='&#123;"query": "болт M6 нержавеющая сталь"&#125;'
            rows="4"
          ></textarea>
        </div>

        <button 
          class="primary" 
          on:click={runTask} 
          disabled={processing}
        >
          {processing ? '⏳ Обработка...' : '▶️ Запустить'}
        </button>

        {#if result}
          <div class="result">
            <h4>Результат:</h4>
            <pre>{result}</pre>
          </div>
        {/if}
      </div>
    </section>

    <section class="card">
      <h3>История задач</h3>
      
      {#if tasks.length === 0}
        <p class="empty">Нет задач</p>
      {:else}
        <div class="task-list">
          {#each tasks.slice(0, 10) as task}
            <div class="task-item">
              <div>
                <div class="task-type">{task.task_type}</div>
                <div class="task-meta">
                  {new Date(task.created_at).toLocaleString('ru-RU')}
                </div>
              </div>
              <span class="badge {task.status === 'completed' ? 'success' : task.status === 'failed' ? 'error' : 'warning'}">
                {task.status}
              </span>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  .ai-page {
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

  .ai-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-lg);
  }

  h3 {
    margin-bottom: var(--spacing-lg);
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .form-group label {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-secondary);
  }

  .result {
    padding: var(--spacing-md);
    background: var(--surface-2);
    border-radius: var(--radius-md);
    border: 1px solid var(--surface-3);
  }

  .result h4 {
    margin-bottom: var(--spacing-sm);
    font-size: 14px;
    color: var(--text-secondary);
  }

  .result pre {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .empty {
    padding: var(--spacing-xl);
    text-align: center;
    color: var(--text-tertiary);
  }

  .task-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .task-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    background: var(--surface-2);
    border-radius: var(--radius-md);
  }

  .task-type {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
  }

  .task-meta {
    font-size: 12px;
    color: var(--text-tertiary);
    margin-top: 2px;
  }
</style>
