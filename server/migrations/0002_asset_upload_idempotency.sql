CREATE TABLE IF NOT EXISTS asset_upload_idempotency (
    request_method VARCHAR(8) COLLATE utf8mb4_bin NOT NULL,
    request_path VARCHAR(255) COLLATE utf8mb4_bin NOT NULL,
    idempotency_key BINARY(16) NOT NULL,
    claim_token BINARY(16) NOT NULL,
    file_sha256 BINARY(32) NOT NULL,
    alt VARCHAR(200) NOT NULL,
    status VARCHAR(16) NOT NULL,
    object_key VARCHAR(512) COLLATE utf8mb4_bin NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    expires_at TIMESTAMP(3) NOT NULL,
    completed_at TIMESTAMP(3) NULL,

    PRIMARY KEY (
        request_method,
        request_path,
        idempotency_key
    ),

    KEY idx_asset_upload_idempotency_expires_at (
        expires_at
    ),

    CONSTRAINT chk_asset_upload_idempotency_method
        CHECK (request_method = 'POST'),

    CONSTRAINT chk_asset_upload_idempotency_alt
        CHECK (CHAR_LENGTH(alt) BETWEEN 1 AND 200),

    CONSTRAINT chk_asset_upload_idempotency_status
        CHECK (status IN ('processing', 'completed')),

    CONSTRAINT chk_asset_upload_idempotency_expiration
        CHECK (expires_at > created_at),

    CONSTRAINT chk_asset_upload_idempotency_completed_fields
        CHECK (
            (
                status = 'processing'
                AND object_key IS NULL
                AND completed_at IS NULL
            )
            OR
            (
                status = 'completed'
                AND object_key IS NOT NULL
                AND completed_at IS NOT NULL
            )
        ),

    CONSTRAINT chk_asset_upload_idempotency_object_key
        CHECK (
            object_key IS NULL
            OR object_key LIKE 'v1/problems/%'
        )
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;
