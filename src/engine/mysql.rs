use crate::model::{FieldType, SQLTable};
use anyhow::Result;
use std::fmt::Write;
use std::str::FromStr;

pub fn create_table(conf: &SQLTable) -> Result<String> {
    let mut sql = String::new();
    let f = &mut sql;
    write!(f, "CREATE TABLE IF NOT EXISTS `{}` ( ", conf.name)?;
    for (index, field) in conf.fields.iter().enumerate() {
        let ftype_parsed = FieldType::from_str(&field.f_type)?;
        let field_type = field_type_mysql(&ftype_parsed);
        write!(f, "`{}` {}", field.name, field_type)?;
        if let Some(v) = &field.value {
            if v.increment.unwrap_or_default() {
                write!(f, " AUTO_INCREMENT")?;
            }
            if v.not_null.unwrap_or_default() {
                write!(f, " NOT NULL")?;
            } else {
                write!(f, " NULL")?;
            }

            if let Some(v) = &v.default {
                match ftype_parsed {
                    FieldType::Char(_)
                    | FieldType::FixedString(_)
                    | FieldType::Varchar(_)
                    | FieldType::Text
                    | FieldType::Date
                    | FieldType::DateTime
                    | FieldType::DateTime64
                    | FieldType::Time
                    | FieldType::Timestamp
                    | FieldType::Json
                    | FieldType::IPv4
                    | FieldType::IPv6 => {
                        write!(f, " DEFAULT '{}'", v)?;
                    }
                    _ => {
                        write!(f, " DEFAULT {}", v)?;
                    }
                }
            }
        }

        if index.lt(&(conf.fields.len() - 1)) {
            write!(f, ", ")?;
        }
    }
    write!(f, ")")?;
    Ok(sql)
}

pub fn field_type_mysql(field_type: &FieldType) -> String {
    match field_type {
        FieldType::Char(len) => format!("char({})", len),
        FieldType::Varchar(len) => format!("varchar({})", len),
        FieldType::Text => "text".to_string(),
        FieldType::Int8 => "tinyint".to_string(),
        FieldType::Int16 => "smallint".to_string(),
        FieldType::Int32 => "int".to_string(),
        FieldType::Int64 => "bigint".to_string(),
        FieldType::Float16 => "float".to_string(),
        FieldType::Float32 => "float".to_string(),
        FieldType::Float64 => "double".to_string(),
        FieldType::Double => "double".to_string(),
        FieldType::Decimal(m, n) => format!("decimal({},{})", m, n),
        FieldType::Boolean => "tinyint(1)".to_string(),
        FieldType::Date => "date".to_string(),
        FieldType::Time => "time".to_string(),
        FieldType::DateTime => "datetime".to_string(),
        FieldType::DateTime64 => "datetime".to_string(),
        FieldType::Timestamp => "timestamp".to_string(),
        FieldType::Json => "json".to_string(),
        FieldType::VarBinary => "blob".to_string(),
        FieldType::FixedString(len) => format!("char({})", len),
        FieldType::UInt8 => "tinyint unsigned".to_string(),
        FieldType::UInt16 => "smallint unsigned".to_string(),
        FieldType::UInt32 => "int unsigned".to_string(),
        FieldType::UInt64 => "bigint unsigned".to_string(),
        FieldType::UInt128 => "decimal(39,0)".to_string(),
        FieldType::UInt256 => "decimal(78,0)".to_string(),
        FieldType::Int128 => "decimal(39,0)".to_string(),
        FieldType::Int256 => "decimal(78,0)".to_string(),
        FieldType::IPv4 => "varchar(15)".to_string(),
        FieldType::IPv6 => "varchar(39)".to_string(),
        FieldType::Array(_) | FieldType::Struct | FieldType::Null => "json".to_string(),
    }
}
