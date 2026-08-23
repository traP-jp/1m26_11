use std::{
    env,
    error::Error,
    io::{self, ErrorKind},
    path::PathBuf,
};

use server::{
    migrate,
    problem::{load_problem_data, seed_problem_data},
};
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("failed to seed problem data: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let problem_data_root = env::args_os().nth(1).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidInput,
            "usage: seed_problem_data <problem-data-directory>",
        )
    })?;

    let database_url = env::var("DATABASE_URL").map_err(|_| {
        io::Error::new(
            ErrorKind::NotFound,
            "DATABASE_URL must be set for problem data seeding",
        )
    })?;

    // DBへ接続する前に、すべてのJSONを読み込んでvalidationする。
    let catalog = load_problem_data(&problem_data_root)?;

    let pool = MySqlPoolOptions::new().connect(&database_url).await?;

    migrate(&pool).await?;

    let result = seed_problem_data(&pool, &catalog).await;

    pool.close().await;

    let summary = result?;

    println!(
        "seeded {} room(s) and {} problem(s)",
        summary.room_count, summary.problem_count
    );

    Ok(())
}
