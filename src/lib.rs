use eyre::ContextCompat;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};

use fluvio_smartmodule::{
    dataplane::smartmodule::SmartModuleExtraParams, //, SmartModuleInitError},
    eyre, smartmodule, SmartModuleRecord, RecordData, Result,
};
use chrono::{FixedOffset, NaiveDateTime, TimeZone};

static SPEC: OnceCell<DateOpsParams> = OnceCell::new();
// const PARAM_NAME: &str = "spec";

#[derive(Debug, Serialize, Deserialize)]
struct DateOpsParams {
    source_format: String,
    output_format: String,
    source_timezone: i32,
    output_timezone: String, // Assuming UTC for output timezone
    fields: Vec<String>,
}

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let key: Option<RecordData> = record.key.clone();
    let spec = SPEC.get().wrap_err("spec is not initialized")?;
    let date_str = record.value.as_str()?;
    let formatted_date = date_change(&spec, date_str)?;

    Ok((key, RecordData::from(formatted_date)))

    // let record_data: Value = serde_json::from_slice(record.value.as_ref())?;

    // let source_timezone = FixedOffset::east_opt(spec.source_timezone * 3600).unwrap();

    // let mut updated_record_data = record_data.clone();
    // let fields = vec!["foo"];
    // let fiedls = spec.fields;
    // for field in &fields {
    //     if let Some(date_str) = record_data.get(field).and_then(Value::as_str) {
    //         let ndt = NaiveDateTime::parse_from_str(date_str, &spec.source_format)
    //             .map_err(|e| eyre!("Failed to parse date: {}", e))?;
    //         // unwrap always succeeds because of Fixed offset of source_timezone
    //         // see https://docs.rs/chrono/latest/chrono/offset/enum.MappedLocalTime.html
    //         let dt_tz = source_timezone.from_local_datetime(&ndt).unwrap();
    //         let utc_date = dt_tz.to_utc();
    //         let formatted_date = utc_date.format(&spec.output_format).to_string();
    //         updated_record_data[field] = json!(formatted_date);
    //     }
    // }

    // Ok((key, RecordData::from(serde_json::to_vec(&updated_record_data)?)))
}

fn date_change(spec: &DateOpsParams, date_str: &str) -> Result<String> {
    let source_timezone = FixedOffset::east_opt(spec.source_timezone * 3600).unwrap();
    let ndt = NaiveDateTime::parse_from_str(date_str, &spec.source_format)
        .map_err(|e| eyre!("Failed to parse date: {}", e))?;
    // unwrap always succeeds because of Fixed offset of source_timezone
    // see https://docs.rs/chrono/latest/chrono/offset/enum.MappedLocalTime.html
    let dt_tz = source_timezone.from_local_datetime(&ndt).unwrap();
    let utc_date = dt_tz.to_utc();
    let formatted_date = utc_date.format(&spec.output_format).to_string();
    Ok(formatted_date)
}

#[test]
fn t_date_change() -> Result<()> {
    let spec = DateOpsParams {
		source_format: "%Y/%d/%m %H:%M:%S".to_string(),
		output_format: "%Y-%d-%m %H:%M:%S'T'HH:MM:SS.SSS'Z'".to_string(),
		source_timezone: 1,
		output_timezone: "UTC".to_string(),
        fields: vec!["foo".to_string()],
    };
    let date = "2024/12/02 01:13:23";
    let res = date_change(&spec, date)?;
    dbg!(res);

    Ok(())
}

#[smartmodule(init)]
fn init(_params: SmartModuleExtraParams) -> Result<()> {
    let spec = DateOpsParams {
		source_format: "yyyy/dd/mm HH:MM:SS".to_string(),
		output_format: "yyyy-mm-dd'T'HH:MM:SS.SSS'Z'".to_string(),
		source_timezone: 0,
		output_timezone: "UTC".to_string(),
        fields: vec!["foo".to_string()],
    };
    SPEC.set(spec).expect("spec is already initialized");
    Ok(())
    // if let Some(raw_spec) = params.get(PARAM_NAME) {
    //     match serde_json::from_str(raw_spec) {
    //         Ok(spec) => {
    //             SPEC.set(spec).expect("spec is already initialized");
    //             Ok(())
    //         }
    //         Err(err) => {
    //             eprintln!("unable to parse spec from params: {err:?}");
    //             Err(eyre!("cannot parse `spec` param: {:#?}", err))
    //         }
    //     }
    // } else {
    //     Err(SmartModuleInitError::MissingParam(PARAM_NAME.to_string()).into())
    // }
}

