create table if not exists app_state (
  state_key text primary key,
  state_json text not null,
  updated_at text not null
);

create index if not exists idx_app_state_updated_at
  on app_state (updated_at desc);
