-- Add migration script here
CREATE TABLE email_delivery_queue (
   id uuid PRIMARY KEY NOT NULL,
   recipient TEXT NOT NULL,
   subject TEXT NOT NULL,
   email_html TEXT NOT NULL,
   email_text TEXT NOT NULL,
   created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
   n_retries INTEGER NOT NULL DEFAULT 0,
   send_after TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
