create table if not exists archives (
  id text primary key,
  source_id text not null,
  archive_type text not null,
  normalized_zip_path text,
  page_count integer,
  cover_entry_id text,
  status text not null,
  foreign key (source_id) references sources(id)
);

create index if not exists idx_archives_source_id on archives (source_id);

create table if not exists archive_entries (
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

create unique index if not exists idx_archive_entries_archive_path
  on archive_entries (archive_id, entry_path);

create table if not exists thumbnails (
  thumbnail_key text primary key,
  asset_id text not null,
  profile text not null,
  width integer not null,
  height integer not null,
  format text not null,
  disk_path text not null,
  byte_size integer not null,
  state text not null,
  updated_at text not null,
  foreign key (asset_id) references media_assets(id)
);

create index if not exists idx_thumbnails_asset_id on thumbnails (asset_id);
