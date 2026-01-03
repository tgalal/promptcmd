use std::path::PathBuf;

use rusqlite::{params, params_from_iter, Connection};
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
                    variant TEXT,
                    `group` TEXT,
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
                variant,
                `group`,
                prompt_tokens,
                completion_tokens,
                result,
                success,
                time_taken,
                created
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)", params![
                &record.promptname,
                &record.provider,
                &record.model,
                record.variant,
                record.group,
                record.prompt_tokens,
                record.completion_tokens,
                &record.result,
                record.success,
                record.time_taken,
                &record.created.to_rfc3339()
            ]
        ).map_err(|e| LogError::GeneralError(e.to_string()))?;

        Ok(())
    }

    fn records(&self, last: Option<u32>) -> Result<Vec<LogRecord>, FetchError> {
        let mut sql = String::from(
            "SELECT
                promptname,
                provider,
                model,
                variant,
                `group`,
                prompt_tokens,
                completion_tokens,
                result,
                success,
                time_taken,
                created
            FROM logs
        ");

        let mut params: Vec<String> = Vec::new();

        if let Some(last) = last {
            sql.push_str(" ORDER BY id DESC LIMIT ?");
            params.push(last.to_string());
        }
        let mut stmt = self.conn.prepare(&sql)
            .map_err(|err| FetchError::GeneralError(err.to_string()))?;

        let records = stmt.query_map(
            params_from_iter(params.iter()), |row| {
            Ok(
                LogRecord {
                    promptname: row.get(0)?,
                    provider: row.get(1)?,
                    model: row.get(2)?,
                    variant: row.get(3)?,
                    group: row.get(4)?,
                    prompt_tokens: row.get(5)?,
                    completion_tokens: row.get(6)?,
                    result: row.get(7)?,
                    success: row.get(8)?,
                    time_taken: row.get(9)?,
                    created: row.get(10)?
                }
            )
        }).map_err(|err| FetchError::GeneralError(err.to_string()))?;

        //let result: Vec<SummaryItem> = records.filter_map(Result::ok).collect();
        let result: Result<Vec<_>, _> = records.collect();

        result.map_err(|err| FetchError::GeneralError(err.to_string()))
    }

    fn summary(&self,
        provider: Option<String>,
        model: Option<String>,
        variant: Option<String>,
        group: Option<String>,
        success: Option<bool>
    ) -> Result<Vec<SummaryItem>, FetchError> {
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

        if let Some(variant) = variant {
            sql.push_str(" AND variant = ?");
            params.push(variant);
        }

        if let Some(group) = group {
            sql.push_str(" AND group = ?");
            params.push(group);
        }

        if let Some(success) = success {
            sql.push_str(" AND success = ?");
            let success = if success { "1" } else { "0" };
            params.push(success.to_string());
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
