use crate::model::SQLTable;
use anyhow::Result;

pub fn create_table(conf: &SQLTable) -> Result<String> {
    Ok(conf.name.clone())
}
