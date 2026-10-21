-- Add queued status to ai_tasks
ALTER TABLE ai_tasks 
DROP CONSTRAINT ai_tasks_status_check;

ALTER TABLE ai_tasks 
ADD CONSTRAINT ai_tasks_status_check 
CHECK (status IN ('queued', 'pending', 'processing', 'completed', 'failed', 'cancelled'));
