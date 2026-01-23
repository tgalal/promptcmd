use std::{path::PathBuf, sync::{Arc, Mutex}};

use chrono::{Duration, Utc};
use rusqlite::{params, params_from_iter, Connection};
use thiserror::Error;
use log::debug;

use crate::stats::{store::{FetchError, LogError, LogRecord, StatsStore, SummaryItem}, DB_NAME};

pub struct RusqliteStore {
    conn: Arc<Mutex<Connection>>
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
        debug!("DB Path: {}", db_path);

        let mut conn = Connection::open(db_path)?;
        let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if version == 1 {
            conn.pragma_update(None, "journal_mode", "WAL")?;
        }

        let tx = conn.transaction()?;

        if version < 1 {
            debug!("Applying v1 migration");
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

        if version < 2 {
            // applied before the transaction
            debug!("Applying v2 migration");
        }

        if version < 3 {
            debug!("Applying v3 migration");
            tx.execute(
                "ALTER TABLE logs ADD COLUMN cache_key INTEGER",
                []
            )?;
            tx.execute(
                "CREATE INDEX idx_cache_composite ON logs (cache_key, created);",
                []
            )?;
        }

        tx.pragma_update(None, "user_version", 3)?;

        tx.commit()?;

        Ok(RusqliteStore { conn: Arc::new(Mutex::new(conn)) })
    }

}

impl StatsStore for RusqliteStore {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        self.conn.lock().unwrap().execute(
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
                created,
                cache_key
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)", params![
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
                &record.created.to_rfc3339(),
                &record.cache_key
            ]
        ).map_err(|e| LogError::GeneralError(e.to_string()))?;

        Ok(())
    }

    fn cached(&self, cache_key: i64, ttl: u32) -> Result<Option<LogRecord>, FetchError> {
        let cutoff = (Utc::now() - Duration::seconds(ttl.into())).to_rfc3339();
        let sql = String::from(
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
                created,
                cache_key
            FROM logs WHERE cache_key = ?1 AND created > ?2 ORDER BY id DESC LIMIT 1
        ");

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql)
            .map_err(|err| FetchError::GeneralError(err.to_string()))?;


        let result = stmt.query_one(params![cache_key, cutoff], |row| {
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
                    created: row.get(10)?,
                    cache_key: row.get(11)?
                }
            )
        });

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(FetchError::GeneralError(err.to_string()))
        }
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
                created,
                cache_key
            FROM logs
        ");

        let mut params: Vec<String> = Vec::new();

        if let Some(last) = last {
            sql.push_str(" ORDER BY id DESC LIMIT ?");
            params.push(last.to_string());
        }
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql)
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
                    created: row.get(10)?,
                    cache_key: row.get(11)?
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

        let mut group_by: Vec<&'static str> = Vec::new();

        if let Some(provider) = provider {
            sql.push_str(" AND provider = ?");
            group_by.push("provider");
            params.push(provider);
        }

        if let Some(model) = model {
            sql.push_str(" AND model = ?");
            group_by.push("model");
            params.push(model);
        }

        if let Some(variant) = variant {
            sql.push_str(" AND variant = ?");
            group_by.push("variant");
            params.push(variant);
        }

        if let Some(group) = group {
            sql.push_str(" AND `group` = ?");
            group_by.push("`group`");
            params.push(group);
        }

        if let Some(success) = success {
            sql.push_str(" AND success = ?");
            let success = if success { "1" } else { "0" };
            params.push(success.to_string());
        }

        sql.push_str(" GROUP BY ");
        if group_by.is_empty() {
            sql.push_str("provider, model");
        } else {
            sql.push_str(&group_by.join(", "));
        }

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql)
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
