CREATE TABLE IF NOT EXISTS images (
  name TEXT PRIMARY KEY,
  image_path TEXT NOT NULL,
  store_path TEXT NOT NULL,
  cols INTEGER NOT NULL,
  rows INTEGER NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL
);