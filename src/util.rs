use std::io;

use serde_json as json;

use crate::UnsupportedInput;

pub fn other_io_error(e: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

pub fn write_json_row(mut sink: impl io::Write, row: &rusqlite::Row) -> io::Result<()> {
    use rusqlite::types::ValueRef::*;
    // TODO: this could probably be made more efficient by using a
    // lower-level serialization interface.
    let values: Vec<_> = (0..row.column_count())
        .map(|i| match row.get_raw(i) {
            Null => Ok(json::Value::Null),
            Integer(n) => Ok(json::Value::from(n)),
            Real(n) => Ok(json::Value::from(n)),
            Text(bytes) => {
                let text = String::from_utf8(bytes.to_vec())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
                Ok(json::Value::String(text))
            }
            Blob(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                UnsupportedInput::Blob,
            )),
        })
        .collect::<Result<_, io::Error>>()?;
    json::to_writer(&mut sink, &values)?;
    sink.write_all(b"\n")?;
    Ok(())
}
