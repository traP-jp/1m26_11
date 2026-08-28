CREATE TABLE IF NOT EXISTS users (
    user_id BINARY(16) NOT NULL,
    auth_provider VARCHAR(32) NOT NULL,
    provider_subject VARCHAR(255) COLLATE utf8mb4_bin NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    PRIMARY KEY (user_id),

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
    session_id BINARY(16) NOT NULL,
    user_id BINARY(16) NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    PRIMARY KEY (session_id),

    KEY idx_demo_sessions_user_id (user_id),

    CONSTRAINT fk_demo_sessions_user_id
        FOREIGN KEY (user_id)
        REFERENCES users (user_id)
        ON DELETE CASCADE
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;


CREATE TABLE IF NOT EXISTS rooms (
    room_id BINARY(16) NOT NULL,
    number INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    genre VARCHAR(64) NOT NULL,
    description TEXT NOT NULL,
    is_published TINYINT(1) NOT NULL DEFAULT 0,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    PRIMARY KEY (room_id),

    UNIQUE KEY uq_rooms_number (number),

    CONSTRAINT chk_rooms_number
        CHECK (number > 0),

    CONSTRAINT chk_rooms_is_published
        CHECK (is_published IN (0, 1))
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;


CREATE TABLE IF NOT EXISTS problems (
    problem_id BINARY(16) NOT NULL,
    room_id BINARY(16) NOT NULL,
    number INT NOT NULL,
    problem_type VARCHAR(16) NOT NULL,
    title VARCHAR(255) NOT NULL,
    body_markdown TEXT NOT NULL,
    submission_type VARCHAR(32) NOT NULL,
    assets JSON NOT NULL,
    input_schema JSON NOT NULL,
    hints JSON NOT NULL,
    judge_config JSON NOT NULL,
    depends_on_problem_id BINARY(16) NULL,
    is_required TINYINT(1) NOT NULL DEFAULT 1,

    PRIMARY KEY (problem_id),

    UNIQUE KEY uq_problems_room_number (
        room_id,
        number
    ),

    CONSTRAINT fk_problems_room_id
        FOREIGN KEY (room_id)
        REFERENCES rooms (room_id)
        ON DELETE RESTRICT,

    CONSTRAINT fk_problems_depends_on
        FOREIGN KEY (depends_on_problem_id)
        REFERENCES problems (problem_id)
        ON DELETE RESTRICT,

    CONSTRAINT chk_problems_number
        CHECK (number > 0),

    CONSTRAINT chk_problems_problem_type
        CHECK (problem_type IN ('small', 'final')),

    CONSTRAINT chk_problems_submission_type
        CHECK (submission_type IN ('operation_sequence', 'string')),

    CONSTRAINT chk_problems_no_self_dependency
        CHECK (
            depends_on_problem_id IS NULL
            OR depends_on_problem_id <> problem_id
        ),

    CONSTRAINT chk_problems_is_required
        CHECK (is_required IN (0, 1))
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;


CREATE TABLE IF NOT EXISTS runs (
    run_id BINARY(16) NOT NULL,
    user_id BINARY(16) NOT NULL,
    room_id BINARY(16) NOT NULL,
    status VARCHAR(16) NOT NULL,
    started_at TIMESTAMP(3) NOT NULL,
    cleared_at TIMESTAMP(3) NULL,
    active_marker TINYINT(1)
        GENERATED ALWAYS AS (
            CASE
                WHEN status = 'active' THEN 1
                ELSE NULL
            END
        ) STORED,

    PRIMARY KEY (run_id),

    UNIQUE KEY uq_runs_user_room_active (
        user_id,
        room_id,
        active_marker
    ),

    KEY idx_runs_ranking (
        room_id,
        status,
        user_id,
        cleared_at,
        started_at
    ),

    CONSTRAINT fk_runs_user_id
        FOREIGN KEY (user_id)
        REFERENCES users (user_id)
        ON DELETE RESTRICT,

    CONSTRAINT fk_runs_room_id
        FOREIGN KEY (room_id)
        REFERENCES rooms (room_id)
        ON DELETE RESTRICT,

    CONSTRAINT chk_runs_status
        CHECK (status IN ('active', 'cleared')),

    CONSTRAINT chk_runs_cleared_at
        CHECK (
            (status = 'active' AND cleared_at IS NULL)
            OR (status = 'cleared' AND cleared_at IS NOT NULL)
        ),

    CONSTRAINT chk_runs_times
        CHECK (
            cleared_at IS NULL
            OR cleared_at >= started_at
        )
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;


CREATE TABLE IF NOT EXISTS problem_progress (
    run_id BINARY(16) NOT NULL,
    problem_id BINARY(16) NOT NULL,
    status VARCHAR(16) NOT NULL,
    answer_attempt_count INT NOT NULL DEFAULT 0,
    max_hint_level INT NOT NULL DEFAULT 0,
    cleared_at TIMESTAMP(3) NULL,

    PRIMARY KEY (
        run_id,
        problem_id
    ),

    KEY idx_problem_progress_run_status (
        run_id,
        status
    ),

    CONSTRAINT fk_problem_progress_run_id
        FOREIGN KEY (run_id)
        REFERENCES runs (run_id)
        ON DELETE CASCADE,

    CONSTRAINT fk_problem_progress_problem_id
        FOREIGN KEY (problem_id)
        REFERENCES problems (problem_id)
        ON DELETE CASCADE,

    CONSTRAINT chk_problem_progress_status
        CHECK (status IN ('locked', 'available', 'cleared')),

    CONSTRAINT chk_problem_progress_answer_attempt_count
        CHECK (answer_attempt_count >= 0),

    CONSTRAINT chk_problem_progress_max_hint_level
        CHECK (max_hint_level >= 0),

    CONSTRAINT chk_problem_progress_cleared_at
        CHECK (
            (status = 'cleared' AND cleared_at IS NOT NULL)
            OR (status <> 'cleared' AND cleared_at IS NULL)
        )
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;


CREATE TABLE IF NOT EXISTS queries (
    query_id BINARY(16) NOT NULL,
    run_id BINARY(16) NOT NULL,
    problem_id BINARY(16) NOT NULL,
    source VARCHAR(16) NOT NULL,
    operations JSON NOT NULL,
    normalized_operations JSON NOT NULL,
    remaining_pattern_count INT NOT NULL,
    is_correct TINYINT(1) NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

    PRIMARY KEY (query_id),

    KEY idx_queries_run_problem_created_at (
        run_id,
        problem_id,
        created_at
    ),

    CONSTRAINT fk_queries_problem_progress
        FOREIGN KEY (run_id, problem_id)
        REFERENCES problem_progress (run_id, problem_id)
        ON DELETE CASCADE,

    CONSTRAINT chk_queries_source
        CHECK (
            source IN ('keyboard', 'mouse', 'serial', 'vr')
        ),

    CONSTRAINT chk_queries_remaining_pattern_count
        CHECK (remaining_pattern_count >= 0),

    CONSTRAINT chk_queries_is_correct
        CHECK (is_correct IN (0, 1))
) ENGINE = InnoDB
  DEFAULT CHARACTER SET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;
