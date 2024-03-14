use eyre::ContextCompat;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use fluvio_smartmodule::{
    dataplane::smartmodule::{SmartModuleExtraParams, SmartModuleInitError},
    eyre, smartmodule, Record, RecordData, Result,
};
use chrono::{FixedOffset, TimeZone, Utc};

static SPEC: OnceCell<DateOpsParams> = OnceCell::new();
const PARAM_NAME: &str = "spec";

#[derive(Debug, Serialize, Deserialize)]
struct DateOpsParams {
    source_format: String,
    output_format: String,
    source_timezone: i32,
    output_timezone: String, // Assuming UTC for output timezone
    fields: Vec<String>,
}

#[smartmodule(map)]
pub fn map(record: &Record) -> Result<(Option<RecordData>, RecordData)> {
    let key: Option<RecordData> = record.key.clone();
    let spec = SPEC.get().wrap_err("spec is not initialized")?;
    let record_data: Value = serde_json::from_slice(record.value.as_ref())?;

    let source_timezone = FixedOffset::east(spec.source_timezone * 3600);

    let mut updated_record_data = record_data.clone();
    for field in &spec.fields {
        if let Some(date_str) = record_data.get(field).and_then(Value::as_str) {
            let date = source_timezone
                .datetime_from_str(date_str, &spec.source_format)
                .map_err(|e| eyre!("Failed to parse date: {}", e))?;

            let utc_date = date.with_timezone(&Utc);
            let formatted_date = utc_date.format(&spec.output_format).to_string();
            updated_record_data[field] = json!(formatted_date);
        }
    }

    Ok((key, RecordData::from(serde_json::to_vec(&updated_record_data)?)))
}

#[smartmodule(init)]
fn init(params: SmartModuleExtraParams) -> Result<()> {
    if let Some(raw_spec) = params.get(PARAM_NAME) {
        match serde_json::from_str(raw_spec) {
            Ok(spec) => {
                SPEC.set(spec).expect("spec is already initialized");
                Ok(())
            }
            Err(err) => {
                eprintln!("unable to parse spec from params: {err:?}");
                Err(eyre!("cannot parse `spec` param: {:#?}", err))
            }
        }
    } else {
        Err(SmartModuleInitError::MissingParam(PARAM_NAME.to_string()).into())
    }
}