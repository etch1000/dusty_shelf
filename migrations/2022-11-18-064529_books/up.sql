-- Your SQL goes here
CREATE TABLE books (
  id SERIAL PRIMARY KEY,
  title VARCHAR NOT NULL,
  author VARCHAR NOT NULL,
  description TEXT NOT NULL,
  published BOOLEAN NOT NULL DEFAULT 'f'
)
