-- Создание таблицы спецификаций (BOM)
CREATE TABLE IF NOT EXISTS boms (
    id UUID PRIMARY KEY,
    assembly_part_id UUID NOT NULL REFERENCES parts(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL DEFAULT '1.0',
    status VARCHAR(20) NOT NULL CHECK (status IN ('draft', 'review', 'released', 'obsolete')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- Индексы
CREATE INDEX idx_boms_assembly_part_id ON boms(assembly_part_id);
CREATE INDEX idx_boms_status ON boms(status);

-- Уникальность версий BOM для сборки
CREATE UNIQUE INDEX idx_boms_unique_version ON boms(assembly_part_id, version);

-- Комментарии
COMMENT ON TABLE boms IS 'Спецификации (Bill of Materials)';
