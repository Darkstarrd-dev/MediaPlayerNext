create table if not exists libraries (
  id text primary key,
  root_path text not null,
  library_type text not null,
  scan_mode text not null,
  created_at text not null,
  updated_at text not null
);

create table if not exists sources (
  id text primary key,
  library_id text not null,
  normalized_path text not null,
  file_name text not null,
  ext text not null,
  kind text not null,
  size integer not null,
  mtime_ms integer not null,
  fingerprint text,
  exists_flag integer not null,
  last_seen_at text not null,
  foreign key (library_id) references libraries(id)
);

create unique index if not exists idx_sources_library_path
  on sources (library_id, normalized_path);

create table if not exists media_assets (
  id text primary key,
  source_kind text not null,
  source_ref_id text not null,
  mime text not null,
  width integer,
  height integer,
  duration_ms integer,
  codec_info_json text,
  orientation integer,
  created_at text not null
);

create index if not exists idx_media_assets_source_ref
  on media_assets (source_kind, source_ref_id);

create table if not exists tasks (
  id text primary key,
  task_type text not null,
  state text not null,
  current integer not null,
  total integer,
  message text,
  error_code text,
  error_message text,
  started_at text,
  finished_at text
);
