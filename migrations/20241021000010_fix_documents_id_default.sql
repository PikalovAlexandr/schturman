-- Добавление DEFAULT для id в таблице documents
ALTER TABLE documents
  ALTER COLUMN id SET DEFAULT gen_random_uuid();
