use std::path::PathBuf;

use rusqlite::{params_from_iter, Connection};
use thiserror::Error;

use crate::stats::{store::{FetchError, LogError, LogRecord, StatsStore, SummaryItem}, DB_NAME};

pub struct RusqliteStore {
    conn: Connection
}

#[derive(Debug, Error)]
pub enum RusqliteError {
    #[error("Rusqlite Error")]
    MyError(#[from] rusqlite::Error),

    #[error("Could not init store: {0}")]
    GeneralError(String),
}

impl RusqliteStore {
    pub fn new(path: PathBuf) -> Result<Self, RusqliteError> {
        let db_path = path.join(DB_NAME).to_string_lossy().to_string();

        let mut conn = Connection::open(db_path)?;
        let tx = conn.transaction()?;
        let version: i32 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if version < 1 {
            tx.execute_batch(
                "CREATE TABLE logs (
                    id INTEGER PRIMARY KEY,
                    promptname TEXT NOT NULL,
                    provider TEXT NOT NULL,
                    model TEXT NOT NULL,
                    prompt_tokens INTEGER NOT NULL DEFAULT 0,
                    completion_tokens INTEGER NOT NULL DEFAULT 0,
                    result TEXT,
                    success INTEGER NOT NULL,
                    time_taken INTEGER NOT NULL,
                    created TEXT NOT NULL
                );"
            )?;
        }
        tx.pragma_update(None, "user_version", 1)?;

        tx.commit()?;

        Ok(RusqliteStore { conn })
    }

}

impl StatsStore for RusqliteStore {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        self.conn.execute(
            "INSERT INTO logs (
                promptname,
                provider,
                model,
                prompt_tokens,
                completion_tokens,
                result,
                success,
                time_taken,
                created
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", [
                &record.promptname,
                &record.provider,
                &record.model,
                &record.prompt_tokens.to_string(),
                &record.completion_tokens.to_string(),
                &record.result,
                &record.success.to_string(),
                &record.time_taken.to_string(),
                &record.created.to_rfc3339()
            ]
        ).map_err(|e| LogError::GeneralError(e.to_string()))?;

        Ok(())
    }

    fn all(&self) -> Result<Vec<LogRecord>, FetchError> {
        let mut stmt = self.conn.prepare(
            "SELECT
                promptname,
                provider,
                model,
                prompt_tokens,
                completion_tokens,
                result,
                success,
                time_taken,
                created
            FROM logs
        ").map_err(|err| FetchError::GeneralError(err.to_string()))?;

        let records = stmt.query_map([], |row| {
            Ok(
                LogRecord {
                    promptname: row.get(0)?,
                    provider: row.get(1)?,
                    model: row.get(2)?,
                    prompt_tokens: row.get(3)?,
                    completion_tokens: row.get(4)?,
                    result: row.get(5)?,
                    success: row.get(6)?,
                    time_taken: row.get(7)?,
                    created: row.get(8)?
                }
            )
        }).map_err(|err| FetchError::GeneralError(err.to_string()))?;

        let result: Vec<LogRecord> = records.filter_map(Result::ok).collect();
        Ok(result)
    }

    fn summary(&self, provider: Option<String>, model: Option<String>) -> Result<Vec<SummaryItem>, FetchError> {
        let mut sql = String::from(
            "SELECT
                provider,
                model,
                COUNT(*),
                SUM(prompt_tokens),
                SUM(completion_tokens),
                COALESCE(SUM(completion_tokens) * 1.0 / SUM(time_taken), 0)
            FROM logs WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(provider) = provider {
            sql.push_str(" AND provider = ?");
            params.push(provider);
        }

        if let Some(model) = model {
            sql.push_str(" AND model = ?");
            params.push(model);
        }

        sql.push_str(" GROUP BY provider, model ORDER BY provider, model");

        let mut stmt = self.conn.prepare(&sql)
            .map_err(|err| FetchError::GeneralError(err.to_string()))?;

        let records = stmt.query_map(params_from_iter(params.iter()), |row| {
            Ok(
                SummaryItem {
                    provider: row.get(0)?,
                    model: row.get(1)?,
                    count: row.get(2)?,
                    prompt_tokens: row.get(3)?,
                    completion_tokens: row.get(4)?,
                    tps: row.get::<_, f64>(5)? as u32
                }
            )
        }).map_err(|err| FetchError::GeneralError(err.to_string()))?;

        //let result: Vec<SummaryItem> = records.filter_map(Result::ok).collect();
        let result: Result<Vec<_>, _> = records.collect();

        result.map_err(|err| FetchError::GeneralError(err.to_string()))

    }

}
