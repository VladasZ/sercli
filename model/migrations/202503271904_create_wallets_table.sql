CREATE TYPE "wallet_type" AS ENUM (
    'fiat',
    'crypto'
    );

CREATE TABLE "wallets"
(
    "id"      bigserial PRIMARY KEY,
    "user_id" bigint      NOT NULL,
    "name"    varchar     NOT NULL,
    "amount"  decimal     NOT NULL,
    "tp"      wallet_type NOT NULL
);

ALTER TABLE "wallets"
    ADD CONSTRAINT "user_wallet" FOREIGN KEY ("user_id") REFERENCES "users" ("id");
