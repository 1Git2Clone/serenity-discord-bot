CREATE TABLE reminders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    remind_at TIMESTAMPTZ NOT NULL,
    message TEXT NOT NULL
);

-- Drives the polling query (due reminders) and the per-user pending count.
CREATE INDEX idx_reminders_remind_at ON reminders (remind_at);
CREATE INDEX idx_reminders_user_id ON reminders (user_id);
