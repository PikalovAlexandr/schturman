-- Создание таблицы векторных embeddings для AI-поиска
CREATE TABLE IF NOT EXISTS part_embeddings (
    id UUID PRIMARY KEY,
    part_id UUID NOT NULL REFERENCES parts(id) ON DELETE CASCADE,
    embedding_type VARCHAR(50) NOT NULL,
    embedding vector(384),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Индексы
CREATE INDEX idx_part_embeddings_part_id ON part_embeddings(part_id);
CREATE INDEX idx_part_embeddings_type ON part_embeddings(embedding_type);

-- Комментарии
COMMENT ON TABLE part_embeddings IS 'Векторные представления компонентов для семантического поиска';
COMMENT ON COLUMN part_embeddings.embedding IS 'Вектор embeddings (384 размерность для all-MiniLM-L6-v2)';
