CREATE TABLE IF NOT EXISTS rooms (
    id BINARY(16) NOT NULL,
    number INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    genre VARCHAR(64) NOT NULL,
    description TEXT NOT NULL,
    is_published TINYINT(1) NOT NULL DEFAULT 0,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uq_rooms_number (number),
    CONSTRAINT chk_rooms_number CHECK (number > 0)
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS runs (
    id BINARY(16) NOT NULL,
    user_id BINARY(16) NOT NULL,
    room_id BINARY(16) NOT NULL,
    status VARCHAR(16) NOT NULL,
    started_at TIMESTAMP(3) NOT NULL,
    cleared_at TIMESTAMP(3) NULL,
    active_marker TINYINT(1) GENERATED ALWAYS AS (CASE WHEN status = 'active' THEN 1 ELSE NULL END) STORED,
    PRIMARY KEY (id),
    UNIQUE KEY uq_runs_user_room_active (user_id, room_id, active_marker),
    CONSTRAINT fk_runs_user_id FOREIGN KEY (user_id) REFERENCES users (id),
    CONSTRAINT fk_runs_room_id FOREIGN KEY (room_id) REFERENCES rooms (id),
    CONSTRAINT chk_runs_status CHECK (status IN ('active', 'cleared')),
    CONSTRAINT chk_runs_cleared_at CHECK (
        (status = 'active' AND cleared_at IS NULL) OR
        (status = 'cleared' AND cleared_at IS NOT NULL)
    ),
    CONSTRAINT chk_runs_times CHECK (cleared_at IS NULL OR cleared_at >= started_at)
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;

CREATE INDEX idx_runs_ranking ON runs (room_id, status, user_id, cleared_at, started_at);
