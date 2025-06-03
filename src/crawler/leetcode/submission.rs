use serde::{Deserialize, Serialize};

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
    pub submissions: Vec<SubmissionMeta>,
    #[serde(rename = "__typename")]
    pub typename: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionMeta {
    pub id: String,
    pub status_display: String,
    pub lang: String,
    pub runtime: String,
    pub timestamp: String,
    pub url: String,
    pub is_pending: String,
    #[serde(rename = "__typename")]
    pub typename: String,
}

impl SubmissionMeta {
    pub fn is_accepted(&self) -> bool {
        self.status_display == "Accepted"
    }
}
