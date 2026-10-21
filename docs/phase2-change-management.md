# Phase 2: Change Management Service

## Обзор

Change-service реализует полный цикл управления изменениями (ECO/ECR/MCR) с workflow-процессом утверждения.

**Порт**: 3005  
**База данных**: PostgreSQL (порт 5434)  
**Аутентификация**: `x-user-id` header

---

## Типы изменений

- **ECO** (Engineering Change Order) — изменение конструкции
- **ECR** (Engineering Change Request) — запрос на изменение
- **MCR** (Manufacturing Change Request) — изменение процесса производства

---

## Workflow Status

```
draft → submitted → approved → implemented
                  ↘ rejected
                  ↘ cancelled
```

---

## API Endpoints

### Создание и управление ECO

#### Создать ECO
```bash
POST /api/changes
Content-Type: application/json
x-user-id: {user_uuid}

{
  "number": "ECO-001",
  "title": "Название изменения",
  "description": "Описание изменения",
  "reason": "Причина изменения",
  "change_type": "eco",  # eco | ecr | mcr
  "priority": "high"     # low | medium | high | critical
}

Response: ChangeOrder (status: draft)
```

#### Получить ECO
```bash
GET /api/changes/{id}
x-user-id: {user_uuid}

Response: ChangeOrder
```

#### Список ECO
```bash
GET /api/changes?status=draft&change_type=eco&page=1&per_page=20
x-user-id: {user_uuid}

Response: { items: [ChangeOrder], total: number, page: number }
```

#### Обновить ECO
```bash
PUT /api/changes/{id}
Content-Type: application/json
x-user-id: {user_uuid}

{
  "title": "Новое название",
  "description": "Новое описание",
  "priority": "critical",
  "assignee_id": "{user_uuid}",
  "due_date": "2026-12-31T23:59:59Z"
}

Response: ChangeOrder
```

#### Удалить ECO
```bash
DELETE /api/changes/{id}
x-user-id: {user_uuid}

Response: 204 No Content
```

---

### Workflow Transitions

#### Отправить на утверждение
```bash
POST /api/changes/{id}/submit
x-user-id: {user_uuid}

Response: ChangeOrder (status: submitted)
```

#### Финальное утверждение (менеджером)
```bash
POST /api/changes/{id}/approve
Content-Type: application/json
x-user-id: {user_uuid}

{
  "comment": "Утверждено к внедрению"
}

Response: ChangeOrder (status: approved)
```

#### Отклонить ECO
```bash
POST /api/changes/{id}/reject
Content-Type: application/json
x-user-id: {user_uuid}

{
  "comment": "Причина отклонения"
}

Response: ChangeOrder (status: rejected)
```

#### Реализовать ECO
```bash
POST /api/changes/{id}/implement
Content-Type: application/json
x-user-id: {user_uuid}

{
  "comment": "Внедрено в производство"
}

Response: ChangeOrder (status: implemented)
```

#### Отменить ECO
```bash
POST /api/changes/{id}/cancel
Content-Type: application/json
x-user-id: {user_uuid}

{
  "comment": "Причина отмены"
}

Response: ChangeOrder (status: cancelled)
```

---

### Affected Items (затронутые элементы)

#### Добавить затронутую деталь или документ
```bash
POST /api/changes/{id}/affected-items
Content-Type: application/json
x-user-id: {user_uuid}

# Для детали:
{ "part_id": "{part_uuid}" }

# Для документа:
{ "document_id": "{document_uuid}" }

Response: AffectedItem
```

#### Список затронутых элементов
```bash
GET /api/changes/{id}/affected-items
x-user-id: {user_uuid}

Response: [AffectedItem]
```

#### Удалить затронутый элемент
```bash
DELETE /api/changes/{id}/affected-items?part_id={part_uuid}
# или
DELETE /api/changes/{id}/affected-items?document_id={document_uuid}
x-user-id: {user_uuid}

Response: 204 No Content
```

---

### Approvals (утверждения)

#### Добавить утверждающего
```bash
POST /api/changes/{id}/approvers
Content-Type: application/json
x-user-id: {user_uuid}

{
  "approver_id": "{user_uuid}"
}

Response: Approval
```

#### Список утверждающих
```bash
GET /api/changes/{id}/approvals
x-user-id: {user_uuid}

Response: [Approval]
```

#### Обновить статус утверждения (от имени утверждающего)
```bash
POST /api/changes/{id}/approvals/{approver_id}
Content-Type: application/json
x-user-id: {approver_uuid}  # должен совпадать с approver_id

{
  "status": "approved",  # approved | rejected
  "comment": "Одобрено"
}

Response: Approval
```

---

### History & Comments

#### История изменений ECO
```bash
GET /api/changes/{id}/history
x-user-id: {user_uuid}

Response: [ChangeHistory]
```

#### Добавить комментарий
```bash
POST /api/changes/{id}/comments
Content-Type: application/json
x-user-id: {user_uuid}

{
  "comment": "Текст комментария"
}

Response: ChangeComment
```

#### Список комментариев
```bash
GET /api/changes/{id}/comments
x-user-id: {user_uuid}

Response: [ChangeComment]
```

---

## Модели данных

### ChangeOrder
```rust
{
  "id": "uuid",
  "number": "ECO-001",
  "title": "Название",
  "description": "Описание",
  "reason": "Причина",
  "change_type": "eco",
  "status": "draft",
  "priority": "high",
  "initiator_id": "uuid",
  "assignee_id": "uuid | null",
  "due_date": "timestamp | null",
  "created_at": "timestamp",
  "updated_at": "timestamp",
  "updated_by": "uuid"
}
```

### AffectedItem
```rust
{
  "change_id": "uuid",
  "part_id": "uuid | null",
  "document_id": "uuid | null",
  "created_at": "timestamp"
}
```

### Approval
```rust
{
  "id": "uuid",
  "change_order_id": "uuid",
  "approver_id": "uuid",
  "status": "pending",  # pending | approved | rejected
  "comment": "string | null",
  "created_at": "timestamp",
  "updated_at": "timestamp"
}
```

### ChangeHistory
```rust
{
  "id": "uuid",
  "change_order_id": "uuid",
  "user_id": "uuid",
  "action": "created",  # created | submitted | approved | rejected | implemented | cancelled
  "details": "jsonb | null",
  "created_at": "timestamp"
}
```

### ChangeComment
```rust
{
  "id": "uuid",
  "change_order_id": "uuid",
  "user_id": "uuid",
  "comment": "string",
  "created_at": "timestamp"
}
```

---

## Пример полного workflow

```bash
# 1. Создать ECO
ECO_ID=$(curl -s -X POST http://localhost:3005/api/changes \
  -H "x-user-id: $INITIATOR_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "number": "ECO-003",
    "title": "Изменение материала корпуса",
    "description": "Замена стали на алюминий",
    "reason": "Снижение веса",
    "change_type": "eco",
    "priority": "medium"
  }' | jq -r .id)

# 2. Добавить затронутую деталь
curl -s -X POST "http://localhost:3005/api/changes/$ECO_ID/affected-items" \
  -H "x-user-id: $INITIATOR_ID" \
  -H "Content-Type: application/json" \
  -d "{\"part_id\": \"$PART_ID\"}"

# 3. Отправить на утверждение
curl -s -X POST "http://localhost:3005/api/changes/$ECO_ID/submit" \
  -H "x-user-id: $INITIATOR_ID"

# 4. Добавить утверждающих
curl -s -X POST "http://localhost:3005/api/changes/$ECO_ID/approvers" \
  -H "x-user-id: $INITIATOR_ID" \
  -H "Content-Type: application/json" \
  -d "{\"approver_id\": \"$APPROVER1_ID\"}"

curl -s -X POST "http://localhost:3005/api/changes/$ECO_ID/approvers" \
  -H "x-user-id: $INITIATOR_ID" \
  -H "Content-Type: application/json" \
  -d "{\"approver_id\": \"$APPROVER2_ID\"}"

# 5. Утверждение от approver1
curl -s -X POST "http://localhost:3005/api/changes/$ECO_ID/approvals/$APPROVER1_ID" \
  -H "x-user-id: $APPROVER1_ID" \
  -H "Content-Type: application/json" \
  -d '{"status": "approved", "comment": "Одобрено инженером"}'

# 6. Утверждение от approver2
curl -s -X POST "http://localhost:3005/api/changes/$ECO_ID/approvals/$APPROVER2_ID" \
  -H "x-user-id: $APPROVER2_ID" \
  -H "Content-Type: application/json" \
  -d '{"status": "approved", "comment": "Одобрено ОТК"}'

# 7. Финальное утверждение менеджером
curl -s -X POST "http://localhost:3005/api/changes/$ECO_ID/approve" \
  -H "x-user-id: $MANAGER_ID" \
  -H "Content-Type: application/json" \
  -d '{"comment": "Утверждено к внедрению"}'

# 8. Реализация
curl -s -X POST "http://localhost:3005/api/changes/$ECO_ID/implement" \
  -H "x-user-id: $MANAGER_ID" \
  -H "Content-Type: application/json" \
  -d '{"comment": "Внедрено в производство"}'

# 9. Проверка истории
curl -s "http://localhost:3005/api/changes/$ECO_ID/history" \
  -H "x-user-id: $INITIATOR_ID" | jq '.[] | {action, created_at}'
```

---

## База данных

### Таблица `changes`
```sql
CREATE TABLE changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    number VARCHAR(100) UNIQUE NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    reason TEXT,
    change_type VARCHAR(20) NOT NULL CHECK (change_type IN ('eco', 'ecr', 'mcr')),
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'submitted', 'inreview', 'approved', 'rejected', 'implemented', 'cancelled')),
    priority VARCHAR(20) NOT NULL CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    initiator_id UUID NOT NULL REFERENCES users(id),
    assignee_id UUID REFERENCES users(id),
    due_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);
```

### Индексы
- `idx_changes_number` — поиск по номеру
- `idx_changes_status` — фильтрация по статусу
- `idx_changes_type` — фильтрация по типу
- `idx_changes_priority` — фильтрация по приоритету
- `idx_changes_initiator_id` — ECO по инициатору
- `idx_changes_assignee_id` — ECO по ответственному
- `idx_changes_due_date` — сортировка по сроку
- `idx_changes_search` — полнотекстовый поиск (GIN, russian)

---

## Запуск сервиса

```bash
# Переменные окружения
export DATABASE_URL="postgresql://schturman:schturman_password@localhost:5434/schturman"
export JWT_SECRET="test_secret_key_12345"

# Сборка
cd /mnt/storage/Штурман
cargo build --release -p change-service

# Запуск через systemd-run
systemctl --user stop change-service.service 2>/dev/null || true
systemd-run --user --unit=change-service \
  --working-directory=/mnt/storage/Штурман/services/change-service \
  --setenv=DATABASE_URL="$DATABASE_URL" \
  --setenv=JWT_SECRET="$JWT_SECRET" \
  /mnt/storage/Штурман/target/release/change-service

# Проверка статуса
systemctl --user status change-service.service

# Логи
journalctl --user -u change-service.service -f
```

---

## Тестирование

Создано 2 тестовых ECO:
- **ECO-001**: статус `implemented` (полный цикл)
- **ECO-002**: статус `implemented` (полный цикл)

Тестовые пользователи:
- **tester**: `eb649f37-843d-4a53-9854-fb1aac65cd4d` (пароль: `test1234`)
- **approver**: `ae0994ce-ba4d-4cd6-b853-9d306b7d1662` (пароль: `test1234`)
- **manager**: `ecab0d4f-ee35-4e9b-afcd-a083891a87a4` (пароль: `test1234`)

---

## Следующие шаги (Phase 3)

1. **Notification system** — уведомления при изменении статуса ECO
2. **Email integration** — отправка писем утверждающим
3. **Document versioning** — связь ECO с версиями документов
4. **BOM impact analysis** — автоматический анализ влияния на BOM
5. **Workflow customization** — настраиваемые workflow для разных типов изменений
6. **Reports** — отчеты по изменениям (статистика, метрики)

---

## Известные ограничения

1. **Аутентификация**: используется простой `x-user-id` header вместо JWT (для Phase 3)
2. **Permissions**: нет проверки прав доступа (любой пользователь может создать/утвердить ECO)
3. **Notifications**: нет автоматических уведомлений при смене статуса
4. **File attachments**: нет возможности прикрепить файлы к ECO напрямую (только через affected documents)
5. **Workflow validation**: нет проверки, что все approvers одобрили перед финальным approve

---

## Архитектурные решения

### Почему `changes` вместо `change_orders`?
- Унификация терминологии: ECO/ECR/MCR объединены в один тип `changes`
- Упрощение схемы: одна таблица вместо трех
- Совместимость с миграциями из AGENTS.md

### Почему `change_order_id` в `change_approvals`?
- Историческая причина: изначально таблица называлась `change_orders`
- FK constraint переименован, но колонка сохранила имя для обратной совместимости
- В Phase 3 можно переименовать в `change_id` для консистентности

### Почему affected_items разделены на `change_affected_parts` и `change_affected_documents`?
- Разные FK constraints: части и документы в разных таблицах
- Упрощение запросов: можно фильтровать только части или только документы
- Потенциальное расширение: возможность добавить специфичные поля для каждого типа

---

## Ссылки

- [AGENTS.md](/home/user/Загрузки/AGENTS.md) — общий план разработки
- [Phase 0: Infrastructure](phase0-infrastructure.md) — базовая инфраструктура
- [Phase 1: Core Services](phase1-core-services.md) — auth, part, document, bom services
- Auth service API: http://localhost:3001
- Part service API: http://localhost:3002
- Document service API: http://localhost:3003
- BOM service API: http://localhost:3004
- Change service API: http://localhost:3005
