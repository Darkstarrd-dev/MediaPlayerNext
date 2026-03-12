create table if not exists media_sources (
  id text primary key,
  library_id text not null,
  source_type text not null,
  backing_source_id text,
  absolute_path text not null,
  tree_path_json text not null,
  display_name text not null,
  item_count integer not null,
  cover_asset_id text,
  last_seen_revision text,
  exists_flag integer not null,
  updated_at text not null,
  foreign key (library_id) references libraries(id),
  foreign key (backing_source_id) references sources(id),
  foreign key (cover_asset_id) references media_assets(id)
);

create unique index if not exists idx_media_sources_library_path
  on media_sources (library_id, absolute_path);

create index if not exists idx_media_sources_library_updated
  on media_sources (library_id, updated_at desc);

create index if not exists idx_media_sources_backing_source_id
  on media_sources (backing_source_id);

create table if not exists image_items (
  id text primary key,
  media_source_id text not null,
  asset_id text not null,
  ordinal integer not null,
  width integer,
  height integer,
  size_bytes integer,
  media_locator_json text,
  hidden_flag integer not null,
  last_seen_revision text,
  updated_at text not null,
  foreign key (media_source_id) references media_sources(id),
  foreign key (asset_id) references media_assets(id)
);

create unique index if not exists idx_image_items_source_ordinal
  on image_items (media_source_id, ordinal);

create index if not exists idx_image_items_asset_id
  on image_items (asset_id);
