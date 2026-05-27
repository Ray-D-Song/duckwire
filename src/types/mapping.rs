use duckdb::types::{Value, ValueRef};
use pgwire::api::Type;
use pgwire::api::results::{DataRowEncoder, FieldFormat, FieldInfo};
use std::sync::Arc;

use crate::errors::DuckWireError;

use arrow::datatypes::DataType;

pub fn arrow_type_to_pg(dt: &DataType) -> Result<Type, DuckWireError> {
    match dt {
        DataType::Null => Ok(Type::UNKNOWN),
        DataType::Boolean => Ok(Type::BOOL),
        DataType::Int8 => Ok(Type::INT2),
        DataType::Int16 => Ok(Type::INT2),
        DataType::Int32 => Ok(Type::INT4),
        DataType::Int64 => Ok(Type::INT8),
        DataType::UInt8 => Ok(Type::INT2),
        DataType::UInt16 => Ok(Type::INT4),
        DataType::UInt32 => Ok(Type::INT8),
        DataType::UInt64 => Ok(Type::INT8),
        DataType::Float32 => Ok(Type::FLOAT4),
        DataType::Float64 => Ok(Type::FLOAT8),
        DataType::Decimal128(_, _) | DataType::Decimal256(_, _) => Ok(Type::NUMERIC),
        DataType::Timestamp(_, _) => Ok(Type::TIMESTAMP),
        DataType::Date32 | DataType::Date64 => Ok(Type::DATE),
        DataType::Time64(_) => Ok(Type::TIME),
        DataType::Duration(_) => Ok(Type::INTERVAL),
        DataType::Utf8 | DataType::LargeUtf8 => Ok(Type::TEXT),
        DataType::Binary | DataType::LargeBinary | DataType::FixedSizeBinary(_) => Ok(Type::BYTEA),
        DataType::List(inner) => {
            let inner_pg = arrow_type_to_pg(inner.data_type())?;
            match inner_pg {
                Type::INT2 => Ok(Type::INT2_ARRAY),
                Type::INT4 => Ok(Type::INT4_ARRAY),
                Type::INT8 => Ok(Type::INT8_ARRAY),
                Type::FLOAT4 => Ok(Type::FLOAT4_ARRAY),
                Type::FLOAT8 => Ok(Type::FLOAT8_ARRAY),
                Type::BOOL => Ok(Type::BOOL_ARRAY),
                Type::TEXT | Type::VARCHAR => Ok(Type::TEXT_ARRAY),
                _ => Ok(Type::TEXT_ARRAY),
            }
        }
        DataType::Struct(_) => Ok(Type::JSON),
        DataType::Map(_, _) => Ok(Type::JSON),
        // Unwrap dictionary-encoded columns to their inner value type
        DataType::Dictionary(_, value_type) => arrow_type_to_pg(value_type.as_ref()),
        DataType::Union(_, _) => Ok(Type::JSON),
        _ => Ok(Type::TEXT),
    }
}

pub fn build_field_info(name: &str, dt: &DataType) -> Result<FieldInfo, DuckWireError> {
    let pg_type = arrow_type_to_pg(dt)?;
    Ok(FieldInfo::new(
        name.into(),
        None,
        None,
        pg_type,
        FieldFormat::Text,
    ))
}

pub fn build_schema_from_columns(
    columns: &[(String, DataType)],
) -> Result<Arc<Vec<FieldInfo>>, DuckWireError> {
    let fields: Vec<FieldInfo> = columns
        .iter()
        .map(|(name, dt)| build_field_info(name, dt))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(fields))
}

fn encode_text(encoder: &mut DataRowEncoder, val: &str) -> Result<(), DuckWireError> {
    encoder
        .encode_field(&val)
        .map_err(|e| DuckWireError::Protocol(e.to_string()))
}

pub fn encode_duckdb_value(
    encoder: &mut DataRowEncoder,
    value_ref: ValueRef<'_>,
) -> Result<(), DuckWireError> {
    let e = |err: pgwire::error::PgWireError| DuckWireError::Protocol(err.to_string());
    match value_ref {
        ValueRef::Null => encoder.encode_field(&None::<i32>).map_err(e),
        ValueRef::Boolean(b) => encoder.encode_field(&b).map_err(e),
        ValueRef::TinyInt(i) => encoder.encode_field(&(i as i16)).map_err(e),
        ValueRef::SmallInt(i) => encoder.encode_field(&i).map_err(e),
        ValueRef::Int(i) => encoder.encode_field(&i).map_err(e),
        ValueRef::BigInt(i) => encoder.encode_field(&i).map_err(e),
        ValueRef::Float(f) => encoder.encode_field(&f).map_err(e),
        ValueRef::Double(f) => encoder.encode_field(&f).map_err(e),
        ValueRef::HugeInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::UTinyInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::USmallInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::UInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::UBigInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::Decimal(d) => encode_text(encoder, &d.to_string()),
        // Manual Gregorian calendar conversion from epoch microseconds.
        // DuckDB timestamps are stored as micros since 1970-01-01 UTC.
        // Avoids pulling in a heavy date library.
        // NOTE: assumes post-epoch timestamps (no negative day counts).
        ValueRef::Timestamp(_, ts) => {
            let micros = ts;
            let secs = micros / 1_000_000;
            let remain_micros = micros % 1_000_000;
            let days_since_epoch = secs / 86400;
            let time_secs = secs % 86400;
            let hours = time_secs / 3600;
            let mins = (time_secs % 3600) / 60;
            let secs_rem = time_secs % 60;
            // Walk forward from 1970 to find the year
            let mut year: i32 = 1970;
            let mut remaining = days_since_epoch;
            loop {
                let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    366
                } else {
                    365
                };
                if remaining < days_in_year {
                    break;
                }
                remaining -= days_in_year;
                year += 1;
            }
            let is_leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            let month_days = [
                31,
                if is_leap { 29 } else { 28 },
                31,
                30,
                31,
                30,
                31,
                31,
                30,
                31,
                30,
                31,
            ];
            let mut month: usize = 0;
            for (i, &md) in month_days.iter().enumerate() {
                if remaining < md {
                    month = i;
                    break;
                }
                remaining -= md;
            }
            let day = remaining + 1;
            let result = if remain_micros > 0 {
                let frac_str = format!("{:06}", remain_micros)
                    .trim_end_matches('0')
                    .to_string();
                format!(
                    "{year:04}-{:02}-{:02} {:02}:{:02}:{:02}.{frac_str}",
                    month + 1,
                    day,
                    hours,
                    mins,
                    secs_rem
                )
            } else {
                format!(
                    "{year:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    month + 1,
                    day,
                    hours,
                    mins,
                    secs_rem
                )
            };
            encode_text(encoder, &result)
        }
        ValueRef::Text(t) => match std::str::from_utf8(t) {
            Ok(s) => encode_text(encoder, s),
            Err(_) => encode_text(encoder, &format!("{:?}", t)),
        },
        ValueRef::Blob(b) => {
            let hex_str: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
            encode_text(encoder, &format!("\\\\x{hex_str}"))
        }
        ValueRef::Date32(d) => encode_text(encoder, &d.to_string()),
        ValueRef::Time64(_, t) => encode_text(encoder, &t.to_string()),
        ValueRef::Interval {
            months,
            days,
            nanos,
        } => encode_text(encoder, &format!("{months} mons {days} days {nanos} ns")),
        ValueRef::List(_, _) => encode_text(encoder, "[list]"),
        ValueRef::Enum(_, _) => match value_ref.as_str() {
            Ok(s) => encode_text(encoder, s),
            Err(_) => encode_text(encoder, "?"),
        },
        ValueRef::Struct(_, _) => encode_text(encoder, "[struct]"),
        ValueRef::Array(_, _) => encode_text(encoder, "[array]"),
        ValueRef::Map(_, _) => encode_text(encoder, "[map]"),
        ValueRef::Union(_, _) => encode_text(encoder, "[union]"),
    }
}

pub fn encode_duckdb_owned_value(
    encoder: &mut DataRowEncoder,
    value: &Value,
) -> Result<(), DuckWireError> {
    match value {
        Value::List(values) | Value::Array(values) => {
            encode_text(encoder, &format_pg_array_literal(values))
        }
        Value::Struct(_) | Value::Map(_) | Value::Union(_) => encode_text(encoder, &format!("{value:?}")),
        Value::Enum(s) => encode_text(encoder, s),
        _ => encode_duckdb_value(encoder, ValueRef::from(value)),
    }
}

fn format_pg_array_literal(values: &[Value]) -> String {
    let elements = values
        .iter()
        .map(format_pg_array_element)
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{elements}}}")
}

fn format_pg_array_element(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Text(s) | Value::Enum(s) => quote_pg_array_element(s),
        Value::Boolean(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::TinyInt(v) => v.to_string(),
        Value::SmallInt(v) => v.to_string(),
        Value::Int(v) => v.to_string(),
        Value::BigInt(v) => v.to_string(),
        Value::HugeInt(v) => v.to_string(),
        Value::UTinyInt(v) => v.to_string(),
        Value::USmallInt(v) => v.to_string(),
        Value::UInt(v) => v.to_string(),
        Value::UBigInt(v) => v.to_string(),
        Value::Float(v) => v.to_string(),
        Value::Double(v) => v.to_string(),
        Value::Decimal(v) => v.to_string(),
        Value::Timestamp(_, v) => quote_pg_array_element(&v.to_string()),
        Value::Blob(v) => quote_pg_array_element(&format!("{v:?}")),
        Value::Date32(v) => quote_pg_array_element(&v.to_string()),
        Value::Time64(_, v) => quote_pg_array_element(&v.to_string()),
        Value::Interval {
            months,
            days,
            nanos,
        } => quote_pg_array_element(&format!("{months} mons {days} days {nanos} ns")),
        Value::List(values) | Value::Array(values) => quote_pg_array_element(&format_pg_array_literal(values)),
        Value::Struct(_) | Value::Map(_) | Value::Union(_) => quote_pg_array_element(&format!("{value:?}")),
    }
}

fn quote_pg_array_element(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
