-- Add migration script here
CREATE TABLE blog_posts(
    slug TEXT NOT NULL,
    PRIMARY KEY (slug)
);
