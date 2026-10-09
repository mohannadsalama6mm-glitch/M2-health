use crate::{finance::models::FinanceQuery, reporting::models::ReportFilter};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Phase6Filter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub branch_id: Option<String>,
}

impl Phase6Filter {
    pub fn report(&self) -> ReportFilter {
        ReportFilter {
            date_from: self.date_from.clone(),
            date_to: self.date_to.clone(),
            branch_id: self.branch_id.clone(),
            ..Default::default()
        }
    }
    pub fn finance(&self) -> FinanceQuery {
        FinanceQuery {
            date_from: self.date_from.clone(),
            date_to: self.date_to.clone(),
            branch_id: self.branch_id.clone(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CsvExportInput {
    pub kind: String,
    #[serde(default)]
    pub report_filter: ReportFilter,
    #[serde(default)]
    pub finance_query: FinanceQuery,
}
