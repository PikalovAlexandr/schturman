-- Создание таблицы компонентов
CREATE TABLE IF NOT EXISTS parts (
    id UUID PRIMARY KEY,
    number VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    part_type VARCHAR(20) NOT NULL CHECK (part_type IN ('part', 'assembly', 'purchased', 'material', 'standard')),
    status VARCHAR(20) NOT NULL CHECK (status IN ('draft', 'review', 'released', 'obsolete')),
    version VARCHAR(50) NOT NULL DEFAULT '1.0',
    revision INTEGER NOT NULL DEFAULT 1,
    attributes JSONB NOT NULL DEFAULT '{}',
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- Индексы
CREATE INDEX idx_parts_number ON parts(number);
CREATE INDEX idx_parts_name ON parts(name);
CREATE INDEX idx_parts_type ON parts(part_type);
CREATE INDEX idx_parts_status ON parts(status);
CREATE INDEX idx_parts_owner ON parts(owner_id);
CREATE INDEX idx_parts_attributes ON parts USING GIN (attributes);

-- Полнотекстовый поиск
CREATE INDEX idx_parts_search ON parts USING GIN (
    to_tsvector('russian', coalesce(name, '') || ' ' || coalesce(description, '') || ' ' || coalesce(number, ''))
);

-- Комментарии
COMMENT ON TABLE parts IS 'Компоненты и детали';
COMMENT ON COLUMN parts.part_type IS 'Тип: part, assembly, purchased, material, standard';
COMMENT ON COLUMN parts.status IS 'Статус: draft, review, released, obsolete';
COMMENT ON COLUMN parts.attributes IS 'Пользовательские атрибуты в формате JSONB';
