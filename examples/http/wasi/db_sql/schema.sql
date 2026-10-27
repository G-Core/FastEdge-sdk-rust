-- Apply once, outside the app (psql, Neon SQL editor, or your migration tool):
--   psql "$DATABASE_URL" -f schema.sql
create table if not exists example_users (
    id       int8 primary key,
    email    text not null unique,
    nickname text,
    created  timestamptz not null default now(),
    meta     jsonb not null default '{}'
);

insert into example_users (id, email, nickname, meta) values
    (1, 'ann@example.com', null,    '{"plan": "pro"}'),
    (2, 'bob@example.com', 'bobby', '{"plan": "free"}')
on conflict (id) do nothing;
