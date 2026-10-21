-- Создание таблицы связи изменений с компонентами
CREATE TABLE IF NOT EXISTS change_affected_parts (
    change_id UUID NOT NULL REFERENCES changes(id) ON DELETE CASCADE,
    part_id UUID NOT NULL REFERENCES parts(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (change_id, part_id)
);

-- Индексы
CREATE INDEX idx_change_affected_parts_change_id ON change_affected_parts(change_id);
CREATE INDEX idx_change_affected_parts_part_id ON change_affected_parts(part_id);

-- Комментарии
COMMENT ON TABLE change_affected_parts IS 'Связь изменений с затронутыми компонентами';
