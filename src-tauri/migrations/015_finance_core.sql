CREATE TABLE finance_accounts (
 id TEXT PRIMARY KEY NOT NULL,
 code TEXT NOT NULL UNIQUE COLLATE NOCASE,
 name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 180),
 account_type TEXT NOT NULL CHECK(account_type IN ('ASSET','LIABILITY','EQUITY','REVENUE','EXPENSE')),
 normal_side TEXT NOT NULL CHECK(normal_side IN ('DEBIT','CREDIT')),
 is_system INTEGER NOT NULL DEFAULT 0 CHECK(is_system IN (0,1)),
 is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 CHECK((account_type IN ('ASSET','EXPENSE') AND normal_side='DEBIT') OR (account_type IN ('LIABILITY','EQUITY','REVENUE') AND normal_side='CREDIT'))
) STRICT;
CREATE INDEX finance_accounts_list ON finance_accounts(account_type,is_active,code);

CREATE TABLE finance_journal_sequences (
 day TEXT PRIMARY KEY NOT NULL,
 next_number INTEGER NOT NULL CHECK(next_number>0)
) STRICT;

CREATE TABLE finance_journals (
 id TEXT PRIMARY KEY NOT NULL,
 journal_number TEXT NOT NULL UNIQUE,
 status TEXT NOT NULL DEFAULT 'POSTING' CHECK(status IN ('POSTING','POSTED')),
 source_type TEXT NOT NULL CHECK(source_type IN ('SALE','SALE_RETURN','SALE_VOID','EXPENSE')),
 source_id TEXT NOT NULL,
 source_parent_id TEXT,
 branch_id TEXT REFERENCES branches(id) ON DELETE RESTRICT,
 reversal_of_journal_id TEXT REFERENCES finance_journals(id) ON DELETE RESTRICT,
 description TEXT NOT NULL CHECK(length(trim(description)) BETWEEN 1 AND 4000),
 currency TEXT NOT NULL DEFAULT 'EGP' CHECK(currency='EGP'),
 debit_total_minor INTEGER NOT NULL CHECK(debit_total_minor BETWEEN 1 AND 9007199254740991),
 credit_total_minor INTEGER NOT NULL CHECK(credit_total_minor BETWEEN 1 AND 9007199254740991),
 cogs_complete INTEGER NOT NULL DEFAULT 1 CHECK(cogs_complete IN (0,1)),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 posted_at TEXT,
 CHECK(debit_total_minor=credit_total_minor),
 CHECK((status='POSTED')=(posted_at IS NOT NULL)),
 CHECK(reversal_of_journal_id IS NULL OR source_type='SALE_VOID'),
 UNIQUE(source_type,source_id)
) STRICT;
CREATE INDEX finance_journals_history ON finance_journals(posted_at DESC,id DESC);
CREATE INDEX finance_journals_branch ON finance_journals(branch_id,posted_at DESC,id DESC);
CREATE INDEX finance_journals_source_parent ON finance_journals(source_parent_id,posted_at,id);

CREATE TABLE finance_journal_lines (
 id TEXT PRIMARY KEY NOT NULL,
 journal_id TEXT NOT NULL REFERENCES finance_journals(id) ON DELETE RESTRICT,
 position INTEGER NOT NULL CHECK(position>=0),
 account_id TEXT NOT NULL REFERENCES finance_accounts(id) ON DELETE RESTRICT,
 side TEXT NOT NULL CHECK(side IN ('DEBIT','CREDIT')),
 amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 1 AND 9007199254740991),
 memo TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(journal_id,position)
) STRICT;
CREATE INDEX finance_journal_lines_account ON finance_journal_lines(account_id,journal_id,position);

CREATE TABLE finance_expense_sequences (
 day TEXT PRIMARY KEY NOT NULL,
 next_number INTEGER NOT NULL CHECK(next_number>0)
) STRICT;

CREATE TABLE finance_expenses (
 id TEXT PRIMARY KEY NOT NULL,
 expense_number TEXT NOT NULL UNIQUE,
 request_id TEXT NOT NULL UNIQUE,
 request_payload TEXT NOT NULL CHECK(json_valid(request_payload)),
 status TEXT NOT NULL DEFAULT 'POSTING' CHECK(status IN ('POSTING','POSTED')),
 branch_id TEXT REFERENCES branches(id) ON DELETE RESTRICT,
 expense_account_id TEXT NOT NULL REFERENCES finance_accounts(id) ON DELETE RESTRICT,
 payment_account_id TEXT NOT NULL REFERENCES finance_accounts(id) ON DELETE RESTRICT,
 amount_minor INTEGER NOT NULL CHECK(amount_minor BETWEEN 1 AND 9007199254740991),
 currency TEXT NOT NULL DEFAULT 'EGP' CHECK(currency='EGP'),
 description TEXT NOT NULL CHECK(length(trim(description)) BETWEEN 1 AND 4000),
 journal_id TEXT UNIQUE REFERENCES finance_journals(id) ON DELETE RESTRICT,
 created_by TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 posted_at TEXT,
 CHECK((status='POSTED')=(posted_at IS NOT NULL AND journal_id IS NOT NULL))
) STRICT;
CREATE INDEX finance_expenses_history ON finance_expenses(posted_at DESC,id DESC);
CREATE INDEX finance_expenses_branch ON finance_expenses(branch_id,posted_at DESC,id DESC);
CREATE INDEX finance_expenses_account ON finance_expenses(expense_account_id,posted_at DESC,id DESC);

CREATE TRIGGER finance_account_no_delete BEFORE DELETE ON finance_accounts BEGIN
 SELECT RAISE(ABORT,'finance_account_history_must_be_retained');
END;
CREATE TRIGGER finance_system_account_immutable BEFORE UPDATE ON finance_accounts WHEN OLD.is_system=1 BEGIN
 SELECT RAISE(ABORT,'finance_system_account_immutable');
END;

CREATE TRIGGER finance_journal_line_insert_guard BEFORE INSERT ON finance_journal_lines BEGIN
 SELECT CASE WHEN (SELECT status FROM finance_journals WHERE id=NEW.journal_id)!='POSTING' THEN RAISE(ABORT,'finance_journal_lines_require_posting') END;
 SELECT CASE WHEN (SELECT is_active FROM finance_accounts WHERE id=NEW.account_id)!=1 THEN RAISE(ABORT,'finance_account_inactive') END;
 SELECT CASE WHEN NEW.side='DEBIT' AND NEW.amount_minor+coalesce((SELECT sum(amount_minor) FROM finance_journal_lines WHERE journal_id=NEW.journal_id AND side='DEBIT'),0)>(SELECT debit_total_minor FROM finance_journals WHERE id=NEW.journal_id) THEN RAISE(ABORT,'finance_journal_debit_exceeded') END;
 SELECT CASE WHEN NEW.side='CREDIT' AND NEW.amount_minor+coalesce((SELECT sum(amount_minor) FROM finance_journal_lines WHERE journal_id=NEW.journal_id AND side='CREDIT'),0)>(SELECT credit_total_minor FROM finance_journals WHERE id=NEW.journal_id) THEN RAISE(ABORT,'finance_journal_credit_exceeded') END;
END;
CREATE TRIGGER finance_journal_post_guard BEFORE UPDATE OF status ON finance_journals WHEN NEW.status='POSTED' BEGIN
 SELECT CASE WHEN OLD.status!='POSTING' OR NEW.posted_at IS NULL THEN RAISE(ABORT,'finance_journal_invalid_transition') END;
 SELECT CASE WHEN (SELECT count(*) FROM finance_journal_lines WHERE journal_id=OLD.id)<2 THEN RAISE(ABORT,'finance_journal_requires_lines') END;
 SELECT CASE WHEN (SELECT coalesce(sum(amount_minor),0) FROM finance_journal_lines WHERE journal_id=OLD.id AND side='DEBIT')!=OLD.debit_total_minor THEN RAISE(ABORT,'finance_journal_unbalanced_debit') END;
 SELECT CASE WHEN (SELECT coalesce(sum(amount_minor),0) FROM finance_journal_lines WHERE journal_id=OLD.id AND side='CREDIT')!=OLD.credit_total_minor THEN RAISE(ABORT,'finance_journal_unbalanced_credit') END;
END;
CREATE TRIGGER finance_journal_guard BEFORE UPDATE ON finance_journals BEGIN
 SELECT CASE WHEN OLD.status='POSTED' THEN RAISE(ABORT,'finance_journal_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.journal_number IS NOT OLD.journal_number OR NEW.source_type IS NOT OLD.source_type OR NEW.source_id IS NOT OLD.source_id OR NEW.source_parent_id IS NOT OLD.source_parent_id OR NEW.branch_id IS NOT OLD.branch_id OR NEW.reversal_of_journal_id IS NOT OLD.reversal_of_journal_id OR NEW.description IS NOT OLD.description OR NEW.currency IS NOT OLD.currency OR NEW.debit_total_minor IS NOT OLD.debit_total_minor OR NEW.credit_total_minor IS NOT OLD.credit_total_minor OR NEW.cogs_complete IS NOT OLD.cogs_complete OR NEW.created_at IS NOT OLD.created_at THEN RAISE(ABORT,'finance_journal_identity_immutable') END;
END;
CREATE TRIGGER finance_journal_no_delete BEFORE DELETE ON finance_journals BEGIN SELECT RAISE(ABORT,'finance_journal_immutable'); END;
CREATE TRIGGER finance_journal_line_no_update BEFORE UPDATE ON finance_journal_lines BEGIN SELECT RAISE(ABORT,'finance_journal_line_immutable'); END;
CREATE TRIGGER finance_journal_line_no_delete BEFORE DELETE ON finance_journal_lines BEGIN SELECT RAISE(ABORT,'finance_journal_line_immutable'); END;

CREATE TRIGGER finance_expense_insert_guard BEFORE INSERT ON finance_expenses BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM finance_accounts WHERE id=NEW.expense_account_id AND account_type='EXPENSE' AND is_active=1) THEN RAISE(ABORT,'finance_expense_account_invalid') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM finance_accounts WHERE id=NEW.payment_account_id AND account_type='ASSET' AND is_active=1) THEN RAISE(ABORT,'finance_payment_account_invalid') END;
 SELECT CASE WHEN NEW.branch_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM branches WHERE id=NEW.branch_id AND is_active=1) THEN RAISE(ABORT,'finance_expense_branch_invalid') END;
END;
CREATE TRIGGER finance_expense_guard BEFORE UPDATE ON finance_expenses BEGIN
 SELECT CASE WHEN OLD.status!='POSTING' OR NEW.status!='POSTED' THEN RAISE(ABORT,'finance_expense_immutable') END;
 SELECT CASE WHEN NEW.id IS NOT OLD.id OR NEW.expense_number IS NOT OLD.expense_number OR NEW.request_id IS NOT OLD.request_id OR NEW.request_payload IS NOT OLD.request_payload OR NEW.branch_id IS NOT OLD.branch_id OR NEW.expense_account_id IS NOT OLD.expense_account_id OR NEW.payment_account_id IS NOT OLD.payment_account_id OR NEW.amount_minor IS NOT OLD.amount_minor OR NEW.currency IS NOT OLD.currency OR NEW.description IS NOT OLD.description OR NEW.created_by IS NOT OLD.created_by OR NEW.created_at IS NOT OLD.created_at THEN RAISE(ABORT,'finance_expense_identity_immutable') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM finance_journals WHERE id=NEW.journal_id AND status='POSTED' AND source_type='EXPENSE' AND source_id=OLD.id AND debit_total_minor=OLD.amount_minor) THEN RAISE(ABORT,'finance_expense_journal_mismatch') END;
END;
CREATE TRIGGER finance_expense_no_delete BEFORE DELETE ON finance_expenses BEGIN SELECT RAISE(ABORT,'finance_expense_immutable'); END;

INSERT INTO finance_accounts(id,code,name,account_type,normal_side,is_system) VALUES
 ('00000000-0000-4000-8000-00000000f001','1000','Recorded Cash Sale Payments','ASSET','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f002','1010','Recorded Card Sale Payments','ASSET','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f003','1020','Recorded Other Sale Payments','ASSET','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f004','1090','Unallocated Customer Refund Clearing','ASSET','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f005','1200','Inventory','ASSET','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f006','2100','Recorded Sales Tax Clearing','LIABILITY','CREDIT',1),
 ('00000000-0000-4000-8000-00000000f007','4000','Sales Revenue','REVENUE','CREDIT',1),
 ('00000000-0000-4000-8000-00000000f008','4010','Sales Returns','REVENUE','CREDIT',1),
 ('00000000-0000-4000-8000-00000000f009','5000','Cost of Goods Sold','EXPENSE','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f010','6100','Rent Expense','EXPENSE','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f011','6110','Utilities Expense','EXPENSE','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f012','6120','Transport Expense','EXPENSE','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f013','6130','Maintenance Expense','EXPENSE','DEBIT',1),
 ('00000000-0000-4000-8000-00000000f014','6190','Miscellaneous Operating Expense','EXPENSE','DEBIT',1);
