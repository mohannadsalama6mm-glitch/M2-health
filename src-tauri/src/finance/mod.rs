//! Canonical double-entry Finance services. Operational domains remain source events.
pub mod commands;
pub mod models;
use crate::{
    cloud::outbox,
    db::{
        catalog_validation::{id, invalid, optional, required, MAX_SAFE_MINOR},
        connection::AppDb,
    },
    error::{AppError, ErrorCode},
    inventory,
};
use models::{CreateExpense, FinanceQuery, Line, Side};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde_json::{json, Value};

const CASH: &str = "1000";
const CARD: &str = "1010";
const OTHER: &str = "1020";
const REFUND: &str = "1090";
const INVENTORY: &str = "1200";
const TAX: &str = "2100";
const REVENUE: &str = "4000";
const RETURNS: &str = "4010";
const COGS: &str = "5000";
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn safe(v: i128) -> Result<i64, AppError> {
    i64::try_from(v)
        .ok()
        .filter(|n| *n >= 0 && *n <= MAX_SAFE_MINOR)
        .ok_or_else(|| invalid("Finance amount exceeds the safe integer range."))
}
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let s: String = r.get(0)?;
    serde_json::from_str(&s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
fn page(items: Vec<Value>, total: i64, limit: i64, offset: i64) -> Value {
    json!({"items":items,"total":total,"limit":limit,"offset":offset})
}
fn validate(c: &Connection, q: &FinanceQuery) -> Result<(i64, i64, String), AppError> {
    for d in [&q.date_from, &q.date_to].into_iter().flatten() {
        inventory::date(c, d)?;
    }
    if matches!((&q.date_from,&q.date_to),(Some(a),Some(b)) if a>b) {
        return Err(invalid("Invalid Finance date range."));
    }
    for v in [&q.branch_id, &q.account_id].into_iter().flatten() {
        id(v)?;
    }
    if let Some(v) = &q.branch_id {
        if !c.query_row(
            "SELECT EXISTS(SELECT 1 FROM branches WHERE id=?1)",
            [v],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(AppError::new(ErrorCode::NotFound, "Branch not found."));
        }
    }
    if let Some(v) = &q.account_id {
        if !c.query_row(
            "SELECT EXISTS(SELECT 1 FROM finance_accounts WHERE id=?1)",
            [v],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(AppError::new(
                ErrorCode::NotFound,
                "Finance account not found.",
            ));
        }
    }
    let s = q.search.as_deref().unwrap_or("").trim();
    if s.chars().count() > 200 || s.contains('\0') {
        return Err(invalid("Finance search must be at most 200 characters."));
    }
    let (l, o) = inventory::page(q.limit, q.offset)?;
    Ok((l, o, s.into()))
}
fn account(c: &Connection, code: &str) -> Result<String, AppError> {
    c.query_row(
        "SELECT id FROM finance_accounts WHERE code=?1 AND is_active=1",
        [code],
        |r| r.get(0),
    )
    .map_err(Into::into)
}
fn number(c: &Connection) -> Result<String, AppError> {
    let day: String = c.query_row("SELECT strftime('%Y%m%d','now')", [], |r| r.get(0))?;
    c.execute("INSERT INTO finance_journal_sequences(day,next_number) VALUES(?1,1) ON CONFLICT(day) DO UPDATE SET next_number=next_number+1",[&day])?;
    let n: i64 = c.query_row(
        "SELECT next_number FROM finance_journal_sequences WHERE day=?1",
        [&day],
        |r| r.get(0),
    )?;
    Ok(format!("JRN-{day}-{n:05}"))
}
fn post(
    c: &Connection,
    source_type: &str,
    source_id: &str,
    parent: Option<&str>,
    branch: Option<&str>,
    description: &str,
    reversal: Option<&str>,
    cogs_complete: bool,
    mut lines: Vec<Line>,
) -> Result<Option<String>, AppError> {
    id(source_id)?;
    if let Some(v) = parent {
        id(v)?;
    }
    if let Some(v) = branch {
        id(v)?;
    }
    required(description)?;
    lines.retain(|l| l.amount > 0);
    if lines.is_empty() {
        return Ok(None);
    }
    let debit = safe(
        lines
            .iter()
            .filter(|l| matches!(l.side, Side::Debit))
            .map(|l| i128::from(l.amount))
            .sum(),
    )?;
    let credit = safe(
        lines
            .iter()
            .filter(|l| matches!(l.side, Side::Credit))
            .map(|l| i128::from(l.amount))
            .sum(),
    )?;
    if debit == 0 || debit != credit || lines.len() < 2 {
        return Err(invalid(
            "Finance journal must contain balanced positive debit and credit lines.",
        ));
    }
    let journal = uuid();
    let journal_number = number(c)?;
    c.execute("INSERT INTO finance_journals(id,journal_number,source_type,source_id,source_parent_id,branch_id,reversal_of_journal_id,description,debit_total_minor,credit_total_minor,cogs_complete) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9,?10)",params![journal,journal_number,source_type,source_id,parent,branch,reversal,description,debit,cogs_complete])?;
    for (position, line) in lines.into_iter().enumerate() {
        let account = account(c, line.code)?;
        c.execute("INSERT INTO finance_journal_lines(id,journal_id,position,account_id,side,amount_minor,memo) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![uuid(),journal,position as i64,account,line.side.as_str(),line.amount,line.memo])?;
    }
    c.execute("UPDATE finance_journals SET status='POSTED',posted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[&journal])?;
    let actor: Option<String> = match source_type {
        "SALE" => c
            .query_row(
                "SELECT created_by FROM sales WHERE id=?1",
                [source_id],
                |r| r.get(0),
            )
            .optional()?
            .flatten(),
        "SALE_RETURN" => c
            .query_row(
                "SELECT created_by FROM sale_returns WHERE id=?1",
                [source_id],
                |r| r.get(0),
            )
            .optional()?
            .flatten(),
        "SALE_VOID" => parent
            .map(|sale_id| {
                c.query_row("SELECT created_by FROM sales WHERE id=?1", [sale_id], |r| {
                    r.get(0)
                })
                .optional()
            })
            .transpose()?
            .flatten()
            .flatten(),
        "EXPENSE" => c
            .query_row(
                "SELECT created_by FROM finance_expenses WHERE id=?1",
                [source_id],
                |r| r.get(0),
            )
            .optional()?
            .flatten(),
        _ => None,
    };
    if actor.is_some() {
        let payload = journal_on(c, &journal)?;
        outbox::enqueue(
            c,
            branch,
            actor.as_deref(),
            "finance_journal",
            &journal,
            "APPENDED",
            &payload,
            Some(source_id),
        )?;
    }
    Ok(Some(journal))
}
fn payment_code(method: &str) -> Result<&'static str, AppError> {
    match method {
        "CASH" => Ok(CASH),
        "CARD" => Ok(CARD),
        "OTHER" => Ok(OTHER),
        _ => Err(invalid("Unsupported canonical Sale payment method.")),
    }
}

fn known_code(value: &str) -> Result<&'static str, AppError> {
    match value {
        CASH => Ok(CASH),
        CARD => Ok(CARD),
        OTHER => Ok(OTHER),
        REFUND => Ok(REFUND),
        INVENTORY => Ok(INVENTORY),
        TAX => Ok(TAX),
        REVENUE => Ok(REVENUE),
        RETURNS => Ok(RETURNS),
        COGS => Ok(COGS),
        _ => Err(invalid(
            "Original Finance journal contains an unsupported account.",
        )),
    }
}

fn sale_cost(c: &Connection, sale: &str) -> Result<(i64, bool), AppError> {
    let mut statement = c.prepare(
        "SELECT a.quantity,a.unit_cost_minor FROM sale_item_allocations a JOIN sale_items i ON i.id=a.sale_item_id WHERE i.sale_id=?1",
    )?;
    let mut total = 0_i128;
    let mut missing = false;
    for result in statement.query_map([sale], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?))
    })? {
        let (quantity, cost) = result?;
        if let Some(cost) = cost {
            total = total
                .checked_add(i128::from(quantity) * i128::from(cost))
                .ok_or_else(|| invalid("Allocation cost exceeds supported limits."))?;
        } else {
            missing = true;
        }
    }
    Ok((safe(total)?, missing))
}

fn return_cost(c: &Connection, return_id: &str) -> Result<(i64, bool), AppError> {
    let mut statement = c.prepare(
        "SELECT ra.quantity,a.unit_cost_minor FROM sale_return_allocations ra JOIN sale_return_items ri ON ri.id=ra.return_item_id JOIN sale_item_allocations a ON a.id=ra.original_allocation_id WHERE ri.return_id=?1",
    )?;
    let mut total = 0_i128;
    let mut missing = false;
    for result in statement.query_map([return_id], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?))
    })? {
        let (quantity, cost) = result?;
        if let Some(cost) = cost {
            total = total
                .checked_add(i128::from(quantity) * i128::from(cost))
                .ok_or_else(|| invalid("Returned allocation cost exceeds supported limits."))?;
        } else {
            missing = true;
        }
    }
    Ok((safe(total)?, missing))
}

pub(crate) fn post_sale(c: &Connection, sale: &str) -> Result<Option<String>, AppError> {
    let(status,branch,number,subtotal,discount,tax):(String,String,String,i64,i64,i64)=c.query_row("SELECT status,branch_id,sale_number,subtotal_minor,discount_minor,tax_minor FROM sales WHERE id=?1",[sale],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))?;
    if status != "POSTED" {
        return Err(invalid("Only a posted Sale can create a Finance journal."));
    }
    let mut lines = Vec::new();
    {
        let mut q=c.prepare("SELECT payment_method,amount_minor FROM sale_payments WHERE sale_id=?1 ORDER BY created_at,id")?;
        for result in q.query_map([sale], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })? {
            let (method, amount) = result?;
            lines.push(Line {
                code: payment_code(&method)?,
                side: Side::Debit,
                amount,
                memo: format!("{method} payment"),
            });
        }
    }
    let net = safe(i128::from(subtotal) - i128::from(discount))?;
    lines.push(Line {
        code: REVENUE,
        side: Side::Credit,
        amount: net,
        memo: "Net-of-discount Sale revenue".into(),
    });
    lines.push(Line {
        code: TAX,
        side: Side::Credit,
        amount: tax,
        memo: "Recorded Sale tax".into(),
    });
    let (cost, missing) = sale_cost(c, sale)?;
    if !missing {
        lines.push(Line {
            code: COGS,
            side: Side::Debit,
            amount: cost,
            memo: "Allocation-based COGS".into(),
        });
        lines.push(Line {
            code: INVENTORY,
            side: Side::Credit,
            amount: cost,
            memo: "Inventory cost released".into(),
        });
    }
    post(
        c,
        "SALE",
        sale,
        None,
        Some(&branch),
        &format!("Posted Sale {number}"),
        None,
        !missing,
        lines,
    )
}

pub(crate) fn post_return(c: &Connection, return_id: &str) -> Result<Option<String>, AppError> {
    let(sale,status,refund,branch,number):(String,String,i64,String,String)=c.query_row("SELECT r.sale_id,r.status,r.refund_minor,s.branch_id,s.sale_number FROM sale_returns r JOIN sales s ON s.id=r.sale_id WHERE r.id=?1",[return_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
    if status != "POSTED" {
        return Err(invalid(
            "Only a posted Sale Return can create a Finance journal.",
        ));
    }
    let mut tax: i128 = 0;
    {
        let mut q=c.prepare("SELECT ri.sale_item_id,ri.quantity,si.quantity,si.tax_minor,coalesce((SELECT sum(pri.quantity) FROM sale_return_items pri JOIN sale_returns pr ON pr.id=pri.return_id WHERE pri.sale_item_id=ri.sale_item_id AND pr.status='POSTED' AND pr.id!=?1),0) FROM sale_return_items ri JOIN sale_items si ON si.id=ri.sale_item_id WHERE ri.return_id=?1")?;
        for result in q.query_map([return_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })? {
            let (_, qty, sold, item_tax, prior) = result?;
            tax += (i128::from(prior + qty) * i128::from(item_tax) / i128::from(sold))
                - (i128::from(prior) * i128::from(item_tax) / i128::from(sold));
        }
    }
    let tax = safe(tax)?;
    if tax > refund {
        return Err(invalid("Returned tax exceeds the canonical refund."));
    }
    let mut lines = vec![
        Line {
            code: RETURNS,
            side: Side::Debit,
            amount: refund - tax,
            memo: "Canonical Sale Return excluding tax".into(),
        },
        Line {
            code: TAX,
            side: Side::Debit,
            amount: tax,
            memo: "Returned recorded Sale tax".into(),
        },
        Line {
            code: REFUND,
            side: Side::Credit,
            amount: refund,
            memo: "Canonical customer refund; settlement channel unallocated".into(),
        },
    ];
    let (cost, missing) = return_cost(c, return_id)?;
    if !missing {
        lines.push(Line {
            code: INVENTORY,
            side: Side::Debit,
            amount: cost,
            memo: "Original allocation cost restored".into(),
        });
        lines.push(Line {
            code: COGS,
            side: Side::Credit,
            amount: cost,
            memo: "COGS reversed for returned allocation".into(),
        });
    }
    post(
        c,
        "SALE_RETURN",
        return_id,
        Some(&sale),
        Some(&branch),
        &format!("Posted Return for Sale {number}"),
        None,
        !missing,
        lines,
    )
}

pub(crate) fn post_void(
    c: &Connection,
    void_id: &str,
    sale: &str,
) -> Result<Option<String>, AppError> {
    let original:Option<(String,String)>=c.query_row("SELECT id,branch_id FROM finance_journals WHERE source_type='SALE' AND source_id=?1 AND status='POSTED'",[sale],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    let Some((original, branch)) = original else {
        return Ok(None);
    };
    let cogs_complete: bool = c.query_row(
        "SELECT cogs_complete FROM finance_journals WHERE id=?1",
        [&original],
        |r| r.get(0),
    )?;
    let mut q=c.prepare("SELECT a.code,l.side,l.amount_minor,l.memo FROM finance_journal_lines l JOIN finance_accounts a ON a.id=l.account_id WHERE l.journal_id=?1 ORDER BY l.position")?;
    let raw_lines = q
        .query_map([&original], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let lines = raw_lines
        .into_iter()
        .map(|(code, side, amount, memo)| {
            Ok(Line {
                code: known_code(&code)?,
                side: if side == "DEBIT" {
                    Side::Credit
                } else {
                    Side::Debit
                },
                amount,
                memo: format!("Reversal: {}", memo.unwrap_or_default()),
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    post(
        c,
        "SALE_VOID",
        void_id,
        Some(sale),
        Some(&branch),
        "Sale Void reversal",
        Some(&original),
        cogs_complete,
        lines,
    )
}

pub fn accounts(db: &AppDb) -> Result<Vec<Value>, AppError> {
    let c = db.lock()?;
    let mut q=c.prepare("SELECT json_object('id',id,'code',code,'name',name,'accountType',account_type,'normalSide',normal_side,'isSystem',json(CASE is_system WHEN 1 THEN 'true' ELSE 'false' END),'isActive',json(CASE is_active WHEN 1 THEN 'true' ELSE 'false' END)) FROM finance_accounts ORDER BY code")?;
    let result = q.query_map([], row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(result)
}
fn expense_on(c: &Connection, value: &str) -> Result<Value, AppError> {
    id(value)?;
    c.query_row("SELECT json_object('id',e.id,'expenseNumber',e.expense_number,'requestId',e.request_id,'status',e.status,'branchId',e.branch_id,'branchName',b.name,'expenseAccountId',e.expense_account_id,'expenseAccountCode',a.code,'expenseAccountName',a.name,'paymentAccountId',e.payment_account_id,'paymentAccountCode',p.code,'paymentAccountName',p.name,'amountMinor',e.amount_minor,'currency',e.currency,'description',e.description,'journalId',e.journal_id,'createdBy',e.created_by,'createdAt',e.created_at,'postedAt',e.posted_at) FROM finance_expenses e LEFT JOIN branches b ON b.id=e.branch_id JOIN finance_accounts a ON a.id=e.expense_account_id JOIN finance_accounts p ON p.id=e.payment_account_id WHERE e.id=?1 OR e.expense_number=?1",[value],row).optional()?.ok_or_else(||AppError::new(ErrorCode::NotFound,"Expense not found."))
}
pub fn expense(db: &AppDb, value: &str) -> Result<Value, AppError> {
    let c = db.lock()?;
    expense_on(&c, value)
}
pub fn create_expense(db: &AppDb, input: &CreateExpense) -> Result<Value, AppError> {
    id(&input.id)?;
    id(&input.request_id)?;
    id(&input.expense_account_id)?;
    id(&input.payment_account_id)?;
    if let Some(v) = &input.branch_id {
        id(v)?;
    }
    required(&input.description)?;
    optional(&input.created_by)?;
    if !(1..=MAX_SAFE_MINOR).contains(&input.amount_minor) {
        return Err(invalid(
            "Expense amount must be a positive safe integer minor-unit value.",
        ));
    }
    let payload =
        serde_json::to_string(input).map_err(|_| invalid("Expense payload is invalid."))?;
    let mut c = db.lock()?;
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some((old, id)) = tx
        .query_row(
            "SELECT request_payload,id FROM finance_expenses WHERE request_id=?1",
            [&input.request_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if old != payload {
            return Err(AppError::new(
                ErrorCode::Conflict,
                "Expense request identity belongs to another payload.",
            ));
        }
        return expense_on(&tx, &id);
    }
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM finance_expenses WHERE id=?1)",
        [&input.id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(AppError::new(
            ErrorCode::Conflict,
            "Expense ID already exists.",
        ));
    }
    let day: String = tx.query_row("SELECT strftime('%Y%m%d','now')", [], |r| r.get(0))?;
    tx.execute("INSERT INTO finance_expense_sequences(day,next_number) VALUES(?1,1) ON CONFLICT(day) DO UPDATE SET next_number=next_number+1",[&day])?;
    let n: i64 = tx.query_row(
        "SELECT next_number FROM finance_expense_sequences WHERE day=?1",
        [&day],
        |r| r.get(0),
    )?;
    let expense_number = format!("EXP-{day}-{n:05}");
    tx.execute("INSERT INTO finance_expenses(id,expense_number,request_id,request_payload,branch_id,expense_account_id,payment_account_id,amount_minor,description,created_by) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![input.id,expense_number,input.request_id,payload,input.branch_id,input.expense_account_id,input.payment_account_id,input.amount_minor,input.description,input.created_by])?;
    let expense_code: String = tx.query_row(
        "SELECT code FROM finance_accounts WHERE id=?1 AND account_type='EXPENSE' AND is_active=1",
        [&input.expense_account_id],
        |r| r.get(0),
    )?;
    let payment_code: String = tx.query_row(
        "SELECT code FROM finance_accounts WHERE id=?1 AND account_type='ASSET' AND is_active=1",
        [&input.payment_account_id],
        |r| r.get(0),
    )?;
    let expense_static = match expense_code.as_str() {
        "6100" => "6100",
        "6110" => "6110",
        "6120" => "6120",
        "6130" => "6130",
        "6190" => "6190",
        _ => return Err(invalid("Select a supported operating Expense account.")),
    };
    let payment_static = match payment_code.as_str() {
        "1000" => CASH,
        "1010" => CARD,
        "1020" => OTHER,
        _ => return Err(invalid("Select a supported recorded payment asset.")),
    };
    let journal = post(
        &tx,
        "EXPENSE",
        &input.id,
        None,
        input.branch_id.as_deref(),
        &input.description,
        None,
        true,
        vec![
            Line {
                code: expense_static,
                side: Side::Debit,
                amount: input.amount_minor,
                memo: input.description.clone(),
            },
            Line {
                code: payment_static,
                side: Side::Credit,
                amount: input.amount_minor,
                memo: "Recorded Expense payment".into(),
            },
        ],
    )?
    .ok_or_else(|| invalid("Expense journal cannot be empty."))?;
    tx.execute("UPDATE finance_expenses SET status='POSTED',journal_id=?2,posted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![input.id,journal])?;
    let result = expense_on(&tx, &input.id)?;
    tx.commit()?;
    Ok(result)
}

pub fn expenses(db: &AppDb, q: &FinanceQuery) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, q)?;
    let filter="e.status='POSTED' AND (?1 IS NULL OR substr(e.posted_at,1,10)>=?1) AND (?2 IS NULL OR substr(e.posted_at,1,10)<=?2) AND (?3 IS NULL OR e.branch_id=?3) AND (?4 IS NULL OR e.expense_account_id=?4) AND (?5='' OR instr(lower(e.expense_number||' '||e.description||' '||a.name),lower(?5))>0)";
    let total=c.query_row(&format!("SELECT count(*) FROM finance_expenses e JOIN finance_accounts a ON a.id=e.expense_account_id WHERE {filter}"),params![q.date_from,q.date_to,q.branch_id,q.account_id,s],|r|r.get(0))?;
    let mut stmt=c.prepare(&format!("SELECT json_object('id',e.id,'expenseNumber',e.expense_number,'branchId',e.branch_id,'branchName',b.name,'expenseAccountId',a.id,'expenseAccountCode',a.code,'expenseAccountName',a.name,'paymentAccountId',p.id,'paymentAccountName',p.name,'amountMinor',e.amount_minor,'description',e.description,'journalId',e.journal_id,'postedAt',e.posted_at) FROM finance_expenses e LEFT JOIN branches b ON b.id=e.branch_id JOIN finance_accounts a ON a.id=e.expense_account_id JOIN finance_accounts p ON p.id=e.payment_account_id WHERE {filter} ORDER BY e.posted_at DESC,e.id DESC LIMIT ?6 OFFSET ?7"))?;
    let items = stmt
        .query_map(
            params![q.date_from, q.date_to, q.branch_id, q.account_id, s, l, o],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}
fn journal_on(c: &Connection, value: &str) -> Result<Value, AppError> {
    id(value)?;
    let mut result:Value=c.query_row("SELECT json_object('id',j.id,'journalNumber',j.journal_number,'status',j.status,'sourceType',j.source_type,'sourceId',j.source_id,'sourceParentId',j.source_parent_id,'branchId',j.branch_id,'branchName',b.name,'reversalOfJournalId',j.reversal_of_journal_id,'description',j.description,'currency',j.currency,'debitTotalMinor',j.debit_total_minor,'creditTotalMinor',j.credit_total_minor,'cogsComplete',json(CASE j.cogs_complete WHEN 1 THEN 'true' ELSE 'false' END),'postedAt',j.posted_at) FROM finance_journals j LEFT JOIN branches b ON b.id=j.branch_id WHERE j.id=?1 OR j.journal_number=?1",[value],row).optional()?.ok_or_else(||AppError::new(ErrorCode::NotFound,"Finance journal not found."))?;
    let jid = result["id"].as_str().unwrap_or("");
    let mut q=c.prepare("SELECT json_object('id',l.id,'position',l.position,'accountId',a.id,'accountCode',a.code,'accountName',a.name,'accountType',a.account_type,'side',l.side,'amountMinor',l.amount_minor,'memo',l.memo) FROM finance_journal_lines l JOIN finance_accounts a ON a.id=l.account_id WHERE l.journal_id=?1 ORDER BY l.position")?;
    result["lines"] = json!(q.query_map([jid], row)?.collect::<Result<Vec<_>, _>>()?);
    Ok(result)
}
pub fn journal(db: &AppDb, value: &str) -> Result<Value, AppError> {
    let c = db.lock()?;
    journal_on(&c, value)
}
pub fn journals(db: &AppDb, q: &FinanceQuery) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, q)?;
    if q.source_type
        .as_ref()
        .is_some_and(|v| !["SALE", "SALE_RETURN", "SALE_VOID", "EXPENSE"].contains(&v.as_str()))
    {
        return Err(invalid("Invalid Finance source type."));
    }
    let filter="j.status='POSTED' AND (?1 IS NULL OR substr(j.posted_at,1,10)>=?1) AND (?2 IS NULL OR substr(j.posted_at,1,10)<=?2) AND (?3 IS NULL OR j.branch_id=?3) AND (?4 IS NULL OR j.source_type=?4) AND (?5='' OR instr(lower(j.journal_number||' '||j.description||' '||j.source_id),lower(?5))>0)";
    let total = c.query_row(
        &format!("SELECT count(*) FROM finance_journals j WHERE {filter}"),
        params![q.date_from, q.date_to, q.branch_id, q.source_type, s],
        |r| r.get(0),
    )?;
    let mut stmt=c.prepare(&format!("SELECT json_object('id',j.id,'journalNumber',j.journal_number,'sourceType',j.source_type,'sourceId',j.source_id,'sourceParentId',j.source_parent_id,'branchId',j.branch_id,'branchName',b.name,'description',j.description,'debitTotalMinor',j.debit_total_minor,'creditTotalMinor',j.credit_total_minor,'cogsComplete',json(CASE j.cogs_complete WHEN 1 THEN 'true' ELSE 'false' END),'reversalOfJournalId',j.reversal_of_journal_id,'postedAt',j.posted_at) FROM finance_journals j LEFT JOIN branches b ON b.id=j.branch_id WHERE {filter} ORDER BY j.posted_at DESC,j.id DESC LIMIT ?6 OFFSET ?7"))?;
    let items = stmt
        .query_map(
            params![q.date_from, q.date_to, q.branch_id, q.source_type, s, l, o],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}

const REPORT_SCOPE:&str="j.status='POSTED' AND (?1 IS NULL OR substr(j.posted_at,1,10)>=?1) AND (?2 IS NULL OR substr(j.posted_at,1,10)<=?2) AND (?3 IS NULL OR j.branch_id=?3)";
pub fn trial_balance(db: &AppDb, q: &FinanceQuery) -> Result<Value, AppError> {
    let c = db.lock()?;
    validate(&c, q)?;
    let mut stmt=c.prepare(&format!("WITH activity AS (SELECT l.account_id,l.side,l.amount_minor FROM finance_journal_lines l JOIN finance_journals j ON j.id=l.journal_id WHERE {REPORT_SCOPE}) SELECT json_object('accountId',a.id,'accountCode',a.code,'accountName',a.name,'accountType',a.account_type,'normalSide',a.normal_side,'debitMinor',coalesce(sum(CASE WHEN l.side='DEBIT' THEN l.amount_minor ELSE 0 END),0),'creditMinor',coalesce(sum(CASE WHEN l.side='CREDIT' THEN l.amount_minor ELSE 0 END),0),'balanceMinor',CASE WHEN a.normal_side='DEBIT' THEN coalesce(sum(CASE WHEN l.side='DEBIT' THEN l.amount_minor ELSE -l.amount_minor END),0) ELSE coalesce(sum(CASE WHEN l.side='CREDIT' THEN l.amount_minor ELSE -l.amount_minor END),0) END) FROM finance_accounts a LEFT JOIN activity l ON l.account_id=a.id GROUP BY a.id ORDER BY a.code"))?;
    let items = stmt
        .query_map(params![q.date_from, q.date_to, q.branch_id], row)?
        .collect::<Result<Vec<_>, _>>()?;
    let(debit,credit):(i64,i64)=c.query_row(&format!("SELECT coalesce(sum(CASE WHEN l.side='DEBIT' THEN l.amount_minor ELSE 0 END),0),coalesce(sum(CASE WHEN l.side='CREDIT' THEN l.amount_minor ELSE 0 END),0) FROM finance_journal_lines l JOIN finance_journals j ON j.id=l.journal_id WHERE {REPORT_SCOPE}"),params![q.date_from,q.date_to,q.branch_id],|r|Ok((r.get(0)?,r.get(1)?)))?;
    Ok(
        json!({"items":items,"debitTotalMinor":debit,"creditTotalMinor":credit,"balanced":debit==credit}),
    )
}
pub fn account_activity(db: &AppDb, q: &FinanceQuery) -> Result<Value, AppError> {
    let c = db.lock()?;
    let (l, o, s) = validate(&c, q)?;
    let account_id = q
        .account_id
        .as_ref()
        .ok_or_else(|| invalid("Select a Finance account."))?;
    let filter=format!("{REPORT_SCOPE} AND l.account_id=?4 AND (?5='' OR instr(lower(j.journal_number||' '||j.description||' '||coalesce(l.memo,'')),lower(?5))>0)");
    let total=c.query_row(&format!("SELECT count(*) FROM finance_journal_lines l JOIN finance_journals j ON j.id=l.journal_id WHERE {filter}"),params![q.date_from,q.date_to,q.branch_id,account_id,s],|r|r.get(0))?;
    let mut stmt=c.prepare(&format!("SELECT json_object('id',l.id,'journalId',j.id,'journalNumber',j.journal_number,'sourceType',j.source_type,'sourceId',j.source_id,'sourceParentId',j.source_parent_id,'branchId',j.branch_id,'description',j.description,'side',l.side,'amountMinor',l.amount_minor,'memo',l.memo,'postedAt',j.posted_at) FROM finance_journal_lines l JOIN finance_journals j ON j.id=l.journal_id WHERE {filter} ORDER BY j.posted_at DESC,j.id DESC,l.position LIMIT ?6 OFFSET ?7"))?;
    let items = stmt
        .query_map(
            params![q.date_from, q.date_to, q.branch_id, account_id, s, l, o],
            row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(page(items, total, l, o))
}
fn net(c: &Connection, code: &str, debit_normal: bool, q: &FinanceQuery) -> Result<i64, AppError> {
    let(d,cr):(i64,i64)=c.query_row(&format!("SELECT coalesce(sum(CASE WHEN l.side='DEBIT' THEN l.amount_minor ELSE 0 END),0),coalesce(sum(CASE WHEN l.side='CREDIT' THEN l.amount_minor ELSE 0 END),0) FROM finance_journal_lines l JOIN finance_accounts a ON a.id=l.account_id JOIN finance_journals j ON j.id=l.journal_id WHERE {REPORT_SCOPE} AND a.code=?4"),params![q.date_from,q.date_to,q.branch_id,code],|r|Ok((r.get(0)?,r.get(1)?)))?;
    Ok(if debit_normal { d - cr } else { cr - d })
}
pub fn sales_summary(db: &AppDb, q: &FinanceQuery) -> Result<Value, AppError> {
    let c = db.lock()?;
    validate(&c, q)?;
    let revenue = net(&c, REVENUE, false, q)?;
    let returns = net(&c, RETURNS, true, q)?;
    let net_revenue = revenue - returns;
    let cogs = net(&c, COGS, true, q)?;
    let refunds = net(&c, REFUND, false, q)?;
    let expenses:i64=c.query_row(&format!("SELECT coalesce(sum(CASE WHEN l.side='DEBIT' THEN l.amount_minor ELSE -l.amount_minor END),0) FROM finance_journal_lines l JOIN finance_accounts a ON a.id=l.account_id JOIN finance_journals j ON j.id=l.journal_id WHERE {REPORT_SCOPE} AND a.account_type='EXPENSE' AND a.code LIKE '6%'"),params![q.date_from,q.date_to,q.branch_id],|r|r.get(0))?;
    let missing:i64=c.query_row(&format!("SELECT count(*) FROM finance_journals j WHERE {REPORT_SCOPE} AND j.source_type IN ('SALE','SALE_RETURN') AND j.cogs_complete=0"),params![q.date_from,q.date_to,q.branch_id],|r|r.get(0))?;
    let margin = if missing == 0 {
        Some(net_revenue - cogs)
    } else {
        None
    };
    Ok(
        json!({"salesRevenueMinor":revenue,"salesReturnsMinor":returns,"netSalesRevenueMinor":net_revenue,"refundOutflowMinor":refunds,"netCogsMinor":cogs,"missingCogsJournalCount":missing,"grossMarginMinor":margin,"operatingExpensesMinor":expenses,"operatingResultMinor":margin.map(|v|v-expenses)}),
    )
}
pub fn expense_report(db: &AppDb, q: &FinanceQuery) -> Result<Value, AppError> {
    let list = expenses(db, q)?;
    let c = db.lock()?;
    let total:i64=c.query_row("SELECT coalesce(sum(e.amount_minor),0) FROM finance_expenses e WHERE e.status='POSTED' AND (?1 IS NULL OR substr(e.posted_at,1,10)>=?1) AND (?2 IS NULL OR substr(e.posted_at,1,10)<=?2) AND (?3 IS NULL OR e.branch_id=?3) AND (?4 IS NULL OR e.expense_account_id=?4)",params![q.date_from,q.date_to,q.branch_id,q.account_id],|r|r.get(0))?;
    let mut result = list;
    result["totalExpenseMinor"] = json!(total);
    Ok(result)
}
