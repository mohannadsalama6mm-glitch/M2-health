-- Phase 5E: link canonical customers to sales without rewriting historical rows.
-- The Phase 4 customer_id placeholder deliberately rejected every non-NULL value.
DROP TRIGGER sales_frozen;
ALTER TABLE sales DROP COLUMN customer_id;
ALTER TABLE sales ADD COLUMN customer_id TEXT REFERENCES customers(id) ON DELETE RESTRICT;
ALTER TABLE sales ADD COLUMN customer_code_snapshot TEXT;
ALTER TABLE sales ADD COLUMN customer_phone_snapshot TEXT;

CREATE INDEX sales_customer_history
  ON sales(customer_id, created_at DESC, id DESC)
  WHERE customer_id IS NOT NULL;

CREATE TRIGGER sales_customer_shape
BEFORE INSERT ON sales
WHEN (NEW.customer_id IS NULL) != (NEW.customer_name IS NULL)
BEGIN
  SELECT RAISE(ABORT,'Customer link and snapshot must be supplied together');
END;

CREATE TRIGGER sales_frozen BEFORE UPDATE ON sales WHEN OLD.status IN ('POSTED','VOIDED') AND NOT (
 OLD.status='POSTED' AND NEW.status='VOIDED' AND NEW.voided_at IS NOT NULL AND NEW.payment_status='REFUNDED'
 AND NEW.id IS OLD.id AND NEW.sale_number IS OLD.sale_number AND NEW.branch_id IS OLD.branch_id
 AND NEW.customer_id IS OLD.customer_id AND NEW.customer_name IS OLD.customer_name
 AND NEW.customer_code_snapshot IS OLD.customer_code_snapshot AND NEW.customer_phone_snapshot IS OLD.customer_phone_snapshot
 AND NEW.draft_json IS OLD.draft_json
 AND NEW.subtotal_minor IS OLD.subtotal_minor AND NEW.discount_minor IS OLD.discount_minor AND NEW.tax_minor IS OLD.tax_minor AND NEW.total_minor IS OLD.total_minor
 AND NEW.created_by IS OLD.created_by AND NEW.notes IS OLD.notes AND NEW.post_request_id IS OLD.post_request_id AND NEW.created_at IS OLD.created_at AND NEW.posted_at IS OLD.posted_at AND NEW.revision=OLD.revision+1
 AND EXISTS(SELECT 1 FROM sale_voids WHERE sale_id=OLD.id))
BEGIN SELECT RAISE(ABORT,'Posted sales are immutable'); END;
