CREATE TABLE IF NOT EXISTS users (
    id BINARY(16) NOT NULL,
    auth_provider VARCHAR(32) NOT NULL,
    provider_subject VARCHAR(255) COLLATE utf8mb4_bin NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uq_users_auth_provider_subject (
        auth_provider,
        provider_subject
    ),
    CONSTRAINT chk_users_auth_provider
        CHECK (auth_provider IN ('demo', 'neoshowcase'))
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS demo_sessions (
    id BINARY(16) NOT NULL,
    user_id BINARY(16) NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    KEY idx_demo_sessions_user_id (user_id),
    CONSTRAINT fk_demo_sessions_user_id
        FOREIGN KEY (user_id) REFERENCES users (id)
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;
