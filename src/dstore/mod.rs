use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, Read, Seek, SeekFrom, Write};

use std::path::Path;
use std::result;
use uuid::Uuid;

mod error;
mod handler;
mod json;
mod status;
mod store;

use handler::Handler;
use status::Status;
use store::{DStoreHashMap, Document, JsonDocument, Store};
pub type Result<T> = result::Result<T, error::DStoreError>;

#[derive(Debug)]
pub struct DStore {
    handler: Handler,
    store: Store,
}

impl DStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<DStore> {
        let mut buf = Vec::new();
        let handler = Handler::new(path)?;
        //TODO IMPROVE READ LINE BY LINE (split into a new function)
        handler
            .file
            .try_lock()
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();

        let mut map: DStoreHashMap = HashMap::new();
        println!("[DStore] Parsing data into memory");

        for (_index, line) in buf.lines().enumerate() {
            let content = &line.unwrap();
            let json_doc: JsonDocument = serde_json::from_str(content)?;
            let _id = match &json_doc._id.as_str() {
                Some(_id) => Uuid::parse_str(_id).unwrap(),
                None => panic!("ERR: Wrong Uuid format!"),
            };
            let doc = Document {
                data: json_doc.data,
                status: Status::Saved,
            };
            map.insert(_id, doc);
        }
        println!("[DStore] Up & running");

        Ok(Self {
            handler: handler,
            store: Store::new(map)?,
        })
    }

    //TODO data to be json
    pub fn insert(&mut self, data: String) -> Value {
        self.store.insert(data).unwrap()
    }

    pub fn find_by_id(&self, id: &Value) -> Value {
        self.store.find_by_id(id).unwrap()
    }

    pub fn find(&self, data: String) -> Value {
        self.store.find(data).unwrap()
    }

    pub fn get(&self) {
        self.store.get().unwrap()
    }

    pub fn persist(&mut self) -> Result<()> {
        let mut file = self.handler.file.lock()?;
        let docs_to_save = self.store.format_jsondocs();
        file.seek(SeekFrom::End(0))?;
        file.write_all(&docs_to_save)?;
        file.sync_all()?;
        Ok(())
    }
}
