CREATE TABLE "users"
(
    "id"       bigserial PRIMARY KEY,
    "email"    varchar UNIQUE NOT NULL,
    "password" varchar        NOT NULL,
    "age"      integer        NOT NULL
);
