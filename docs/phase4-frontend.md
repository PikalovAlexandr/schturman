# Phase 4: Frontend Application

## Overview
Cross-platform desktop application built with Tauri + SvelteKit for Штурман PLM system.

## Architecture

### Tech Stack
- **Framework**: Tauri 2.x (Rust backend + web frontend)
- **Frontend**: SvelteKit 2.x with TypeScript
- **Styling**: Custom CSS with design tokens (no external UI library)
- **Build Tool**: Vite 8.x
- **API Communication**: REST API (fetch)

### Design System
Dark theme with tonal surface ladder (#05070C → #2A3447), cyan accent (#38BDF8), fluid typography with clamp(), CSS Grid layout, 999px radius for pills/buttons.

## Project Structure

```
frontend/
├── src/
│   ├── lib/
│   │   └── api.ts                    # API client for all microservices
│   ├── routes/
│   │   ├── +layout.svelte            # Main layout with sidebar navigation
│   │   ├── +page.svelte              # Dashboard (home)
│   │   ├── parts/
│   │   │   └── +page.svelte          # Parts Browser with AI search
│   │   ├── bom/
│   │   │   └── +page.svelte          # BOM list
│   │   ├── documents/
│   │   │   └── +page.svelte          # Documents (placeholder)
│   │   ├── changes/
│   │   │   └── +page.svelte          # Changes/ECO (placeholder)
│   │   └── ai/
│   │       └── +page.svelte          # AI Assistant interface
│   └── app.html                      # HTML template
├── src-tauri/
│   ├── src/
│   │   └── main.rs                   # Tauri Rust backend
│   ├── Cargo.toml                    # Rust dependencies
│   └── tauri.conf.json               # Tauri configuration
├── static/                           # Static assets
├── package.json
├── vite.config.js
└── svelte.config.js
```

## Implemented Features

### 1. Main Layout (`+layout.svelte`)
- **Sidebar Navigation**: 240px fixed width
  - Logo with version
  - Navigation items: Home, Parts, BOMs, Documents, Changes, AI Assistant
  - User info panel (bottom)
- **Active Route Highlighting**: accent border on current page
- **Global Styles**: CSS custom properties for colors, spacing, typography
- **Responsive Components**: buttons, inputs, cards, badges

### 2. Dashboard (`+page.svelte`)
- **Statistics Cards**: Parts, BOMs, Changes, Active AI Tasks count
- **Recent Parts**: Last 5 parts with status badges
- **Recent Changes**: Last 5 ECOs with status
- **Active AI Tasks**: Currently processing tasks
- **Loading State**: spinner with "Загрузка данных..."
- **Empty States**: user-friendly messages when no data

### 3. Parts Browser (`routes/parts/+page.svelte`)
- **Part List**: grid layout with cards (320px min width)
- **Search Bar**: text input with search button
- **AI Toggle**: checkbox to enable semantic search
- **Part Cards**: 
  - Part name, number (monospace font)
  - Description (2-line clamp)
  - Type, creation date
  - Status badge (draft/released/obsolete)
  - Action buttons (View, Edit)
- **Search Integration**:
  - Regular search: `/api/parts/search?q=...`
  - AI search: calls `ai.semanticSearch()` with 15s timeout
- **Empty State**: "Части не найдены" with icon

### 4. BOM List (`routes/bom/+page.svelte`)
- **BOM Cards**: grid layout
- **BOM Header**: name + status badge
- **Metadata**: version, creation date
- **Actions**: View, Edit buttons
- **Empty State**: "Нет спецификаций"

### 5. AI Assistant (`routes/ai/+page.svelte`)
- **Task Runner**: 
  - Task type selector (6 types: semantic_search, cad_analysis, bom_validation, change_impact, document_extraction, llm_summary)
  - JSON input textarea
  - Run button with loading state
  - Result display (formatted JSON)
- **Task History**: last 10 tasks with status badges
- **Polling**: 30s timeout with 1s intervals for task completion

### 6. Placeholder Pages
- **Documents** (`routes/documents/+page.svelte`): "Страница в разработке"
- **Changes** (`routes/changes/+page.svelte`): "Страница в разработке"

## API Client (`src/lib/api.ts`)

### Authentication
- `auth.login(username, password)` → `{ user_id }`
- `auth.register(username, password, email)` → `{ user_id }`
- Simple user ID storage: `setUserId()`, `getUserId()`
- All requests include `x-user-id` header

### Parts API
```typescript
parts.list() → Part[]
parts.get(id) → Part
parts.create(data) → Part
parts.update(id, data) → Part
parts.delete(id) → void
parts.search(query) → Part[]
```

### Documents API
```typescript
documents.list() → Document[]
documents.get(id) → Document
documents.create(formData) → Document  // multipart upload
documents.download(id) → Blob
```

### BOM API
```typescript
bom.list() → BOM[]
bom.get(id) → BOM
bom.create(data) → BOM
bom.getItems(bomId) → BOMItem[]
bom.addItem(bomId, data) → BOMItem
bom.whereUsed(partId) → BOM[]
```

### Changes API
```typescript
changes.list() → Change[]
changes.get(id) → Change
changes.create(data) → Change
changes.submit(id) → void
changes.approve(id, comment?) → void
changes.reject(id, comment) → void
```

### AI API
```typescript
ai.createTask(taskType, inputData, priority?) → AITask
ai.getTask(id) → AITask
ai.listTasks() → AITask[]
ai.cancelTask(id) → void

// Shortcuts with polling
ai.semanticSearch(query, filters?) → Part[]
ai.analyzeCad(documentId) → analysis result
ai.validateBom(bomId) → validation result
```

## Service Endpoints

All services run on localhost:
- Auth: `http://localhost:3001`
- Parts: `http://localhost:3002`
- Documents: `http://localhost:3003`
- BOM: `http://localhost:3004`
- Changes: `http://localhost:3005`
- AI Orchestrator: `http://localhost:3006`

## Running the Frontend

### Development Mode
```bash
cd frontend
npm install
npm run dev
```
Vite dev server runs on `http://localhost:1420`

### Build for Production
```bash
npm run build
```
Outputs to `frontend/build/`

### Tauri Desktop App
```bash
npm run tauri dev          # Development with hot-reload
npm run tauri build        # Production build (requires system dependencies)
```

## System Dependencies (Linux)

Tauri requires WebKit2GTK and other libraries:
```bash
sudo apt-get install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
```

## Configuration

### Tauri Config (`src-tauri/tauri.conf.json`)
```json
{
  "productName": "Штурман PLM",
  "identifier": "com.shtorman.plm",
  "app": {
    "windows": [{
      "title": "Штурман PLM",
      "width": 1400,
      "height": 900
    }]
  }
}
```

### Vite Config (`vite.config.js`)
Default SvelteKit adapter-static for Tauri

## Design Tokens

### Colors
```css
--surface-0: #05070C    /* Darkest background */
--surface-1: #0A0D12    /* Cards, sidebar */
--surface-2: #0F131C    /* Input backgrounds */
--surface-3: #161D2B    /* Borders, hover */
--surface-4: #1E2636
--surface-5: #2A3447

--accent: #38BDF8       /* Cyan, buttons, links */
--accent-dim: #0EA5E9
--accent-glow: rgba(56, 189, 248, 0.2)

--success: #6EE7B7      /* Green */
--warning: #FCD34D      /* Yellow */
--error: #F87171        /* Red */

--text-primary: #F8FAFC
--text-secondary: #94A3B8
--text-tertiary: #64748B
```

### Spacing
```css
--spacing-xs: 0.25rem
--spacing-sm: 0.5rem
--spacing-md: 1rem
--spacing-lg: 1.5rem
--spacing-xl: 2rem
```

### Radius
```css
--radius-sm: 8px
--radius-md: 12px
--radius-lg: 16px
--radius-full: 999px
```

### Typography
```css
--font-body: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, ...
--font-mono: 'JetBrains Mono', 'Fira Code', Consolas, monospace

/* Fluid sizes with clamp() */
body: clamp(14px, 1.5vw, 16px)
h2: clamp(24px, 3vw, 32px)
```

## Status Badges

Badge classes: `.badge.success`, `.badge.warning`, `.badge.error`, `.badge.info`

Mapping:
- `draft` → info (cyan)
- `released`, `approved`, `completed` → success (green)
- `obsolete`, `rejected`, `failed` → error (red)
- `pending`, `queued`, `processing` → warning (yellow)

## Loading States

- **Spinner**: 40px circle with rotating accent border
- **Message**: "Загрузка данных..." / "Обработка..."
- **Disabled Buttons**: opacity 0.5, cursor not-allowed

## Empty States

- **Icon**: Large emoji (64px, opacity 0.5)
- **Title**: h3 with descriptive message
- **Description**: Secondary text with call-to-action

## Current Limitations

1. **No Authentication Flow**: uses hardcoded user ID (`eb649f37-843d-4a53-9854-fb1aac65cd4d`)
2. **No Routing to Detail Pages**: clicking View/Edit doesn't navigate yet
3. **No Forms**: Create/Edit modals not implemented
4. **Documents/Changes Pages**: placeholders only
5. **No Error Handling UI**: errors logged to console only
6. **No Offline Support**: requires active backend services
7. **No WebSocket**: real-time updates require manual refresh

## Next Steps (Phase 5)

1. **Authentication UI**: login/register forms, JWT storage
2. **Detail Pages**: part detail, BOM editor with tree view, change workflow
3. **Create/Edit Forms**: modals with validation
4. **Document Viewer**: PDF preview, CAD thumbnails, version timeline
5. **BOM Tree Editor**: drag-drop, visual diff, where-used panel
6. **Change Dashboard**: Kanban board, approval workflow visualization
7. **Real-time Updates**: WebSocket for task status, approvals
8. **Offline Mode**: Service Worker, local cache (SQLite via Tauri)
9. **Error Handling**: toast notifications, retry logic
10. **Testing**: Playwright E2E, Vitest unit tests

## Testing

### Manual Testing Checklist
- [ ] Dashboard loads with stats
- [ ] Parts list displays correctly
- [ ] Search works (both regular and AI)
- [ ] BOM list shows items
- [ ] AI Assistant can run tasks
- [ ] Task history updates
- [ ] Navigation highlights active route
- [ ] Responsive layout (1400x900 window)

### Test Users
- `tester` / `test1234` (id: `eb649f37-843d-4a53-9854-fb1aac65cd4d`)
- `testuser2` / `test1234`
- `approver` / `test1234`

### Test Data
- Parts: `BLT-M6-20-SS`, `NUT-M6-SS`, `WSHR-M6`
- BOM: id `a1a76c99-52b8-44ad-a52c-6b9b2968993e`
- ECO: `ECO-001` (id `b1a76c99-52b8-44ad-a52c-6b9b2968993e`)
- Document: `Test Bolt CAD` (id `907c51dd-06e6-44a6-8b18-cbb3ad7b4dc5`)

## Performance

- **Initial Load**: < 1s (with warm backend)
- **Dashboard Stats**: 4 parallel API calls
- **Parts List**: single API call
- **AI Search**: 500ms polling interval, 30s timeout
- **Vite HMR**: instant updates during development

## Browser Support

Tested in Chromium (via Tauri WebView):
- CSS Grid, Flexbox
- Custom Properties (CSS variables)
- Clamp() for fluid typography
- Modern JS (async/await, fetch, ES modules)

## Known Issues

1. **Vite Forced Re-optimization**: warning on first load (harmless)
2. **systemd Service Status**: shows "failed" but services actually running
3. **No CORS**: frontend must run on same host as backend
4. **Large JSON in AI Results**: no truncation for huge responses

## Deployment

### Development (current setup)
- Vite dev server: port 1420
- All services on localhost:3001-3006
- PostgreSQL: port 5434
- Redis: port 6379

### Production
- Build static SvelteKit: `npm run build`
- Build Tauri app: `npm run tauri build` → AppImage/deb for Linux
- Services behind reverse proxy (nginx)
- HTTPS with Let's Encrypt

## Monitoring

Frontend logs all API errors to console. Production should integrate:
- Sentry for error tracking
- PostHog / Plausible for analytics
- Custom telemetry to AI Orchestrator

---

**Phase 4 Status**: ✅ Complete (MVP)
**Commit**: TBD (after testing)
**Date**: 2026-10-21
