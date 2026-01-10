use educe::Educe;
use serde::{Deserialize, Serialize};
use wp_error::{ConfError, ConfReason, ConfResult};

// ----------------- ClickHouse Engine -----------------

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug, Default)]
pub enum ClickhouseEngine {
    #[default]
    MergeTree,
    ReplacingMergeTree,
    SummingMergeTree,
    AggregatingMergeTree,
    CollapsingMergerTree(i8),
    VersionedCollapsingMergeTree(i8, u8),
    TinyLog,
    StripeLog,
    Log,
}

impl std::fmt::Display for ClickhouseEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClickhouseEngine::MergeTree => write!(f, "MergeTree()"),
            ClickhouseEngine::ReplacingMergeTree => write!(f, "ReplacingMergeTree()"),
            ClickhouseEngine::SummingMergeTree => write!(f, "SummingMergeTree()"),
            ClickhouseEngine::AggregatingMergeTree => write!(f, "AggregatingMergeTree()"),
            ClickhouseEngine::CollapsingMergerTree(sign) => {
                write!(f, "CollapsingMergeTree({})", sign)
            }
            ClickhouseEngine::VersionedCollapsingMergeTree(sign, version) => {
                write!(f, "VersionedCollapsingMergeTree({},{})", sign, version)
            }
            ClickhouseEngine::TinyLog => write!(f, "TinyLog"),
            ClickhouseEngine::StripeLog => write!(f, "StripeLog"),
            ClickhouseEngine::Log => write!(f, "Log"),
        }
    }
}

impl From<&str> for ClickhouseEngine {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "log" => ClickhouseEngine::Log,
            _ => ClickhouseEngine::MergeTree,
        }
    }
}

// ----------------- SQL Table Model -----------------

#[derive(PartialEq, Deserialize, Serialize, Clone, Debug)]
pub struct ValueConf {
    pub increment: Option<bool>,
    pub default: Option<String>,
    pub not_null: Option<bool>,
}

#[derive(PartialEq, Deserialize, Serialize, Clone, Debug)]
pub enum IndexType {
    Primary,
    Unique,
    Index,
    FullText,
    Spatial,
}

#[derive(PartialEq, Deserialize, Serialize, Clone, Debug)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub f_type: String,
    pub index: Option<IndexType>,
    pub value: Option<ValueConf>,
}

impl Field {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Field {
            name: name.into(),
            f_type: "varchar(255)".to_string(),
            index: None,
            value: None,
        }
    }
    pub fn new2<T: Into<String>>(name: T, table_type: T) -> Self {
        Field {
            name: name.into(),
            f_type: table_type.into(),
            index: None,
            value: None,
        }
    }
    pub fn primary_key<T: Into<String>>(name: T) -> Self {
        Field {
            name: name.into(),
            f_type: "int32".to_string(),
            index: Some(IndexType::Primary),
            value: Some(ValueConf {
                increment: Some(true),
                default: None,
                not_null: Some(true),
            }),
        }
    }
}

#[derive(Educe, PartialEq, Deserialize, Serialize, Clone)]
#[educe(Debug, Default)]
pub struct SQLTable {
    #[serde(rename = "table_name")]
    #[educe(Default = "my_table")]
    pub name: String,
    #[educe(Default(expression = "vec![Field::primary_key(\"id\"), Field::new(\"value\")]"))]
    pub fields: Vec<Field>,
    #[educe(Default(expression = "None"))]
    pub table_engine: Option<ClickhouseEngine>,
    #[educe(Default(expression = "None"))]
    pub order_by: Option<Vec<String>>,
}

// ----------------- FieldType -----------------

use std::str::FromStr;
use winnow::Parser;
use winnow::error::{ContextError, ErrMode};
use winnow::token::take_while;
//use wp_error::config_error::{ConfError, ConfReason, ConfResult};

#[derive(Debug, Eq, PartialEq)]
pub enum FieldType {
    Char(u8),
    Varchar(u32),
    FixedString(usize),
    Text,
    VarBinary,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    UInt256,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int256,
    Float16,
    Float32,
    Float64,
    Double,
    Decimal(usize, usize),
    Boolean,
    Date,
    Time,
    DateTime,
    DateTime64,
    Timestamp,
    Json,
    IPv4,
    IPv6,
    Array(Box<FieldType>),
    Struct,
    Null,
}

impl FromStr for FieldType {
    type Err = ConfError;
    fn from_str(s: &str) -> ConfResult<Self> {
        let f_type = s.to_lowercase();

        // 提取类型名（直到 '(' 为止，至少 1 个字符）
        let (rem, t) = take_while(1.., |c: char| c != '(')
            .parse_peek(f_type.as_str())
            .map_err(|e: ErrMode<ContextError>| {
                ConfError::from(ConfReason::Syntax("解析SQL字段类型失败".into()))
                    .with_detail(e.to_string())
            })?;

        // 解析可选参数：形如 "(10,2)" 或 "(int32)"，仅允许字母/数字/逗号
        let stats = if let Some(rest) = rem.strip_prefix('(') {
            if let Some(end) = rest.find(')') {
                let inner = &rest[..end];
                if inner.chars().all(|c| c.is_ascii_alphanumeric() || c == ',') {
                    inner
                } else {
                    ""
                }
            } else {
                ""
            }
        } else {
            ""
        };
        let field_type = match t {
            "char" => {
                if stats.is_empty() {
                    FieldType::Char(255)
                } else {
                    FieldType::Char(stats.parse().map_err(|e| {
                        ConfError::from(ConfReason::Syntax(format!("期望数字范围为1～255: {}", e)))
                    })?)
                }
            }
            "varchar" => {
                if stats.is_empty() {
                    FieldType::Varchar(255)
                } else {
                    FieldType::Varchar(stats.parse().map_err(|e| {
                        ConfError::from(ConfReason::Syntax(format!("期待大于0的数字: {}", e)))
                    })?)
                }
            }
            "text" => FieldType::Text,
            "uint8" => FieldType::UInt8,
            "uint16" => FieldType::UInt16,
            "uint32" => FieldType::UInt32,
            "uint128" => FieldType::UInt128,
            "uint256" => FieldType::UInt256,
            "int8" => FieldType::Int8,
            "int16" => FieldType::Int16,
            "int32" | "int" => FieldType::Int32,
            "int128" => FieldType::Int128,
            "int256" => FieldType::Int256,
            "float32" => FieldType::Float32,
            "double" => FieldType::Double,
            "decimal" => {
                if stats.is_empty() {
                    FieldType::Decimal(10, 2)
                } else {
                    let d: Vec<&str> = stats.split(',').collect();
                    let m = d[0].parse().map_err(|e| {
                        ConfError::from(ConfReason::Syntax(format!(
                            "期待一个大于0的数字，表示该类型的总共可以存多少为数字: {}",
                            e
                        )))
                    })?;
                    let n = d[1].parse().map_err(|e| {
                        ConfError::from(ConfReason::Syntax(format!(
                            "期待一个大于0的数字，表示该数字可以保留几位数字: {}",
                            e
                        )))
                    })?;
                    FieldType::Decimal(m, n)
                }
            }
            "boolean" => FieldType::Boolean,
            "datetime" => FieldType::DateTime,
            "datetime64" => FieldType::DateTime64,
            "timestamp" => FieldType::Timestamp,
            "time" => FieldType::Time,
            "date" => FieldType::Date,
            "json" => FieldType::Json,
            "ipv4" => FieldType::IPv4,
            "ipv6" => FieldType::IPv6,
            "array" => {
                let f = Self::from_str(stats)?;
                FieldType::Array(Box::new(f))
            }
            _ => {
                return Err(ConfError::from(ConfReason::Syntax(format!(
                    "当前wpconf不支持该字段类型: {}",
                    t
                ))));
            }
        };
        Ok(field_type)
    }
}
