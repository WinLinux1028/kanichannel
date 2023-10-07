use crate::Error;

use sqlx::PgPool;

pub async fn init(db: &PgPool) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS threads(
            threadid BIGINT NOT NULL,
            title TEXT NOT NULL,
            lastupdate BIGINT NOT NULL,
            PRIMARY KEY(threadid)
        );",
    )
    .execute(db)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS lastupdate_index ON threads (lastupdate);")
        .execute(db)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts(
            threadid BIGINT NOT NULL,
            name TEXT NOT NULL,
            mail TEXT NOT NULL,
            date BIGINT NOT NULL,
            id TEXT NOT NULL,
            body TEXT NOT NULL,
            PRIMARY KEY(date)
        );",
    )
    .execute(db)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS threadid_index ON posts (threadid);")
        .execute(db)
        .await?;

    let _ = sqlx::query("INSERT INTO threads(threadid, title, lastupdate) VALUES (1000000000, 'Hello World!', 1000000000);")
        .execute(db)
        .await;
    let _ = sqlx::query(
        "INSERT INTO posts(threadid, name, mail, date, id, body)
                VALUES (1000000000, '</b>System', '', 65536000000000, 'System', 'おめでとうございます<br>あなたの掲示板が立ちました!')",
    )
    .execute(db)
    .await;

    Ok(())
}
