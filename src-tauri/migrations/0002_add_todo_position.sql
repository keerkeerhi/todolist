ALTER TABLE todos ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

UPDATE todos
SET position = (
  SELECT COUNT(*) - 1
  FROM todos AS earlier
  WHERE earlier.created_at < todos.created_at
     OR (earlier.created_at = todos.created_at AND earlier.id <= todos.id)
);
