use duckdb::types::ValueRef;
use pgwire::api::results::{DataRowEncoder, FieldFormat, FieldInfo};
use pgwire::api::Type;
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
    encoder.encode_field(&val).map_err(|e| DuckWireError::Protocol(e.to_string()))
}

pub fn encode_duckdb_value(
    encoder: &mut DataRowEncoder,
    value_ref: ValueRef<'_>,
) -> Result<(), DuckWireError> {
    match value_ref {
        ValueRef::Null => {
            encoder.encode_field(&None::<i32>)
                .map_err(|e| DuckWireError::Protocol(e.to_string()))
        }
        ValueRef::Boolean(b) => {
            let s = if b { "t" } else { "f" };
            encode_text(encoder, s)
        }
        ValueRef::TinyInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::SmallInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::Int(i) => encode_text(encoder, &i.to_string()),
        ValueRef::BigInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::HugeInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::UTinyInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::USmallInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::UInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::UBigInt(i) => encode_text(encoder, &i.to_string()),
        ValueRef::Float(f) => encode_text(encoder, &f.to_string()),
        ValueRef::Double(f) => encode_text(encoder, &f.to_string()),
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
                let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
                if remaining < days_in_year { break; }
                remaining -= days_in_year;
                year += 1;
            }
            let is_leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            let month_days = [31, if is_leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut month: usize = 0;
            for (i, &md) in month_days.iter().enumerate() {
                if remaining < md { month = i; break; }
                remaining -= md;
            }
            let day = remaining + 1;
            let result = if remain_micros > 0 {
                let frac_str = format!("{:06}", remain_micros).trim_end_matches('0').to_string();
                format!("{year:04}-{:02}-{:02} {:02}:{:02}:{:02}.{frac_str}", month + 1, day, hours, mins, secs_rem)
            } else {
                format!("{year:04}-{:02}-{:02} {:02}:{:02}:{:02}", month + 1, day, hours, mins, secs_rem)
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
        ValueRef::Interval { months, days, nanos } => {
            encode_text(encoder, &format!("{months} mons {days} days {nanos} ns"))
        }
        ValueRef::List(_, _) => encode_text(encoder, "[list]"),
        ValueRef::Enum(_, _) => {
            match value_ref.as_str() {
                Ok(s) => encode_text(encoder, s),
                Err(_) => encode_text(encoder, "?"),
            }
        }
        ValueRef::Struct(_, _) => encode_text(encoder, "[struct]"),
        ValueRef::Array(_, _) => encode_text(encoder, "[array]"),
        ValueRef::Map(_, _) => encode_text(encoder, "[map]"),
        ValueRef::Union(_, _) => encode_text(encoder, "[union]"),
    }
}