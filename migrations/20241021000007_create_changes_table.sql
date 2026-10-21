-- Создание таблицы изменений (ECO/ECR)
CREATE TABLE IF NOT EXISTS changes (
    id UUID PRIMARY KEY,
    number VARCHAR(100) UNIQUE NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    reason TEXT,
    change_type VARCHAR(20) NOT NULL CHECK (change_type IN ('eco', 'ecr', 'mcr')),
    status VARCHAR(20) NOT NULL CHECK (status IN ('draft', 'submitted', 'inreview', 'approved', 'rejected', 'implemented', 'cancelled')),
    priority VARCHAR(20) NOT NULL CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    initiator_id UUID NOT NULL REFERENCES users(id),
    assignee_id UUID REFERENCES users(id),
    due_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- Индексы
CREATE INDEX idx_changes_number ON changes(number);
CREATE INDEX idx_changes_type ON changes(change_type);
CREATE INDEX idx_changes_status ON changes(status);
CREATE INDEX idx_changes_priority ON changes(priority);
CREATE INDEX idx_changes_initiator_id ON changes(initiator_id);
CREATE INDEX idx_changes_assignee_id ON changes(assignee_id);
CREATE INDEX idx_changes_due_date ON changes(due_date);

-- Полнотекстовый поиск
CREATE INDEX idx_changes_search ON changes USING GIN (
    to_tsvector('russian', coalesce(title, '') || ' ' || coalesce(description, '') || ' ' || coalesce(number, ''))
);

-- Комментарии
COMMENT ON TABLE changes IS 'Инженерные изменения (ECO/ECR/MCR)';
COMMENT ON COLUMN changes.change_type IS 'Тип: eco (Engineering Change Order), ecr (Engineering Change Request), mcr (Manufacturing Change Request)';
COMMENT ON COLUMN changes.status IS 'Статус: draft, submitted, inreview, approved, rejected, implemented, cancelled';
