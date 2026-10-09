import { nativeRequest } from "./client";

export interface FinanceQuery {
  dateFrom?: string | null; dateTo?: string | null; branchId?: string | null;
  accountId?: string | null; sourceType?: string | null; search?: string | null;
  limit?: number; offset?: number;
}
export interface FinancePage<T> { items:T[]; total:number; limit:number; offset:number }
export interface FinanceAccount { id:string; code:string; name:string; accountType:"ASSET"|"LIABILITY"|"EQUITY"|"REVENUE"|"EXPENSE"; normalSide:"DEBIT"|"CREDIT"; isSystem:boolean; isActive:boolean }
export interface JournalLine { id:string; position:number; accountId:string; accountCode:string; accountName:string; accountType:string; side:"DEBIT"|"CREDIT"; amountMinor:number; memo:string|null }
export interface FinanceJournal { id:string; journalNumber:string; status?:string; sourceType:string; sourceId:string; sourceParentId:string|null; branchId:string|null; branchName?:string|null; reversalOfJournalId:string|null; description:string; currency?:string; debitTotalMinor:number; creditTotalMinor:number; cogsComplete:boolean; postedAt:string; lines?:JournalLine[] }
export interface FinanceExpense { id:string; expenseNumber:string; requestId?:string; status?:string; branchId:string|null; branchName:string|null; expenseAccountId:string; expenseAccountCode:string; expenseAccountName:string; paymentAccountId:string; paymentAccountCode?:string; paymentAccountName:string; amountMinor:number; currency?:string; description:string; journalId:string; createdBy?:string|null; createdAt?:string; postedAt:string }
export interface TrialBalanceRow { accountId:string; accountCode:string; accountName:string; accountType:string; normalSide:string; debitMinor:number; creditMinor:number; balanceMinor:number }
export interface TrialBalance { items:TrialBalanceRow[]; debitTotalMinor:number; creditTotalMinor:number; balanced:boolean }
export interface ActivityRow { id:string; journalId:string; journalNumber:string; sourceType:string; sourceId:string; sourceParentId:string|null; branchId:string|null; description:string; side:"DEBIT"|"CREDIT"; amountMinor:number; memo:string|null; postedAt:string }
export interface SalesFinancialSummary { salesRevenueMinor:number; salesReturnsMinor:number; netSalesRevenueMinor:number; refundOutflowMinor:number; netCogsMinor:number; missingCogsJournalCount:number; grossMarginMinor:number|null; operatingExpensesMinor:number; operatingResultMinor:number|null }
export interface ExpenseReport extends FinancePage<FinanceExpense> { totalExpenseMinor:number }
export interface CreateExpenseInput { id:string; requestId:string; branchId:string|null; expenseAccountId:string; paymentAccountId:string; amountMinor:number; description:string; createdBy?:string|null }
const call = <T>(command:string, args:Record<string,unknown>={}) => nativeRequest<T>(command,args);
export const finance = {
  accounts:()=>call<FinanceAccount[]>("list_finance_accounts"),
  createExpense:(input:CreateExpenseInput)=>call<FinanceExpense>("create_finance_expense",{input}),
  expense:(id:string)=>call<FinanceExpense>("get_finance_expense",{id}),
  expenses:(query:FinanceQuery)=>call<FinancePage<FinanceExpense>>("list_finance_expenses",{query}),
  journal:(id:string)=>call<FinanceJournal>("get_finance_journal",{id}),
  journals:(query:FinanceQuery)=>call<FinancePage<FinanceJournal>>("list_finance_journals",{query}),
  trialBalance:(query:FinanceQuery)=>call<TrialBalance>("get_trial_balance",{query}),
  activity:(query:FinanceQuery)=>call<FinancePage<ActivityRow>>("get_account_activity",{query}),
  salesSummary:(query:FinanceQuery)=>call<SalesFinancialSummary>("get_sales_financial_summary",{query}),
  expenseReport:(query:FinanceQuery)=>call<ExpenseReport>("get_expense_report",{query}),
};
