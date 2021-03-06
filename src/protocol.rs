use std::collections::HashMap;
use serde_json;
use time;
use Marian;

#[derive(Serialize, Debug)]
struct LastSync {
    finished: String,
}

#[derive(Serialize, Debug)]
struct Status<'a> {
    #[serde(rename = "lastSync")]
    last_sync: Option<LastSync>,
    manifests: Vec<&'a String>,
    errors: &'a HashMap<String, String>,
}

pub fn create_status_string(marian: &Marian) -> String {
    let index = marian.index.read().unwrap();
    let last_sync = match index.finished.sec {
        0 => None,
        secs => {
            let time = time::at_utc(time::Timespec::new(secs as i64, 0));
            Some(LastSync {
                finished: format!("{}", time.rfc3339()),
            })
        }
    };

    let status = Status {
        last_sync,
        manifests: index.manifests.iter().collect(),
        errors: &index.manifest_errors,
    };

    serde_json::to_string(&status).unwrap()
}
