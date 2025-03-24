CREATE TABLE users
(
    id       SERIAL PRIMARY KEY,
    email    VARCHAR(255) NOT NULL UNIQUE,
    age      SMALLINT     NOT NULL,
    name     VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL
);


