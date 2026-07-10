use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, ToSql};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("database error")]
    Sql(#[from] rusqlite::Error),
    #[error("application data directory unavailable")]
    DataDirectoryUnavailable,
    #[error("filesystem error")]
    Io(#[from] std::io::Error),
    #[error("invalid input")]
    InvalidInput,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: String,
    pub content: String,
    pub content_type: String,
    pub source_application: Option<String>,
    pub window_title: Option<String>,
    pub created_at: String,
    pub last_copied_at: String,
    pub copy_count: i64,
    pub pinned: bool,
    pub pin_order: Option<i64>,
    pub label: Option<String>,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub sensitive: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub created_at: String,
    pub item_count: i64,
}
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    pub pinned: Option<bool>,
    pub group_id: Option<String>,
    pub content_type: Option<String>,
    pub source_application: Option<String>,
    pub recent: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionSettings {
    pub max_history_size: i64,
    pub delete_unpinned_after_days: i64,
    pub excluded_content_types: Vec<String>,
    pub excluded_applications: Vec<String>,
}
impl Default for RetentionSettings {
    fn default() -> Self {
        Self {
            max_history_size: 5000,
            delete_unpinned_after_days: 90,
            excluded_content_types: vec![],
            excluded_applications: vec![],
        }
    }
}
pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open() -> Result<Self, DatabaseError> {
        let mut path = std::env::var_os("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .ok_or(DatabaseError::DataDirectoryUnavailable)?;
        path.push("ClipboardPlus");
        fs::create_dir_all(&path)?;
        path.push("clipboard-plus.sqlite3");
        let connection = Connection::open(path)?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);")?;
        for (version, sql) in [
            (1, include_str!("../../migrations/0001_initial.sql")),
            (2, include_str!("../../migrations/0002_milestone2.sql")),
            (3, include_str!("../../migrations/0003_default_groups.sql")),
        ] {
            let applied: Option<i64> = connection
                .query_row(
                    "SELECT version FROM schema_migrations WHERE version=?1",
                    [version],
                    |r| r.get(0),
                )
                .optional()?;
            if applied.is_none() {
                let tx = connection.unchecked_transaction()?;
                tx.execute_batch(sql)?;
                tx.execute(
                    "INSERT INTO schema_migrations(version,applied_at) VALUES(?1,?2)",
                    params![version, Utc::now().to_rfc3339()],
                )?;
                tx.commit()?;
            }
        }
        Ok(Self { connection })
    }
    pub fn capture_text(
        &mut self,
        content: &str,
        context: crate::clipboard::windows::CaptureContext,
    ) -> Result<(), DatabaseError> {
        let settings = self.settings()?;
        let content_type = detect_content_type(content).to_string();
        if settings.excluded_content_types.contains(&content_type)
            || context.source_application.as_ref().is_some_and(|app| {
                settings
                    .excluded_applications
                    .iter()
                    .any(|excluded| excluded.eq_ignore_ascii_case(app))
            })
        {
            return Ok(());
        }
        let hash = hash(content);
        let now = Utc::now().to_rfc3339();
        let existing: Option<String> = self
            .connection
            .query_row(
                "SELECT id FROM clipboard_items WHERE content_hash=?1",
                [&hash],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(id) = existing {
            self.connection.execute("UPDATE clipboard_items SET last_copied_at=?1, copy_count=copy_count+1, source_application=COALESCE(?2,source_application), window_title=COALESCE(?3,window_title) WHERE id=?4",params![now,context.source_application,context.window_title,id])?;
        } else {
            let group_id = automatic_group_id(&content_type, context.source_application.as_deref());
            self.connection.execute("INSERT INTO clipboard_items(id,content,content_type,source_application,window_title,created_at,last_copied_at,copy_count,pinned,group_id,sensitive,backup_eligible,content_hash) VALUES(?1,?2,?3,?4,?5,?6,?6,1,0,?7,0,1,?8)",params![Uuid::new_v4().to_string(),content,content_type,context.source_application,context.window_title,now,group_id,hash])?;
        }
        self.cleanup()?;
        Ok(())
    }
    pub fn list(
        &self,
        query: &str,
        filters: &Filters,
        limit: i64,
    ) -> Result<Vec<ClipboardItem>, DatabaseError> {
        let fts = fts_query(query);
        let mut sql=String::from("SELECT i.id,i.content,i.content_type,i.source_application,i.window_title,i.created_at,i.last_copied_at,i.copy_count,i.pinned,i.pin_order,i.label,i.group_id,g.name,i.sensitive FROM clipboard_items i LEFT JOIN groups g ON g.id=i.group_id");
        let mut values: Vec<Box<dyn ToSql>> = vec![];
        if !fts.is_empty() {
            sql.push_str(
                " JOIN clipboard_items_fts f ON f.rowid=i.rowid WHERE clipboard_items_fts MATCH ?",
            );
            values.push(Box::new(fts));
        } else {
            sql.push_str(" WHERE 1=1");
        }
        if let Some(pinned) = filters.pinned {
            sql.push_str(" AND i.pinned=?");
            values.push(Box::new(pinned));
        }
        if let Some(group) = &filters.group_id {
            sql.push_str(" AND i.group_id=?");
            values.push(Box::new(group.clone()));
        }
        if let Some(kind) = &filters.content_type {
            sql.push_str(" AND i.content_type=?");
            values.push(Box::new(kind.clone()));
        }
        if let Some(app) = &filters.source_application {
            sql.push_str(" AND i.source_application=?");
            values.push(Box::new(app.clone()));
        }
        if filters.recent == Some(true) {
            sql.push_str(" AND i.last_copied_at >= datetime('now','-7 days')");
        }
        sql.push_str(
            " ORDER BY i.pinned DESC, i.pin_order ASC NULLS LAST, i.last_copied_at DESC LIMIT ?",
        );
        values.push(Box::new(limit));
        let refs = values.iter().map(|v| v.as_ref()).collect::<Vec<_>>();
        let mut stmt = self.connection.prepare(&sql)?;
        let rows = stmt.query_map(refs.as_slice(), map_item)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::from)
    }
    pub fn groups(&self) -> Result<Vec<Group>, DatabaseError> {
        let mut stmt=self.connection.prepare("SELECT g.id,g.name,g.kind,g.created_at,COUNT(i.id) FROM groups g LEFT JOIN clipboard_items i ON i.group_id=g.id GROUP BY g.id ORDER BY g.name COLLATE NOCASE")?;
        let rows = stmt.query_map([], |r| {
            Ok(Group {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                created_at: r.get(3)?,
                item_count: r.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::from)
    }
    pub fn create_group(&mut self, name: &str) -> Result<Group, DatabaseError> {
        let name = name.trim();
        if name.is_empty() || name.len() > 60 {
            return Err(DatabaseError::InvalidInput);
        }
        let group = Group {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            kind: "manual".into(),
            created_at: Utc::now().to_rfc3339(),
            item_count: 0,
        };
        self.connection.execute(
            "INSERT INTO groups(id,name,kind,created_at) VALUES(?1,?2,?3,?4)",
            params![group.id, group.name, group.kind, group.created_at],
        )?;
        Ok(group)
    }
    pub fn set_group(&mut self, id: &str, group_id: Option<&str>) -> Result<(), DatabaseError> {
        if let Some(group) = group_id {
            let exists: Option<String> = self
                .connection
                .query_row("SELECT id FROM groups WHERE id=?1", [group], |r| r.get(0))
                .optional()?;
            if exists.is_none() {
                return Err(DatabaseError::InvalidInput);
            }
        }
        self.connection.execute(
            "UPDATE clipboard_items SET group_id=?1 WHERE id=?2",
            params![group_id, id],
        )?;
        Ok(())
    }
    pub fn pin(&mut self, id: &str, pinned: bool) -> Result<(), DatabaseError> {
        if pinned {
            let order: i64 = self.connection.query_row(
                "SELECT COALESCE(MAX(pin_order),0)+1 FROM clipboard_items WHERE pinned=1",
                [],
                |r| r.get(0),
            )?;
            self.connection.execute(
                "UPDATE clipboard_items SET pinned=1,pin_order=?1 WHERE id=?2",
                params![order, id],
            )?;
        } else {
            self.connection.execute(
                "UPDATE clipboard_items SET pinned=0,pin_order=NULL WHERE id=?1",
                [id],
            )?;
        }
        Ok(())
    }
    pub fn reorder_pins(&mut self, ids: &[String]) -> Result<(), DatabaseError> {
        let tx = self.connection.transaction()?;
        for (index, id) in ids.iter().enumerate() {
            if tx.execute(
                "UPDATE clipboard_items SET pin_order=?1 WHERE id=?2 AND pinned=1",
                params![index as i64 + 1, id],
            )? != 1
            {
                return Err(DatabaseError::InvalidInput);
            }
        }
        tx.commit()?;
        Ok(())
    }
    pub fn update(
        &mut self,
        id: &str,
        content: Option<&str>,
        label: Option<&str>,
    ) -> Result<(), DatabaseError> {
        if let Some(content) = content {
            if content.trim().is_empty() || content.len() > 1_000_000 {
                return Err(DatabaseError::InvalidInput);
            }
            self.connection.execute(
                "UPDATE clipboard_items SET content=?1,content_hash=?2,content_type=?3 WHERE id=?4",
                params![content, hash(content), detect_content_type(content), id],
            )?;
        }
        if let Some(label) = label {
            if label.len() > 120 {
                return Err(DatabaseError::InvalidInput);
            }
            self.connection.execute(
                "UPDATE clipboard_items SET label=?1 WHERE id=?2",
                params![label.trim(), id],
            )?;
        }
        Ok(())
    }
    pub fn content_by_id(&self, id: &str) -> Result<Option<String>, DatabaseError> {
        self.connection
            .query_row(
                "SELECT content FROM clipboard_items WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(DatabaseError::from)
    }
    pub fn delete(&mut self, id: &str) -> Result<(), DatabaseError> {
        self.connection
            .execute("DELETE FROM clipboard_items WHERE id=?1", [id])?;
        Ok(())
    }
    pub fn clear(&mut self) -> Result<(), DatabaseError> {
        self.connection
            .execute("DELETE FROM clipboard_items WHERE pinned=0", [])?;
        Ok(())
    }
    pub fn settings(&self) -> Result<RetentionSettings, DatabaseError> {
        let get = |key: &str| {
            self.connection
                .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                    r.get::<_, String>(0)
                })
                .optional()
        };
        let max = get("max_history_size")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(5000);
        let days = get("delete_unpinned_after_days")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(90);
        let types = get("excluded_content_types")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default();
        let apps = get("excluded_applications")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default();
        Ok(RetentionSettings {
            max_history_size: max,
            delete_unpinned_after_days: days,
            excluded_content_types: types,
            excluded_applications: apps,
        })
    }
    pub fn save_settings(&mut self, value: RetentionSettings) -> Result<(), DatabaseError> {
        if !(100..=100_000).contains(&value.max_history_size)
            || !(1..=3650).contains(&value.delete_unpinned_after_days)
        {
            return Err(DatabaseError::InvalidInput);
        }
        for (key, value) in [
            ("max_history_size", value.max_history_size.to_string()),
            (
                "delete_unpinned_after_days",
                value.delete_unpinned_after_days.to_string(),
            ),
            (
                "excluded_content_types",
                serde_json::to_string(&value.excluded_content_types)
                    .map_err(|_| DatabaseError::InvalidInput)?,
            ),
            (
                "excluded_applications",
                serde_json::to_string(&value.excluded_applications)
                    .map_err(|_| DatabaseError::InvalidInput)?,
            ),
        ] {
            self.connection.execute("INSERT INTO settings(key,value,updated_at) VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at",params![key,value,Utc::now().to_rfc3339()])?;
        }
        self.cleanup()?;
        Ok(())
    }
    fn cleanup(&mut self) -> Result<(), DatabaseError> {
        let s = self.settings()?;
        self.connection.execute(
            "DELETE FROM clipboard_items WHERE pinned=0 AND last_copied_at < datetime('now', ?1)",
            [format!("-{} days", s.delete_unpinned_after_days)],
        )?;
        self.connection.execute("DELETE FROM clipboard_items WHERE id IN (SELECT id FROM clipboard_items WHERE pinned=0 ORDER BY last_copied_at DESC LIMIT -1 OFFSET ?1)",[s.max_history_size])?;
        Ok(())
    }
}
fn map_item(r: &rusqlite::Row<'_>) -> rusqlite::Result<ClipboardItem> {
    Ok(ClipboardItem {
        id: r.get(0)?,
        content: r.get(1)?,
        content_type: r.get(2)?,
        source_application: r.get(3)?,
        window_title: r.get(4)?,
        created_at: r.get(5)?,
        last_copied_at: r.get(6)?,
        copy_count: r.get(7)?,
        pinned: r.get(8)?,
        pin_order: r.get(9)?,
        label: r.get(10)?,
        group_id: r.get(11)?,
        group_name: r.get(12)?,
        sensitive: r.get(13)?,
    })
}
fn detect_content_type(c: &str) -> &'static str {
    if url::Url::parse(c.trim()).is_ok() {
        "url"
    } else if c.contains('{') || c.contains("=>") || c.contains("function ") || c.contains("const ")
    {
        "code"
    } else {
        "text"
    }
}
fn hash(c: &str) -> String {
    format!("{:x}", Sha256::digest(c.as_bytes()))
}
fn fts_query(q: &str) -> String {
    q.split_whitespace()
        .filter(|t| {
            t.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        })
        .map(|t| format!("\"{t}\"*", t = t.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn automatic_group_id(
    content_type: &str,
    source_application: Option<&str>,
) -> Option<&'static str> {
    if content_type == "url" {
        Some("system-links")
    } else if content_type == "code"
        || source_application.is_some_and(|app| app.eq_ignore_ascii_case("Code.exe"))
    {
        Some("system-code")
    } else {
        None
    }
}
