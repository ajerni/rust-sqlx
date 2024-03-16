CREATE TABLE book (
    isbn VARCHAR(255) PRIMARY KEY,
    title VARCHAR(255),
    author VARCHAR(255),
    metadata JSONB
);