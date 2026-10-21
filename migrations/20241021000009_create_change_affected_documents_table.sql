-- Создание таблицы связи изменений с документами
CREATE TABLE IF NOT EXISTS change_affected_documents (
    change_id UUID NOT NULL REFERENCES changes(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (change_id, document_id)
);

-- Индексы
CREATE INDEX idx_change_affected_documents_change_id ON change_affected_documents(change_id);
CREATE INDEX idx_change_affected_documents_document_id ON change_affected_documents(document_id);

-- Комментарии
COMMENT ON TABLE change_affected_documents IS 'Связь изменений с затронутыми документами';
