use heed::types::{Bytes, Str};
use heed::{Database, Env, EnvOpenOptions};
use std::sync::{Arc, OnceLock};

pub struct ImageDb {
    env: Env,
    db: Database<Str, Bytes>,
}

impl ImageDb {
    pub fn new() -> Option<Self> {
        let mut path = std::env::current_exe().unwrap_or_default();
        path.pop();
        path.push("sticky_images_db");
        let _ = std::fs::create_dir_all(&path);

        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(100 * 1024 * 1024) // 100MB
                .max_dbs(1)
                .open(path)
                .ok()?
        };

        let mut wtxn = env.write_txn().ok()?;
        let db = env.create_database(&mut wtxn, Some("images")).ok()?;
        wtxn.commit().ok()?;

        Some(Self { env, db })
    }

    pub fn get_image(&self, key: &str) -> Option<Vec<u8>> {
        let rtxn = self.env.read_txn().ok()?;
        self.db.get(&rtxn, key).ok().flatten().map(|b| b.to_vec())
    }

    #[allow(dead_code)]
    pub fn put_image(&self, key: &str, data: &[u8]) -> Option<()> {
        let mut wtxn = self.env.write_txn().ok()?;
        self.db.put(&mut wtxn, key, data).ok()?;
        wtxn.commit().ok()?;
        Some(())
    }
}

pub fn global_db() -> Option<Arc<ImageDb>> {
    static DB: OnceLock<Option<Arc<ImageDb>>> = OnceLock::new();
    DB.get_or_init(|| ImageDb::new().map(Arc::new)).clone()
}
