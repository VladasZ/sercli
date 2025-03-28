CREATE TABLE wallets
(
    id      SERIAL PRIMARY KEY,
    user_id INTEGER        NOT NULL,
    name    VARCHAR(255)   NOT NULL,
    amount  DECIMAL(10, 2) NOT NULL
);
