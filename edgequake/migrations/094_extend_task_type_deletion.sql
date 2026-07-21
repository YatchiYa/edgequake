-- Migration 094: Extend tasks.valid_task_type for deletion + knowledge_injection
--
-- WHY: TaskType::Deletion (async document cascade delete) and
-- TaskType::KnowledgeInjection must persist to `tasks.task_type`. Migration 026
-- only allowed upload/insert/scan/reindex/pdf_processing, so enqueue failed with:
--   new row for relation "tasks" violates check constraint "valid_task_type"
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
        'deletion'
    )
);
