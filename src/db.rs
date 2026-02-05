use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Prompt {
    pub id: i64,
    pub text: String,
    pub source_file: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL UNIQUE,
                source_file TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert_prompt(&self, text: &str, source_file: Option<&str>) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO prompts (text, source_file) VALUES (?1, ?2)",
            params![text, source_file],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_prompts(&self, prompts: &[(String, Option<String>)]) -> Result<usize> {
        let mut count = 0;
        for (text, source) in prompts {
            let rows = self.conn.execute(
                "INSERT OR IGNORE INTO prompts (text, source_file) VALUES (?1, ?2)",
                params![text, source.as_deref()],
            )?;
            count += rows;
        }
        Ok(count)
    }

    pub fn delete_prompt(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM prompts WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_prompt(&self, id: i64, new_text: &str) -> Result<bool> {
        let rows = self.conn.execute(
            "UPDATE prompts SET text = ?1 WHERE id = ?2",
            params![new_text, id],
        )?;
        Ok(rows > 0)
    }

    pub fn get_all(&self) -> Result<Vec<Prompt>> {
        let mut stmt = self.conn.prepare("SELECT id, text, source_file FROM prompts ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Prompt {
                id: row.get(0)?,
                text: row.get(1)?,
                source_file: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn search(&self, query: &str) -> Result<Vec<Prompt>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, text, source_file FROM prompts WHERE text LIKE ?1 ORDER BY id"
        )?;
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(Prompt {
                id: row.get(0)?,
                text: row.get(1)?,
                source_file: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn count(&self) -> Result<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))
    }
}
