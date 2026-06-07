CREATE TABLE reminders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    remind_at TIMESTAMPTZ NOT NULL,
    message TEXT NOT NULL,
    finished_at TIMESTAMPTZ -- NULL = pending; set to fire time once delivered
);

-- Partial index drives the polling query (pending and due).
CREATE INDEX idx_reminders_due ON reminders (remind_at) WHERE finished_at IS NULL;
-- Covers per-user listing, the pending count, and history pruning.
CREATE INDEX idx_reminders_user_id ON reminders (user_id);
