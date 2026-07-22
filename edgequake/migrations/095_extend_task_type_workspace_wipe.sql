-- Migration 095: Extend tasks.valid_task_type for workspace_wipe
--
-- WHY: TaskType::WorkspaceWipe (durable DELETE /documents wipe-all) must
-- persist to `tasks.task_type`. Migration 094 allowed deletion + knowledge_injection
-- but not workspace_wipe.
--
-- SAFE: DROP IF EXISTS + recreate; idempotent on restart.

ALTER TABLE tasks DROP CONSTRAINT IF EXISTS valid_task_type;
ALTER TABLE tasks DROP CONSTRAINT IF EXISTS tasks_valid_type;

ALTER TABLE tasks ADD CONSTRAINT valid_task_type CHECK (
    task_type IN (
        'upload',
        'insert',
        'scan',
        'reindex',
        'pdf_processing',
        'knowledge_injection',
        'deletion',
        'workspace_wipe'
    )
);
