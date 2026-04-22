use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmissionObj {
    pub data: SubmissionData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionData {
    pub submission_list: SubmissionList,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionList {
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub submissions: Vec<SubmissionMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionMeta {
    pub status_display: String,
    pub lang: String,
    pub url: String,
}

impl SubmissionMeta {
    pub fn is_accepted(&self) -> bool {
        self.status_display == "Accepted"
    }
}

fn deserialize_vec_or_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}
