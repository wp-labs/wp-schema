use crate::model::{ClickhouseEngine, FieldType, SQLTable};
use anyhow::Result;
use std::fmt::Write;
use std::str::FromStr;

pub fn create_table(conf: &SQLTable) -> Result<String> {
    let mut sql = String::new();
    let f = &mut sql;
    let mut primary_key = Vec::new();
    write!(f, "CREATE TABLE IF NOT EXISTS `{}` ( ", conf.name)?;
    for (index, field) in conf.fields.iter().enumerate() {
        let ft = FieldType::from_str(&field.f_type)?;
        write!(f, "{} {}", field.name, field_type(&ft))?;
        if let Some(v) = &field.value {
            if v.not_null.unwrap_or_default() {
                write!(f, " NOT NULL")?;
            } else {
                write!(f, " NULL")?;
            }
            if let Some(value) = &v.default {
                match ft {
                    FieldType::Char(_)
                    | FieldType::FixedString(_)
                    | FieldType::Varchar(_)
                    | FieldType::Date
                    | FieldType::DateTime
                    | FieldType::DateTime64 => {
                        write!(f, " DEFAULT '{}'", value)?;
                    }
                    _ => {
                        write!(f, " DEFAULT {}", value)?;
                    }
                }
            }
        }
        if index.lt(&(conf.fields.len() - 1)) {
            write!(f, ", ")?;
        }
        // collect primary key (collapsed nested ifs per clippy)
        if let Some(idx) = &field.index
            && matches!(idx, crate::model::IndexType::Primary)
        {
            primary_key.push(field.name.clone());
        }
    }
    let engine = conf
        .table_engine
        .as_ref()
        .unwrap_or(&ClickhouseEngine::MergeTree);
    write!(f, ") ENGINE = {}", engine)?;
    match engine {
        ClickhouseEngine::MergeTree
        | ClickhouseEngine::ReplacingMergeTree
        | ClickhouseEngine::SummingMergeTree => {
            if !primary_key.is_empty() {
                write!(f, " PRIMARY KEY ({})", primary_key.join(","))?;
            }
            if let Some(order_by) = &conf.order_by {
                let order_by = [primary_key, order_by.clone()].concat();
                write!(f, " ORDER BY ({})", order_by.join(","))?;
            }
        }
        _ => {}
    }
    Ok(sql)
}

pub fn field_type(field_type: &FieldType) -> String {
    match field_type {
        FieldType::UInt8 => "UInt8".to_string(),
        FieldType::UInt16 => "UInt16".to_string(),
        FieldType::UInt32 => "UInt32".to_string(),
        FieldType::UInt64 => "UInt64".to_string(),
        FieldType::UInt128 => "UInt128".to_string(),
        FieldType::UInt256 => "UInt256".to_string(),
        FieldType::Int8 => "Int8".to_string(),
        FieldType::Int16 => "Int16".to_string(),
        FieldType::Int32 => "Int32".to_string(),
        FieldType::Int64 => "Int64".to_string(),
        FieldType::Int128 => "Int128".to_string(),
        FieldType::Int256 => "Int256".to_string(),
        FieldType::Float32 => "Float32".to_string(),
        FieldType::Double => "Float64".to_string(),
        FieldType::Decimal(m, n) => format!("Decimal({}, {})", m, n),
        FieldType::Boolean => "Bool".to_string(),
        FieldType::Varchar(_) | FieldType::Char(_) | FieldType::Text => "String".to_string(),
        FieldType::FixedString(len) => format!("FixedString({})", len),
        FieldType::Date => "Date".to_string(),
        FieldType::DateTime => "DateTime".to_string(),
        FieldType::DateTime64 => "DateTime64".to_string(),
        FieldType::Json => "Json".to_string(),
        FieldType::IPv4 => "IPv4".to_string(),
        FieldType::IPv6 => "IPv6".to_string(),
        FieldType::Array(t) => format!("Array({})", crate::engine::clickhouse::field_type(t)),
        _ => panic!("{:?} is not available in Clickhouse", field_type),
    }
}
