-- 主キー列を改名するため、参照している外部キーをいったん削除する。

ALTER TABLE demo_sessions
    DROP FOREIGN KEY fk_demo_sessions_user_id;

ALTER TABLE runs
    DROP FOREIGN KEY fk_runs_user_id,
    DROP FOREIGN KEY fk_runs_room_id;

ALTER TABLE problems
    DROP FOREIGN KEY fk_problems_room_id,
    DROP FOREIGN KEY fk_problems_depends_on,
    DROP CONSTRAINT chk_problems_no_self_dependency;

ALTER TABLE problem_progress
    DROP FOREIGN KEY fk_problem_progress_run_id,
    DROP FOREIGN KEY fk_problem_progress_problem_id;


-- 用途が分かるように、各テーブルの主キー列を改名する。
-- 列名だけを変更し、BINARY(16)の型や主キー設定は維持する。

ALTER TABLE users
    RENAME COLUMN id TO user_id;

ALTER TABLE demo_sessions
    RENAME COLUMN id TO session_id;

ALTER TABLE rooms
    RENAME COLUMN id TO room_id;

ALTER TABLE problems
    RENAME COLUMN id TO problem_id;

ALTER TABLE runs
    RENAME COLUMN id TO run_id;


-- booleanとして扱う列に、0または1だけを許可するCHECK制約を追加する。

ALTER TABLE rooms
    ADD CONSTRAINT chk_rooms_is_published
        CHECK (is_published IN (0, 1));

ALTER TABLE problems
    ADD CONSTRAINT chk_problems_is_required
        CHECK (is_required IN (0, 1)),
    ADD CONSTRAINT chk_problems_no_self_dependency
        CHECK (
            depends_on_problem_id IS NULL
            OR depends_on_problem_id <> problem_id
        );


-- 新しい主キー名を参照する外部キーを作り直す。
-- セッションはユーザー削除時に一緒に削除する。

ALTER TABLE demo_sessions
    ADD CONSTRAINT fk_demo_sessions_user_id
        FOREIGN KEY (user_id)
        REFERENCES users (user_id)
        ON DELETE CASCADE;


-- 部屋や問題などのマスターデータは、利用中なら削除を拒否する。

ALTER TABLE problems
    ADD CONSTRAINT fk_problems_room_id
        FOREIGN KEY (room_id)
        REFERENCES rooms (room_id)
        ON DELETE RESTRICT,
    ADD CONSTRAINT fk_problems_depends_on
        FOREIGN KEY (depends_on_problem_id)
        REFERENCES problems (problem_id)
        ON DELETE RESTRICT;

ALTER TABLE runs
    ADD CONSTRAINT fk_runs_user_id
        FOREIGN KEY (user_id)
        REFERENCES users (user_id)
        ON DELETE RESTRICT,
    ADD CONSTRAINT fk_runs_room_id
        FOREIGN KEY (room_id)
        REFERENCES rooms (room_id)
        ON DELETE RESTRICT;


-- runやproblemが削除された場合、対応する進捗も一緒に削除する。

ALTER TABLE problem_progress
    ADD CONSTRAINT fk_problem_progress_run_id
        FOREIGN KEY (run_id)
        REFERENCES runs (run_id)
        ON DELETE CASCADE,
    ADD CONSTRAINT fk_problem_progress_problem_id
        FOREIGN KEY (problem_id)
        REFERENCES problems (problem_id)
        ON DELETE CASCADE;


-- 絞り込み操作の履歴を保存するqueriesテーブルを追加する。

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
