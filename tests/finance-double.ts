import type { Page } from "@playwright/test";
import { installCatalogDouble } from "./catalog-double";

export async function installFinanceDouble(page:Page){
 await installCatalogDouble(page);
 await page.addInitScript(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(c:string,a?:Record<string,unknown>)=>Promise<unknown>};financeCalls:Array<{command:string;args:Record<string,unknown>}>;financeDelay:number;financeFail:string;financeEmpty:boolean};
  const original=w.__TAURI_INTERNALS__.invoke,now="2026-10-05T10:00:00.000Z";
  const accounts=[
   {id:"acc-cash",code:"1000",name:"Recorded Cash Sale Payments",accountType:"ASSET",normalSide:"DEBIT",isSystem:true,isActive:true},
   {id:"acc-refund",code:"1090",name:"Unallocated Customer Refund Clearing",accountType:"ASSET",normalSide:"DEBIT",isSystem:true,isActive:true},
   {id:"acc-inventory",code:"1200",name:"Inventory",accountType:"ASSET",normalSide:"DEBIT",isSystem:true,isActive:true},
   {id:"acc-tax",code:"2100",name:"Recorded Sales Tax Clearing",accountType:"LIABILITY",normalSide:"CREDIT",isSystem:true,isActive:true},
   {id:"acc-revenue",code:"4000",name:"Sales Revenue",accountType:"REVENUE",normalSide:"CREDIT",isSystem:true,isActive:true},
   {id:"acc-returns",code:"4010",name:"Sales Returns",accountType:"REVENUE",normalSide:"CREDIT",isSystem:true,isActive:true},
   {id:"acc-cogs",code:"5000",name:"Cost of Goods Sold",accountType:"EXPENSE",normalSide:"DEBIT",isSystem:true,isActive:true},
   {id:"acc-rent",code:"6100",name:"Rent Expense",accountType:"EXPENSE",normalSide:"DEBIT",isSystem:true,isActive:true},
   {id:"acc-misc",code:"6190",name:"Miscellaneous Operating Expense",accountType:"EXPENSE",normalSide:"DEBIT",isSystem:true,isActive:true},
  ];
  const expenses:Array<Record<string,unknown>>=Array.from({length:23},(_,i)=>({id:`expense-${i+1}`,expenseNumber:`EXP-20261005-${String(i+1).padStart(5,"0")}`,requestId:`request-${i+1}`,status:"POSTED",branchId:"main",branchName:"Main Pharmacy",expenseAccountId:"acc-rent",expenseAccountCode:"6100",expenseAccountName:"Rent Expense",paymentAccountId:"acc-cash",paymentAccountCode:"1000",paymentAccountName:"Recorded Cash Sale Payments",amountMinor:15000,description:`October rent ${i+1}`,journalId:`journal-expense-${i+1}`,createdBy:"Mohannad",createdAt:now,postedAt:now}));
  const journals:Array<Record<string,unknown>>=[
   {id:"journal-sale",journalNumber:"JRN-20261005-00001",sourceType:"SALE",sourceId:"sale-1",sourceParentId:null,branchId:"main",branchName:"Main Pharmacy",reversalOfJournalId:null,description:"Posted Sale SAL-1",debitTotalMinor:1590,creditTotalMinor:1590,cogsComplete:true,postedAt:now},
   {id:"journal-return",journalNumber:"JRN-20261005-00002",sourceType:"SALE_RETURN",sourceId:"return-1",sourceParentId:"sale-1",branchId:"main",branchName:"Main Pharmacy",reversalOfJournalId:null,description:"Posted Return",debitTotalMinor:600,creditTotalMinor:600,cogsComplete:true,postedAt:now},
   {id:"journal-void",journalNumber:"JRN-20261005-00003",sourceType:"SALE_VOID",sourceId:"void-1",sourceParentId:"sale-2",branchId:"main",branchName:"Main Pharmacy",reversalOfJournalId:"journal-sale-2",description:"Sale Void reversal",debitTotalMinor:300,creditTotalMinor:300,cogsComplete:true,postedAt:now},
   ...expenses.map((e,i)=>({id:e.journalId,journalNumber:`JRN-20261005-${String(i+4).padStart(5,"0")}`,sourceType:"EXPENSE",sourceId:e.id,sourceParentId:null,branchId:"main",branchName:"Main Pharmacy",reversalOfJournalId:null,description:e.description,debitTotalMinor:e.amountMinor,creditTotalMinor:e.amountMinor,cogsComplete:true,postedAt:now})),
  ];
  w.financeCalls=[];w.financeDelay=0;w.financeFail="";w.financeEmpty=false;
  const wait=async(command:string,args:Record<string,unknown>)=>{w.financeCalls.push({command,args:structuredClone(args)});if(w.financeDelay)await new Promise(r=>setTimeout(r,w.financeDelay));if(w.financeFail===command)throw {code:"database",message:"Controlled Finance failure"}};
  w.__TAURI_INTERNALS__.invoke=async(command,args={})=>{
   if(command==="recovery_status")return {...await original(command,args) as object,financeEnabled:true};
   if(command==="export_phase6_csv"){await wait(command,args);return {fileName:"finance.csv",mimeType:"text/csv;charset=utf-8",content:"Journal,Amount\r\nJRN-1,10.00 EGP\r\n",rowCount:1}}
   if(!command.includes("finance")&&!['get_trial_balance','get_account_activity','get_sales_financial_summary','get_expense_report'].includes(command))return original(command,args);
   await wait(command,args);
   if(command==="list_finance_accounts")return structuredClone(accounts);
   if(command==="get_sales_financial_summary")return {salesRevenueMinor:90000,salesReturnsMinor:36000,netSalesRevenueMinor:54000,refundOutflowMinor:40000,netCogsMinor:39000,missingCogsJournalCount:0,grossMarginMinor:15000,operatingExpensesMinor:15000,operatingResultMinor:0};
   if(command==="list_finance_expenses"||command==="get_expense_report"){const q=args.query as Record<string,unknown>,rows=w.financeEmpty?[]:expenses,offset=Number(q.offset??0),limit=Number(q.limit??20);return {items:structuredClone(rows.slice(offset,offset+limit)),total:rows.length,limit,offset,...(command==="get_expense_report"?{totalExpenseMinor:rows.reduce((s,e)=>s+Number(e.amountMinor),0)}:{})}}
   if(command==="list_finance_journals"){const q=args.query as Record<string,unknown>,rows=w.financeEmpty?[]:journals,offset=Number(q.offset??0),limit=Number(q.limit??20);return {items:structuredClone(rows.slice(offset,offset+limit)),total:rows.length,limit,offset}}
   if(command==="get_trial_balance")return {items:w.financeEmpty?[]:[{accountId:"acc-cash",accountCode:"1000",accountName:"Recorded Cash Sale Payments",accountType:"ASSET",normalSide:"DEBIT",debitMinor:100000,creditMinor:15000,balanceMinor:85000},{accountId:"acc-revenue",accountCode:"4000",accountName:"Sales Revenue",accountType:"REVENUE",normalSide:"CREDIT",debitMinor:0,creditMinor:85000,balanceMinor:85000}],debitTotalMinor:w.financeEmpty?0:100000,creditTotalMinor:w.financeEmpty?0:100000,balanced:true};
   if(command==="get_account_activity")return {items:w.financeEmpty?[]:[{id:"line-1",journalId:"journal-sale",journalNumber:"JRN-20261005-00001",sourceType:"SALE",sourceId:"sale-1",sourceParentId:null,branchId:"main",description:"Posted Sale SAL-1",side:"CREDIT",amountMinor:90000,memo:"Net-of-discount Sale revenue",postedAt:now}],total:w.financeEmpty?0:21,limit:20,offset:Number((args.query as Record<string,unknown>).offset??0)};
   if(command==="get_finance_expense")return structuredClone(expenses.find(e=>e.id===args.id));
   if(command==="get_finance_journal"){const j=journals.find(e=>e.id===args.id)??journals[0];return {...structuredClone(j),status:"POSTED",currency:"EGP",lines:[{id:"line-d",position:0,accountId:"acc-cash",accountCode:"1000",accountName:"Recorded Cash Sale Payments",accountType:"ASSET",side:"DEBIT",amountMinor:j.debitTotalMinor,memo:"Recorded payment"},{id:"line-c",position:1,accountId:"acc-revenue",accountCode:"4000",accountName:"Sales Revenue",accountType:"REVENUE",side:"CREDIT",amountMinor:j.creditTotalMinor,memo:"Revenue"}]}}
   if(command==="create_finance_expense"){const input=args.input as Record<string,unknown>;const row={...expenses[0],...input,expenseNumber:"EXP-20261005-99999",status:"POSTED",expenseAccountCode:"6100",expenseAccountName:"Rent Expense",paymentAccountName:"Recorded Cash Sale Payments",journalId:"journal-new",postedAt:now};expenses.unshift(row);return structuredClone(row)}
   throw new Error(`Unhandled Finance command ${command}`);
  };
 });
}
