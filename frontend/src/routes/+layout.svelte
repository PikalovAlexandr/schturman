<script lang="ts">
  import { page } from '$app/stores';
  
  const navItems = [
    { href: '/', label: 'Главная', icon: '🏠' },
    { href: '/parts', label: 'Части', icon: '⚙️' },
    { href: '/bom', label: 'Спецификации', icon: '📋' },
    { href: '/documents', label: 'Документы', icon: '📄' },
    { href: '/changes', label: 'Изменения', icon: '🔄' },
    { href: '/ai', label: 'AI Ассистент', icon: '🤖' },
  ];
</script>

<div class="app-layout">
  <nav class="sidebar">
    <div class="logo">
      <h1>Штурман PLM</h1>
      <span class="version">v0.1.0</span>
    </div>
    
    <ul class="nav-list">
      {#each navItems as item}
        <li>
          <a 
            href={item.href} 
            class:active={$page.url.pathname === item.href}
            data-sveltekit-preload-data
          >
            <span class="icon">{item.icon}</span>
            <span class="label">{item.label}</span>
          </a>
        </li>
      {/each}
    </ul>
    
    <div class="user-info">
      <div class="avatar">👤</div>
      <div class="details">
        <span class="name">Test User</span>
        <span class="role">Engineer</span>
      </div>
    </div>
  </nav>
  
  <main class="content">
    <slot />
  </main>
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(:root) {
    --surface-0: #05070C;
    --surface-1: #0A0D12;
    --surface-2: #0F131C;
    --surface-3: #161D2B;
    --surface-4: #1E2636;
    --surface-5: #2A3447;
    
    --accent: #38BDF8;
    --accent-dim: #0EA5E9;
    --accent-glow: rgba(56, 189, 248, 0.2);
    
    --text-primary: #F8FAFC;
    --text-secondary: #94A3B8;
    --text-tertiary: #64748B;
    
    --success: #6EE7B7;
    --warning: #FCD34D;
    --error: #F87171;
    
    --radius-sm: 8px;
    --radius-md: 12px;
    --radius-lg: 16px;
    --radius-full: 999px;
    
    --spacing-xs: 0.25rem;
    --spacing-sm: 0.5rem;
    --spacing-md: 1rem;
    --spacing-lg: 1.5rem;
    --spacing-xl: 2rem;
    
    --font-body: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    --font-mono: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, monospace;
  }

  :global(body) {
    font-family: var(--font-body);
    background: var(--surface-0);
    color: var(--text-primary);
    line-height: 1.6;
    font-size: clamp(14px, 1.5vw, 16px);
  }

  .app-layout {
    display: grid;
    grid-template-columns: 240px 1fr;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    background: var(--surface-1);
    border-right: 1px solid var(--surface-3);
    display: flex;
    flex-direction: column;
    padding: var(--spacing-lg);
  }

  .logo {
    margin-bottom: var(--spacing-xl);
  }

  .logo h1 {
    font-size: clamp(20px, 2vw, 24px);
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin-bottom: var(--spacing-xs);
  }

  .logo .version {
    font-size: 12px;
    color: var(--text-tertiary);
    font-family: var(--font-mono);
  }

  .nav-list {
    list-style: none;
    flex: 1;
  }

  .nav-list li {
    margin-bottom: var(--spacing-sm);
  }

  .nav-list a {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    border-radius: var(--radius-md);
    text-decoration: none;
    color: var(--text-secondary);
    transition: all 0.2s;
  }

  .nav-list a:hover {
    background: var(--surface-2);
    color: var(--text-primary);
  }

  .nav-list a.active {
    background: var(--surface-3);
    color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent-glow);
  }

  .nav-list .icon {
    font-size: 20px;
  }

  .nav-list .label {
    font-weight: 500;
    font-size: 14px;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--surface-2);
    border-radius: var(--radius-md);
    margin-top: var(--spacing-lg);
  }

  .avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--surface-4);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
  }

  .details {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .details .name {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
  }

  .details .role {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .content {
    overflow-y: auto;
    background: var(--surface-0);
  }

  :global(h2, h3, h4) {
    font-weight: 600;
    letter-spacing: -0.01em;
    line-height: 1.2;
  }

  :global(button) {
    font-family: var(--font-body);
    font-size: 14px;
    font-weight: 500;
    padding: var(--spacing-sm) var(--spacing-lg);
    border-radius: var(--radius-full);
    border: none;
    cursor: pointer;
    transition: all 0.2s;
  }

  :global(button.primary) {
    background: var(--accent);
    color: var(--surface-0);
  }

  :global(button.primary:hover) {
    background: var(--accent-dim);
    box-shadow: 0 0 16px var(--accent-glow);
  }

  :global(button.secondary) {
    background: var(--surface-3);
    color: var(--text-primary);
  }

  :global(button.secondary:hover) {
    background: var(--surface-4);
  }

  :global(input, textarea, select) {
    font-family: var(--font-body);
    font-size: 14px;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--surface-2);
    border: 1px solid var(--surface-3);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    transition: all 0.2s;
  }

  :global(input:focus, textarea:focus, select:focus) {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  :global(.card) {
    background: var(--surface-1);
    border: 1px solid var(--surface-3);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
  }

  :global(.badge) {
    display: inline-flex;
    align-items: center;
    padding: 4px 12px;
    border-radius: var(--radius-full);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  :global(.badge.success) {
    background: rgba(110, 231, 183, 0.15);
    color: var(--success);
  }

  :global(.badge.warning) {
    background: rgba(252, 211, 77, 0.15);
    color: var(--warning);
  }

  :global(.badge.error) {
    background: rgba(248, 113, 113, 0.15);
    color: var(--error);
  }

  :global(.badge.info) {
    background: var(--accent-glow);
    color: var(--accent);
  }
</style>
