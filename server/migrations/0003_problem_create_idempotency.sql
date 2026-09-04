CREATE TABLE IF NOT EXISTS problem_create_idempotency (
    request_method VARCHAR(8) COLLATE utf8mb4_bin NOT NULL,
    request_path VARCHAR(255) COLLATE utf8mb4_bin NOT NULL,
    idempotency_key BINARY(16) NOT NULL,
    payload_sha256 BINARY(32) NOT NULL,
    problem_id BINARY(16) NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    PRIMARY KEY (
        request_method,
        request_path,
        idempotency_key
    ),

    KEY idx_problem_create_idempotency_problem_id (
        problem_id
    ),

    CONSTRAINT fk_problem_create_idempotency_problem_id
        FOREIGN KEY (problem_id)
        REFERENCES problems (problem_id)
        ON DELETE CASCADE,

    CONSTRAINT chk_problem_create_idempotency_method
        CHECK (request_method = 'POST')
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;
