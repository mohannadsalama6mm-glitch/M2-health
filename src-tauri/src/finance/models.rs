use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateExpense {
    pub id: String,
    pub request_id: String,
    pub branch_id: Option<String>,
    pub expense_account_id: String,
    pub payment_account_id: String,
    pub amount_minor: i64,
    pub description: String,
    pub created_by: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FinanceQuery {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub branch_id: Option<String>,
    pub account_id: Option<String>,
    pub source_type: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Copy)]
pub(crate) enum Side {
    Debit,
    Credit,
}
impl Side {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Debit => "DEBIT",
            Self::Credit => "CREDIT",
        }
    }
}

pub(crate) struct Line {
    pub code: &'static str,
    pub side: Side,
    pub amount: i64,
    pub memo: String,
}
