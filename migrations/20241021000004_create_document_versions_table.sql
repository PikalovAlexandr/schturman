-- Создание таблицы версий документов
CREATE TABLE IF NOT EXISTS document_versions (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    storage_path VARCHAR(1000) NOT NULL,
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

-- Индексы
CREATE INDEX idx_document_versions_document_id ON document_versions(document_id);
CREATE INDEX idx_document_versions_version ON document_versions(version);

-- Уникальность версий для документа
CREATE UNIQUE INDEX idx_document_versions_unique ON document_versions(document_id, version);

-- Комментарии
COMMENT ON TABLE document_versions IS 'История версий документов';
