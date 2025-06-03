use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SolutionObj {
    pub data: SolutionData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolutionData {
    pub question: SolutionQuestion,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionQuestion {
    pub question_id: String,
    pub article: String,
    pub solution: SolutionDetail,
    #[serde(rename = "__typename")]
    pub typename: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionDetail {
    pub id: String,
    pub content: String,
    pub content_type_id: String,
    pub can_see_detail: bool,
    pub paid_only: bool,
    pub rating: SolutionRating,
    #[serde(rename = "__typename")]
    pub typename: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionRating {
    pub id: String,
    pub count: u32,
    pub average: String,
    #[serde(rename = "__typename")]
    pub typename: String,
}
