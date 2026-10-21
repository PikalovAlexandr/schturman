-- Создание таблицы элементов спецификаций
CREATE TABLE IF NOT EXISTS bom_items (
    id UUID PRIMARY KEY,
    bom_id UUID NOT NULL REFERENCES boms(id) ON DELETE CASCADE,
    part_id UUID NOT NULL REFERENCES parts(id) ON DELETE RESTRICT,
    position VARCHAR(50) NOT NULL,
    quantity DOUBLE PRECISION NOT NULL CHECK (quantity > 0),
    unit VARCHAR(20) NOT NULL,
    note TEXT,
    level INTEGER NOT NULL DEFAULT 0 CHECK (level >= 0),
    parent_item_id UUID REFERENCES bom_items(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Индексы
CREATE INDEX idx_bom_items_bom_id ON bom_items(bom_id);
CREATE INDEX idx_bom_items_part_id ON bom_items(part_id);
CREATE INDEX idx_bom_items_parent_item_id ON bom_items(parent_item_id);
CREATE INDEX idx_bom_items_position ON bom_items(position);

-- Уникальность позиций в рамках одной спецификации
CREATE UNIQUE INDEX idx_bom_items_unique_position ON bom_items(bom_id, position);

-- Комментарии
COMMENT ON TABLE bom_items IS 'Элементы спецификаций';
COMMENT ON COLUMN bom_items.level IS 'Уровень вложенности (0 - верхний)';
COMMENT ON COLUMN bom_items.parent_item_id IS 'ID родительского элемента для иерархии';
