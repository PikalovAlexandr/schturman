-- AI Tasks table
CREATE TABLE ai_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR(50) NOT NULL CHECK (task_type IN (
        'semantic_search', 'cad_analysis', 'bom_validation',
        'change_impact', 'document_extraction', 'llm_summary'
    )),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending', 'processing', 'completed', 'failed', 'cancelled'
    )),
    input_data JSONB NOT NULL,
    result_data JSONB,
    error_message TEXT,
    priority INTEGER NOT NULL DEFAULT 5 CHECK (priority BETWEEN 1 AND 10),
    user_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Индексы
CREATE INDEX idx_ai_tasks_user_id ON ai_tasks(user_id);
CREATE INDEX idx_ai_tasks_status ON ai_tasks(status);
CREATE INDEX idx_ai_tasks_task_type ON ai_tasks(task_type);
CREATE INDEX idx_ai_tasks_created_at ON ai_tasks(created_at DESC);
CREATE INDEX idx_ai_tasks_priority ON ai_tasks(priority DESC, created_at);

-- Триггер обновления updated_at
CREATE TRIGGER update_ai_tasks_updated_at
    BEFORE UPDATE ON ai_tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE ai_tasks IS 'AI task queue and execution tracking';
COMMENT ON COLUMN ai_tasks.task_type IS 'Type of AI operation to perform';
COMMENT ON COLUMN ai_tasks.input_data IS 'Input parameters for the AI task (JSON)';
COMMENT ON COLUMN ai_tasks.result_data IS 'Task execution result (JSON)';
COMMENT ON COLUMN ai_tasks.priority IS '1=lowest, 10=highest priority';
