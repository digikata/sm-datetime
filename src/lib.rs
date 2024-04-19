use eyre::ContextCompat;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};

use fluvio_smartmodule::{
    dataplane::smartmodule::{SmartModuleExtraParams, SmartModuleInitError},
    eyre, smartmodule, SmartModuleRecord, RecordData, Result,
};
use chrono::{FixedOffset, NaiveDateTime, TimeZone};

static SPEC: OnceCell<DateOpsParams> = OnceCell::new();
static PARAM_NAME: &str = "date_ops_params";

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
    let formatted_date = match date_change(&spec, date_str) {
        Ok(date) => date,
        Err(e) => format!("error: {e}\n{date_str}")
    };

    Ok((key, RecordData::from(formatted_date)))
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
		output_format: "%Y-%d-%m %H:%M:%S".to_string(),
		source_timezone: 2,
		output_timezone: "UTC".to_string(),
        fields: vec!["foo".to_string()],
    };
    let jspec = serde_json::to_string(&spec).expect("ser error");
    println!("config: {jspec}");
    let date_pre = "2024/12/02 01:13:23";
    let date_post = date_change(&spec, date_pre)?;
    dbg!(date_pre, date_post);


    Ok(())
}

#[smartmodule(init)]
fn init(params: SmartModuleExtraParams) -> Result<()> {

    // hardcoded spec for testing
    // let spec = DateOpsParams {
	// 	source_format: "%Y/%d/%m %H:%M:%S".to_string(),
	// 	output_format: "%Y-%d-%m %H:%M:%S".to_string(),
	// 	source_timezone: 3,
	// 	output_timezone: "UTC".to_string(),
    //     fields: vec!["foo".to_string()],
    // };
    // SPEC.set(spec).expect("spec is already initialized");
    // Ok(())

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

