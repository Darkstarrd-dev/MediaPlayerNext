create table libraries (
  id text primary key,
  root_path text not null,
  library_type text not null,
  scan_mode text not null,
  created_at text not null,
  updated_at text not null
);

create table sources (
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

create unique index idx_sources_library_path on sources (library_id, normalized_path);

create table media_assets (
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

create index idx_media_assets_source_ref on media_assets (source_kind, source_ref_id);

create table tasks (
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

create table archives (
  id text primary key,
  source_id text not null,
  archive_type text not null,
  normalized_zip_path text,
  page_count integer,
  cover_entry_id text,
  status text not null,
  foreign key (source_id) references sources(id)
);

create index idx_archives_source_id on archives (source_id);

create table archive_entries (
  id text primary key,
  archive_id text not null,
  entry_path text not null,
  entry_name text not null,
  page_index integer not null,
  media_kind text not null,
  width integer,
  height integer,
  compressed_size integer,
  uncompressed_size integer,
  crc32 integer,
  foreign key (archive_id) references archives(id)
);

create unique index idx_archive_entries_archive_path on archive_entries (archive_id, entry_path);

create table thumbnails (
  thumbnail_key text primary key,
  asset_id text not null,
  profile text not null,
  width integer not null,
  height integer not null,
  format text not null,
  disk_path text not null,
  byte_size integer not null,
  state text not null,
  updated_at text not null
);

create index idx_thumbnails_asset_id on thumbnails (asset_id);

pragma user_version = 2;
