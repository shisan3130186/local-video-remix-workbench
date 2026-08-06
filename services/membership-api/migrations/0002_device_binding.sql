ALTER TABLE memberships ADD COLUMN bound_device_hash TEXT;
ALTER TABLE memberships ADD COLUMN rebind_month TEXT;
ALTER TABLE memberships ADD COLUMN rebind_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD COLUMN device_hash TEXT;
