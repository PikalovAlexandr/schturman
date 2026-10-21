-- Создание таблицы документов
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    format VARCHAR(20) NOT NULL CHECK (format IN ('step', 'iges', 'stl', 'pdf', 'dwg', 'dxf', 'image', 'other')),
    status VARCHAR(20) NOT NULL CHECK (status IN ('draft', 'checkedout', 'checkedin', 'released', 'obsolete')),
    version VARCHAR(50) NOT NULL DEFAULT '1.0',
    file_size BIGINT NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    storage_path VARCHAR(1000) NOT NULL,
    thumbnail_url VARCHAR(1000),
    part_id UUID REFERENCES parts(id) ON DELETE SET NULL,
    checked_out_by UUID REFERENCES users(id) ON DELETE SET NULL,
    checked_out_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- Индексы
CREATE INDEX idx_documents_name ON documents(name);
CREATE INDEX idx_documents_format ON documents(format);
CREATE INDEX idx_documents_status ON documents(status);
CREATE INDEX idx_documents_part_id ON documents(part_id);
CREATE INDEX idx_documents_file_hash ON documents(file_hash);
CREATE INDEX idx_documents_checked_out_by ON documents(checked_out_by);

-- Комментарии
COMMENT ON TABLE documents IS 'Документы и CAD-файлы';
COMMENT ON COLUMN documents.format IS 'Формат: step, iges, stl, pdf, dwg, dxf, image, other';
COMMENT ON COLUMN documents.file_hash IS 'SHA-256 хеш файла для content-addressable storage';
