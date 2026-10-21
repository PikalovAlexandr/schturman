# Штурман PLM - Руководство по запуску

## Быстрый старт

### 1. Запуск backend сервисов

#### Вариант A: Скрипт (рекомендуется для разработки)
```bash
cd /mnt/storage/Штурман
./start-services.sh
```

Проверка:
```bash
# Все порты должны отвечать
curl -s http://localhost:3001/health  # Auth service
curl -s http://localhost:3002/health  # Part service
curl -s http://localhost:3003/health  # Document service
curl -s http://localhost:3004/health  # BOM service
curl -s http://localhost:3005/health  # Change service
curl -s http://localhost:3006/health  # AI orchestrator
```

#### Вариант B: Systemd (автозапуск)
```bash
systemctl --user start schturman-backend
systemctl --user status schturman-backend
```

Логи:
```bash
tail -f /tmp/auth-service.log
tail -f /tmp/part-service.log
# ... и так далее
```

### 2. Запуск frontend

#### Вариант A: Vite dev server (web UI)
```bash
cd /mnt/storage/Штурман/frontend
npm run dev
# Открыть http://localhost:5173
```

Или через systemd:
```bash
systemctl --user start schturman-frontend
# Откроется на http://localhost:5173
```

#### Вариант B: Tauri (desktop app)
```bash
cd /mnt/storage/Штурман/frontend
npm run tauri dev
# Откроется нативное окно приложения
```

### 3. Тестовые данные

**Пользователи** (пароль для всех: `test1234`):
- `tester` (id: `eb649f37-843d-4a53-9854-fb1aac65cd4d`)
- `testuser2`
- `approver`

**Детали**:
- `BLT-M6-20-SS` - болт M6x20
- `NUT-M6-SS` - гайка M6
- `WSHR-M6` - шайба M6
- `ASM-001` - сборка (тестовая)

**Документ**:
- `Test Bolt CAD` (id: `907c51dd-06e6-44a6-8b18-cbb3ad7b4dc5`)

**BOM**:
- Test BOM (id: `a1a76c99-52b8-44ad-a52c-6b9b2968993e`)

**ECO**:
- `ECO-001` (id: `b1a76c99-52b8-44ad-a52c-6b9b2968993e`)

## Остановка сервисов

```bash
# Backend
pkill -f 'schturman-auth-service|part-service|document-service|bom-service|change-service|ai-orchestrator'

# Frontend
systemctl --user stop schturman-frontend

# Или все сразу через systemd
systemctl --user stop schturman-backend schturman-frontend
```

## Архитектура сервисов

```
┌──────────────────────────────────────────┐
│  Frontend (SvelteKit + Tauri)            │
│  Port: 5173 (dev) / 1420 (Tauri)         │
└────────────────┬─────────────────────────┘
                 │ HTTP REST API
    ┌────────────┴─────────────────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
    │                                      │              │              │              │              │
┌───▼────┐  ┌───────┐  ┌──────────┐  ┌───▼─┐  ┌────────▼──┐  ┌────────▼──┐  ┌────────▼───┐
│ Auth   │  │ Part  │  │ Document │  │ BOM │  │  Change   │  │  AI        │
│ :3001  │  │ :3002 │  │  :3003   │  │:3004│  │  :3005    │  │Orchestrator│
│        │  │       │  │          │  │     │  │           │  │  :3006     │
└───┬────┘  └───┬───┘  └────┬─────┘  └──┬──┘  └─────┬─────┘  └─────┬──────┘
    │           │            │           │           │              │
    └───────────┴────────────┴───────────┴───────────┴──────────────┘
                                    │
                        ┌───────────┴───────────┐
                        │                       │
                  ┌─────▼──────┐          ┌────▼────┐
                  │ PostgreSQL │          │  Redis  │
                  │   :5434    │          │  :6379  │
                  └────────────┘          └─────────┘
```

## Конфигурация

### Переменные окружения (в `start-services.sh`)
```bash
DATABASE_URL=postgresql://schturman:schturman_password@localhost:5434/schturman
REDIS_URL=redis://localhost:6379
JWT_SECRET=test_secret_key_12345
JWT_REFRESH_SECRET=test_refresh_secret_67890
STORAGE_PATH=/tmp/schturman-documents
```

**ВАЖНО**: Для продакшена смените секреты на надёжные!

### Порты
- 3001: Auth service
- 3002: Part service
- 3003: Document service
- 3004: BOM service
- 3005: Change service
- 3006: AI orchestrator
- 5173: Frontend (dev Vite)
- 1420: Frontend (Tauri dev)
- 5434: PostgreSQL
- 6379: Redis

## Сборка production

### Backend
```bash
cd /mnt/storage/Штурман
cargo build --release
# Бинарники в target/release/
```

### Frontend (desktop app)
```bash
cd frontend
npm run tauri build
# Установщики в src-tauri/target/release/bundle/
# - .deb (Debian/Ubuntu)
# - .AppImage (универсальный Linux)
# - .rpm (Red Hat/Fedora)
```

## Диагностика

### Проверка портов
```bash
for port in 3001 3002 3003 3004 3005 3006 5173; do
  nc -z localhost $port && echo "✓ $port" || echo "✗ $port"
done
```

### Проверка логов
```bash
tail -f /tmp/auth-service.log
tail -f /tmp/part-service.log
tail -f /tmp/schturman-frontend.log
```

### Проверка API
```bash
# Parts list
curl http://localhost:5173/api/parts | jq

# Health check
curl http://localhost:3002/health
```

### Типичные проблемы

**Frontend не подключается к API**:
- Проверьте, что все backend сервисы запущены (`./start-services.sh`)
- Проверьте Vite proxy в `frontend/vite.config.js`

**Auth service падает**:
- Проверьте, что установлены `JWT_SECRET` и `JWT_REFRESH_SECRET`
- Проверьте подключение к PostgreSQL и Redis

**Порт занят**:
```bash
# Узнать, что занимает порт
lsof -i :3001

# Убить процесс
kill -9 <PID>
```

**База данных недоступна**:
```bash
# Проверить, что PostgreSQL работает
psql -h localhost -p 5434 -U schturman -d schturman

# Запустить миграции вручную
cd services/auth-service
sqlx migrate run --database-url "postgresql://schturman:schturman_password@localhost:5434/schturman"
```

## Разработка

### Структура проекта
```
/mnt/storage/Штурман/
├── services/          # Microservices (Rust)
│   ├── auth-service/
│   ├── part-service/
│   ├── document-service/
│   ├── bom-service/
│   ├── change-service/
│   └── ai-orchestrator/
├── ai-agents/         # AI agents (Rust)
│   ├── semantic-search/
│   ├── cad-parser/
│   ├── bom-validator/
│   ├── change-impact/
│   ├── document-extractor/
│   └── llm-summary/
├── frontend/          # Frontend (SvelteKit + Tauri)
├── shared/            # Shared Rust crates
│   └── models/
├── migrations/        # Database migrations
├── docs/              # Documentation
└── target/release/    # Compiled binaries
```

### Добавление нового сервиса
1. Создать в `services/new-service/`
2. Добавить в `Cargo.toml` workspace
3. Реализовать Axum/tonic сервер
4. Добавить миграции в `migrations/`
5. Обновить `start-services.sh`
6. Обновить API client в `frontend/src/lib/api.ts`

### Добавление нового AI агента
1. Создать в `ai-agents/new-agent/`
2. Зарегистрировать в AI orchestrator (`services/ai-orchestrator/src/registry.rs`)
3. Добавить task type в frontend (`routes/ai/+page.svelte`)

## Документация

- [Phase 0: Infrastructure](docs/phase0-infrastructure.md)
- [Phase 1: Core Backend](docs/phase1-backend.md)
- [Phase 2: AI Integration](docs/phase2-ai.md)
- [Phase 4: Frontend](docs/phase4-frontend.md)
- [API Reference](docs/api-reference.md)
- [Development Plan](../AGENTS.md)

## Поддержка

- **Issues**: https://github.com/PikalovAlexandr/schturman/issues
- **Email**: [your-email]
- **Wiki**: [project wiki URL]

---

**Статус**: ✅ MVP готов (Phase 4 завершена)
- Все 6 backend сервисов работают
- Frontend с полным UI
- AI orchestrator и 6 агентов
- Готово к разработке новых фич
