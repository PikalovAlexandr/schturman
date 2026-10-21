# Phase 4: Frontend Application

## Overview
Cross-platform desktop application built with Tauri + SvelteKit for PLM system interaction.

## Technology Stack
- **Framework**: Tauri 2.3.0 (Rust backend) + SvelteKit 2.15.1 (web frontend)
- **Language**: TypeScript 5.7.3
- **UI**: Svelte 5.22.0 with SvelteKit
- **Build**: Vite 8.3.5
- **Styling**: Custom CSS with design system tokens

## Project Structure
```
frontend/
├── src/
│   ├── routes/           # SvelteKit pages
│   │   ├── +page.svelte         # Dashboard (home)
│   │   ├── +layout.svelte       # App shell with navigation
│   │   ├── parts/+page.svelte   # Part Browser
│   │   ├── bom/+page.svelte     # BOM Editor
│   │   ├── documents/+page.svelte  # Document Viewer
│   │   ├── changes/+page.svelte    # Change Management
│   │   └── ai/+page.svelte         # AI Assistant
│   ├── lib/
│   │   ├── api.ts        # Backend API client (fetch wrapper)
│   │   └── stores.ts     # Svelte stores for global state
│   └── app.css           # Global styles
├── src-tauri/            # Tauri Rust backend
│   ├── src/main.rs       # Tauri entry point
│   ├── Cargo.toml        # Rust dependencies
│   └── tauri.conf.json   # Tauri configuration
├── package.json          # Node dependencies
├── vite.config.js        # Vite build config
└── svelte.config.js      # SvelteKit config
```

## Running the Application

### Development Mode (Web UI only)
```bash
cd frontend
npm run dev
# Access at http://localhost:5173
```

### Development Mode (Tauri desktop app)
```bash
cd frontend
npm run tauri dev
# Opens native window on port 1420
```

### Production Build
```bash
cd frontend
npm run tauri build
# Creates installers in src-tauri/target/release/bundle/
```

### Systemd Service (dev web server)
```bash
systemctl --user start schturman-frontend
systemctl --user status schturman-frontend
# Logs: /tmp/schturman-frontend.log
```

## API Client (`src/lib/api.ts`)

### Configuration
```typescript
const API_BASES = {
  auth: 'http://localhost:3001',
  parts: 'http://localhost:3002',
  documents: 'http://localhost:3003',
  bom: 'http://localhost:3004',
  changes: 'http://localhost:3005',
  ai: 'http://localhost:3006',
};
```

### Authentication
```typescript
import { auth } from '$lib/api';

// Login
const { token, user } = await auth.login('username', 'password');

// Register
const { token, user } = await auth.register('username', 'password');

// Logout
await auth.logout();
```

### Parts API
```typescript
import { parts } from '$lib/api';

// List all parts (paginated)
const partsList = await parts.list();  // returns Part[]

// Get single part
const part = await parts.get('part-id-uuid');

// Create part
const newPart = await parts.create({
  number: 'BLT-M6-20',
  name: 'M6x20 Bolt',
  description: 'Stainless steel A2',
  part_type: 'hardware',
  status: 'draft',
  version: '1.0'
});

// Update part
const updated = await parts.update('part-id', {
  name: 'Updated name',
  status: 'released'
});

// Delete part
await parts.delete('part-id');
```

### Documents API
```typescript
import { documents } from '$lib/api';

// List documents
const docs = await documents.list();

// Upload document
const formData = new FormData();
formData.append('file', file);
formData.append('name', 'Drawing.pdf');
formData.append('format', 'pdf');
const doc = await documents.upload(formData);

// Download document
const blob = await documents.download('doc-id');

// Get versions
const versions = await documents.getVersions('doc-id');

// Check out/in
await documents.checkout('doc-id');
await documents.checkin('doc-id', 'Release notes');
```

### BOM API
```typescript
import { bom } from '$lib/api';

// List BOMs
const boms = await bom.list();

// Get BOM with items
const bomData = await bom.get('bom-id');

// Create BOM
const newBom = await bom.create({
  name: 'Main Assembly',
  parent_part_id: 'part-id',
  version: '1.0'
});

// Add item to BOM
await bom.addItem('bom-id', {
  part_id: 'part-id',
  position: 'A1',
  quantity: 5.0,
  notes: 'Per drawing'
});

// Update item
await bom.updateItem('bom-id', 'item-id', {
  quantity: 10.0
});

// Remove item
await bom.removeItem('bom-id', 'item-id');

// Compare BOMs
const diff = await bom.compare('bom-id-1', 'bom-id-2');
```

### Changes API
```typescript
import { changes } from '$lib/api';

// List changes
const changesList = await changes.list();

// Get change with affected items
const change = await changes.get('change-id');

// Create change
const eco = await changes.create({
  number: 'ECO-001',
  title: 'Material change',
  change_type: 'engineering',
  priority: 'high',
  reason: 'Cost reduction'
});

// Submit for approval
await changes.submitForApproval('change-id');

// Approve
await changes.approve('change-id', 'Approved by engineering');

// Reject
await changes.reject('change-id', 'Missing documentation');

// Implement
await changes.implement('change-id');
```

### AI API
```typescript
import { ai } from '$lib/api';

// Execute AI task
const task = await ai.executeTask({
  task_type: 'semantic_search',
  priority: 'normal',
  input_data: {
    query: 'M6 bolts stainless steel',
    filters: { status: 'released' },
    limit: 10
  }
});

// Get task status
const status = await ai.getTaskStatus('task-id');

// List all tasks
const tasks = await ai.listTasks({ status: 'completed' });

// Cancel task
await ai.cancelTask('task-id');
```

## UI Modules

### 1. Dashboard (`/`)
- System overview
- Recent changes
- Quick stats (parts count, documents, pending ECOs)
- Quick actions (create part, upload document, new ECO)

**Implementation**: `src/routes/+page.svelte`

### 2. Part Browser (`/parts`)
- Paginated part list with search
- AI-powered semantic search
- Filters: status, type, owner
- Quick view panel with part details
- CRUD operations

**Implementation**: `src/routes/parts/+page.svelte`

**Key Features**:
- Real-time search as you type
- Status badges (draft/released/obsolete)
- Inline edit mode
- Bulk operations (future)

### 3. BOM Editor (`/bom`)
- Tree view of assemblies and sub-assemblies
- Drag-drop reordering
- Add/remove items
- Where-used panel
- BOM comparison (diff viewer)

**Implementation**: `src/routes/bom/+page.svelte`

**Key Features**:
- Hierarchical tree with expand/collapse
- Quantity validation
- Duplicate detection (via AI)
- Export to CSV/Excel

### 4. Document Viewer (`/documents`)
- Document list with thumbnails
- Upload/download
- Version timeline
- Check-out/check-in workflow
- CAD preview (via AI-generated thumbnails)

**Implementation**: `src/routes/documents/+page.svelte`

**Key Features**:
- Multi-file upload
- Drag-drop support
- Format icons (PDF, STEP, IGES, DOCX)
- Version comparison

### 5. Change Management Dashboard (`/changes`)
- Kanban board (Draft → Pending → Approved → Implemented)
- ECO/ECR list with filters
- Approval workflow visualization
- Affected items tracking
- AI-suggested actions

**Implementation**: `src/routes/changes/+page.svelte`

**Key Features**:
- Drag-drop cards between columns
- Priority indicators
- Approval progress bar
- Impact analysis integration (AI)

### 6. AI Assistant Panel (`/ai`)
- Chat interface for natural language queries
- Task status monitoring
- Prompt history
- Available AI capabilities list
- Quick actions (semantic search, BOM validation, change impact)

**Implementation**: `src/routes/ai/+page.svelte`

**Key Features**:
- Real-time task progress
- Streaming results (future)
- Task history with re-run
- Agent status indicators

## Global State Management

### Stores (`src/lib/stores.ts`)
```typescript
import { writable } from 'svelte/store';

// Authentication
export const user = writable<User | null>(null);
export const token = writable<string | null>(null);

// UI state
export const sidebarOpen = writable(true);
export const currentView = writable('dashboard');

// Data cache
export const partsCache = writable<Part[]>([]);
export const documentsCache = writable<Document[]>([]);
```

**Usage in components**:
```svelte
<script lang="ts">
  import { user, token } from '$lib/stores';
  $: isLoggedIn = $user !== null;
</script>

{#if isLoggedIn}
  <p>Welcome, {$user.username}!</p>
{/if}
```

## Design System

### Color Palette (from `app.css`)
- **Surface levels**:
  - `#05070C` (deepest, 3% lightness)
  - `#0A0D12` (level 1)
  - `#0F131C` (level 2)
  - `#161D2B` (level 3)
  - `#1E2636` (level 4, lighter)
- **Accent**: `#3B6DFF` (blue, high saturation)
- **Supporting tint**: `#6EE7B7` (emerald for success states)
- **Text**:
  - Primary: `#E5E7EB` (off-white)
  - Secondary: `#9CA3AF` (muted gray)
  - Muted: `#6B7280` (dark gray)

### Typography
- **Font families**:
  - `--font-sans`: Inter, system-ui, sans-serif
  - `--font-mono`: JetBrains Mono, monospace
- **Sizes** (fluid with clamp()):
  - Display: `clamp(2rem, 5vw, 3rem)`
  - Heading: `clamp(1.5rem, 3vw, 2rem)`
  - Body: `clamp(0.875rem, 1vw, 1rem)`

### Layout
- **Grid-first**: CSS Grid for page structure, Flexbox for components
- **Spacing tokens**:
  - `--space-xs`: 0.25rem
  - `--space-sm`: 0.5rem
  - `--space-md`: 1rem
  - `--space-lg`: 1.5rem
  - `--space-xl`: 2rem
- **Border radius**:
  - `--radius-sm`: 0.375rem
  - `--radius-md`: 0.5rem
  - `--radius-lg`: 0.75rem
  - `--radius-full`: 999px (pills)

### Components
- **Buttons**: Rounded pills (`border-radius: 999px`), consistent padding
- **Cards**: Surface level 2 with shadow, rounded corners
- **Inputs**: Surface level 1, focus ring with accent color
- **Badges**: Small pills with status colors (green=released, yellow=draft, red=obsolete)

## Development Workflow

### Adding a New Page
1. Create `src/routes/mypage/+page.svelte`
2. Add navigation link in `src/routes/+layout.svelte`
3. Create API functions in `src/lib/api.ts` if needed
4. Add stores in `src/lib/stores.ts` for state

### Example: New Page
```svelte
<!-- src/routes/mypage/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { parts } from '$lib/api';
  import type { Part } from '$lib/api';

  let items: Part[] = [];
  let loading = true;

  onMount(async () => {
    try {
      items = await parts.list();
    } catch (err) {
      console.error('Failed to load:', err);
    } finally {
      loading = false;
    }
  });
</script>

<div class="container">
  <h1>My Page</h1>
  
  {#if loading}
    <p>Loading...</p>
  {:else}
    <ul>
      {#each items as item}
        <li>{item.name}</li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .container {
    padding: var(--space-lg);
  }
</style>
```

## Configuration

### Vite Proxy (for development)
In `vite.config.js`, API calls to `/api/*` are proxied to backend services:
```javascript
proxy: {
  '/api': {
    target: 'http://localhost:3002',  // part-service
    changeOrigin: true,
    configure: (proxy) => {
      proxy.on('proxyReq', (proxyReq) => {
        // Auto-inject test user header for dev
        proxyReq.setHeader('x-user-id', 'eb649f37-843d-4a53-9854-fb1aac65cd4d');
      });
    },
  },
}
```

**Note**: In production Tauri build, API calls go directly to configured endpoints (no proxy).

### Tauri Configuration
`src-tauri/tauri.conf.json`:
```json
{
  "productName": "Штурман PLM",
  "version": "0.1.0",
  "identifier": "com.schturman.plm",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../build"
  },
  "bundle": {
    "targets": ["deb", "appimage", "rpm"]
  }
}
```

## Testing

### Manual Testing Checklist
- [ ] Login/logout flow
- [ ] Part CRUD operations
- [ ] Document upload/download
- [ ] BOM tree view and editing
- [ ] ECO approval workflow
- [ ] AI task execution and monitoring
- [ ] Responsive layout on different window sizes
- [ ] Dark theme consistency

### Integration Tests (future)
- Playwright for end-to-end UI tests
- Vitest for unit tests of API client and stores

## Performance Considerations
- **Pagination**: All lists fetched with pagination (default 20 items)
- **Lazy loading**: Routes loaded on demand (SvelteKit code-splitting)
- **Caching**: Stores cache frequently accessed data (parts, documents)
- **Debouncing**: Search inputs debounced (300ms) to reduce API calls
- **Virtual scrolling** (future): For large BOM trees (1000+ items)

## Security
- **Authentication**: JWT tokens stored in localStorage
- **Authorization**: x-user-id header sent with every request
- **Input validation**: Client-side validation before API calls
- **HTTPS**: Production builds use HTTPS for API calls
- **CSP** (future): Content Security Policy headers in Tauri

## Deployment

### Desktop App (Tauri)
```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/
# - Debian package: .deb
# - AppImage: .AppImage
# - RPM: .rpm
```

### Web App (standalone)
```bash
npm run build
# Output: build/
# Serve with static file server (nginx, Caddy)
```

### Docker (web app)
```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/build /usr/share/nginx/html
EXPOSE 80
```

## Future Enhancements
- [ ] Real-time collaboration (WebSocket updates)
- [ ] Offline mode (IndexedDB cache)
- [ ] Mobile app (Tauri mobile or Flutter)
- [ ] Advanced BOM diff (visual comparison)
- [ ] 3D CAD viewer integration (Three.js)
- [ ] Gantt chart for change schedules
- [ ] User preferences (theme, language, layout)
- [ ] Keyboard shortcuts (Cmd+K command palette)
- [ ] Export reports (PDF, Excel)
- [ ] Multi-language support (i18n)

## Troubleshooting

### Vite won't start (port conflict)
```bash
# Kill process on port 5173
lsof -ti:5173 | xargs kill -9

# Or change port in vite.config.js
server: { port: 3000 }
```

### API calls fail (CORS)
- In development, use Vite proxy (already configured)
- In production, backend services must set CORS headers:
  ```rust
  .layer(CorsLayer::permissive())
  ```

### Tauri build fails
- Ensure Rust toolchain installed: `rustup update`
- Install system dependencies (Debian/Ubuntu):
  ```bash
  sudo apt install libwebkit2gtk-4.0-dev \
    build-essential curl wget file \
    libssl-dev libgtk-3-dev libayatana-appindicator3-dev \
    librsvg2-dev
  ```

### Dark theme not applied
- Check `app.css` is imported in `+layout.svelte`
- Clear browser cache (Ctrl+Shift+R)

## Resources
- [Tauri Docs](https://tauri.app/v2/)
- [SvelteKit Docs](https://kit.svelte.dev/)
- [Svelte 5 Runes](https://svelte-5-preview.vercel.app/docs/runes)
- [Vite Config](https://vitejs.dev/config/)

---

**Status**: ✅ Phase 4 frontend shell complete (commit e9703d9)
- All 6 pages created (dashboard, parts, BOM, documents, changes, AI)
- API client with full backend integration
- Navigation and layout working
- Vite proxy configured for development
- Type-safe API interfaces
- Ready for UI implementation and feature development
