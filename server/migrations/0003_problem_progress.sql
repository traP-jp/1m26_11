CREATE TABLE IF NOT EXISTS problems (
    id BINARY(16) NOT NULL,
    room_id BINARY(16) NOT NULL,
    number INT NOT NULL,
    type VARCHAR(16) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    initial_status VARCHAR(16) NOT NULL DEFAULT 'locked',
    requires_previous TINYINT(1) NOT NULL DEFAULT 1,
    answer_length_limit INT NOT NULL DEFAULT 50,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uq_problems_room_number (room_id, number),
    CONSTRAINT fk_problems_room_id FOREIGN KEY (room_id) REFERENCES rooms (id),
    CONSTRAINT chk_problems_number CHECK (number > 0),
    CONSTRAINT chk_problems_initial_status CHECK (initial_status IN ('available', 'locked'))
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS problem_progress (
    run_id BINARY(16) NOT NULL,
    problem_id BINARY(16) NOT NULL,
    status VARCHAR(16) NOT NULL,
    answer_attempt_count INT NOT NULL DEFAULT 0,
    cleared_at TIMESTAMP(3) NULL,
    PRIMARY KEY (run_id, problem_id),
    KEY idx_problem_progress_run_status (run_id, status),
    CONSTRAINT fk_problem_progress_run_id FOREIGN KEY (run_id) REFERENCES runs (id) ON DELETE CASCADE,
    CONSTRAINT fk_problem_progress_problem_id FOREIGN KEY (problem_id) REFERENCES problems (id) ON DELETE CASCADE,
    CONSTRAINT chk_problem_progress_status CHECK (status IN ('locked', 'available', 'cleared')),
    CONSTRAINT chk_problem_progress_answer_attempt_count CHECK (answer_attempt_count >= 0),
    CONSTRAINT chk_problem_progress_cleared_at CHECK (
        (status = 'cleared' AND cleared_at IS NOT NULL) OR
        (status <> 'cleared' AND cleared_at IS NULL)
    )
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;
