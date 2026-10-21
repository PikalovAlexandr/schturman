-- Создание таблицы согласований изменений
CREATE TABLE IF NOT EXISTS change_approvals (
    id UUID PRIMARY KEY,
    change_id UUID NOT NULL REFERENCES changes(id) ON DELETE CASCADE,
    approver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'approved', 'rejected')),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Индексы
CREATE INDEX idx_change_approvals_change_id ON change_approvals(change_id);
CREATE INDEX idx_change_approvals_approver_id ON change_approvals(approver_id);
CREATE INDEX idx_change_approvals_status ON change_approvals(status);

-- Уникальность: один согласующий не может иметь несколько записей для одного изменения
CREATE UNIQUE INDEX idx_change_approvals_unique ON change_approvals(change_id, approver_id);

-- Комментарии
COMMENT ON TABLE change_approvals IS 'Согласования инженерных изменений';
COMMENT ON COLUMN change_approvals.status IS 'Статус: pending, approved, rejected';
