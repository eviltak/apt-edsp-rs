use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Progress {
    progress: String,
    percentage: Option<String>,
    message: Option<String>,
}
