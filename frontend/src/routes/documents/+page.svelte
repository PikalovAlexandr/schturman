<script>
  let placeholder = true;
</script>

<div class="page">
  <header class="page-header">
    <div>
      <h2>Документы</h2>
      <p class="subtitle">CAD файлы, PDF, чертежи</p>
    </div>
    <button class="primary">+ Загрузить документ</button>
  </header>

  <div class="empty-state card">
    <div class="empty-icon">📄</div>
    <h3>Страница в разработке</h3>
    <p>Модуль управления документами будет доступен в следующей версии</p>
  </div>
</div>

<style>
  .page {
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

  .empty-state {
    text-align: center;
    padding: var(--spacing-xl) * 2;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: var(--spacing-lg);
    opacity: 0.5;
  }
</style>
