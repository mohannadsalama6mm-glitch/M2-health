use super::outbox;
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

#[derive(Debug)]
pub struct Failure {
    pub reason: &'static str,
    pub detail: &'static str,
}

fn invalid() -> Failure {
    Failure {
        reason: "INVALID_PAYLOAD",
        detail: "The remote event payload did not match the versioned domain contract.",
    }
}

fn immutable() -> Failure {
    Failure {
        reason: "IMMUTABLE_CONFLICT",
        detail: "The immutable transaction identity already exists with different content.",
    }
}

fn missing() -> Failure {
    Failure {
        reason: "MISSING_DEPENDENCY",
        detail:
            "A required local branch, catalog, batch, account, or prior transaction is unavailable.",
    }
}

fn conflict() -> Failure {
    Failure {
        reason: "REVISION_CONFLICT",
        detail: "A concurrent catalog or partner revision already exists with different content.",
    }
}

fn json_query(connection: &Connection, sql: &str, id: &str) -> Result<Value, AppError> {
    let text: String = connection.query_row(sql, [id], |row| row.get(0))?;
    serde_json::from_str(&text).map_err(|_| {
        AppError::new(
            crate::error::ErrorCode::Validation,
            "The synchronization payload is invalid.",
        )
    })
}

fn json_rows(connection: &Connection, sql: &str, id: &str) -> Result<Vec<Value>, AppError> {
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([id], |row| row.get::<_, String>(0))?;
    let mut values = Vec::new();
    for row in rows {
        values.push(serde_json::from_str(&row?).map_err(|_| {
            AppError::new(
                crate::error::ErrorCode::Validation,
                "The synchronization payload is invalid.",
            )
        })?);
    }
    Ok(values)
}

fn emit(
    connection: &Connection,
    branch_id: Option<&str>,
    actor: Option<&str>,
    entity_type: &str,
    entity_id: &str,
    payload: &Value,
) -> Result<(), AppError> {
    let owned = actor
        .map(str::to_owned)
        .or_else(|| outbox::enrolled_actor(connection));
    outbox::enqueue(
        connection,
        branch_id,
        owned.as_deref(),
        entity_type,
        entity_id,
        "APPENDED",
        payload,
        None,
    )?;
    Ok(())
}

pub fn catalog_lookup(connection: &Connection, table: &str, id: &str) -> Result<(), AppError> {
    let sql = match table {
        "manufacturers" => "SELECT json_object('id',id,'name',name,'normalizedName',normalized_name,'country',country,'phone',phone,'email',email,'website',website,'isActive',is_active,'createdAt',created_at,'updatedAt',updated_at) FROM manufacturers WHERE id=?1",
        "categories" => "SELECT json_object('id',id,'name',name,'normalizedName',normalized_name,'description',description,'isActive',is_active,'createdAt',created_at,'updatedAt',updated_at) FROM categories WHERE id=?1",
        "routes" => "SELECT json_object('id',id,'name',name,'normalizedName',normalized_name,'isActive',is_active,'createdAt',created_at,'updatedAt',updated_at) FROM routes WHERE id=?1",
        "active_ingredients" => "SELECT json_object('id',id,'name',name,'normalizedName',normalized_name,'description',description,'isActive',is_active,'createdAt',created_at,'updatedAt',updated_at) FROM active_ingredients WHERE id=?1",
        _ => return Ok(()),
    };
    let row = json_query(connection, sql, id)?;
    emit(
        connection,
        None,
        None,
        "catalog_lookup",
        id,
        &json!({"table": table, "row": row}),
    )
}

pub fn catalog_product(connection: &Connection, product_id: &str) -> Result<(), AppError> {
    let product = json_query(connection, "SELECT json_object('id',id,'commercialNameEn',commercial_name_en,'commercialNameAr',commercial_name_ar,'normalizedNameEn',normalized_name_en,'normalizedNameAr',normalized_name_ar,'scientificName',scientific_name,'normalizedScientificName',normalized_scientific_name,'manufacturerId',manufacturer_id,'categoryId',category_id,'routeId',route_id,'description',description,'notes',notes,'isActive',is_active,'createdAt',created_at,'updatedAt',updated_at) FROM products WHERE id=?1", product_id)?;
    let payload = json!({
        "product": product,
        "ingredients": json_rows(connection, "SELECT json_object('id',id,'productId',product_id,'activeIngredientId',active_ingredient_id,'strengthText',strength_text,'position',position,'createdAt',created_at,'updatedAt',updated_at) FROM product_active_ingredients WHERE product_id=?1 ORDER BY position,id", product_id)?,
        "packages": json_rows(connection, "SELECT json_object('id',id,'productId',product_id,'packageLabel',package_label,'packSize',pack_size,'unitName',unit_name,'unitsPerPackage',units_per_package,'strengthText',strength_text,'isDefault',is_default,'isActive',is_active,'createdAt',created_at,'updatedAt',updated_at) FROM product_packages WHERE product_id=?1 ORDER BY created_at,id", product_id)?,
        "barcodes": json_rows(connection, "SELECT json_object('id',b.id,'productPackageId',b.product_package_id,'barcode',b.barcode,'isPrimary',b.is_primary,'createdAt',b.created_at) FROM barcodes b JOIN product_packages p ON p.id=b.product_package_id WHERE p.product_id=?1 ORDER BY b.created_at,b.id", product_id)?,
        "prices": json_rows(connection, "SELECT json_object('id',h.id,'productPackageId',h.product_package_id,'sellingPriceMinor',h.selling_price_minor,'costPriceMinor',h.cost_price_minor,'effectiveFrom',h.effective_from,'effectiveTo',h.effective_to,'reason',h.reason,'createdAt',h.created_at) FROM product_price_history h JOIN product_packages p ON p.id=h.product_package_id WHERE p.product_id=?1 ORDER BY h.created_at,h.id", product_id)?,
    });
    emit(
        connection,
        None,
        None,
        "catalog_product",
        product_id,
        &payload,
    )
}

pub fn partner_record(connection: &Connection, table: &str, id: &str) -> Result<(), AppError> {
    let sql = match table {
        "suppliers" => "SELECT json_object('id',id,'code',code,'displayName',display_name,'contactPerson',contact_person,'primaryPhone',primary_phone,'secondaryPhone',secondary_phone,'email',email,'address',address,'taxRegistrationId',tax_registration_id,'notes',notes,'isActive',is_active,'revision',revision,'createdAt',created_at,'updatedAt',updated_at) FROM suppliers WHERE id=?1",
        "customers" => "SELECT json_object('id',id,'code',code,'displayName',display_name,'primaryPhone',primary_phone,'secondaryPhone',secondary_phone,'email',email,'address',address,'notes',notes,'isActive',is_active,'revision',revision,'createdAt',created_at,'updatedAt',updated_at) FROM customers WHERE id=?1",
        _ => return Ok(()),
    };
    let row = json_query(connection, sql, id)?;
    emit(
        connection,
        None,
        None,
        "partner_record",
        id,
        &json!({"table": table, "row": row}),
    )
}

pub fn sale_document(connection: &Connection, sale_id: &str) -> Result<(), AppError> {
    let branch: String = connection.query_row(
        "SELECT branch_id FROM sales WHERE id=?1",
        [sale_id],
        |row| row.get(0),
    )?;
    let actor: Option<String> = connection
        .query_row(
            "SELECT created_by FROM sales WHERE id=?1",
            [sale_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    let payload = json!({
        "sale": json_query(connection, "SELECT json_object('id',id,'saleNumber',sale_number,'branchId',branch_id,'customerId',customer_id,'customerName',customer_name,'customerCodeSnapshot',customer_code_snapshot,'customerPhoneSnapshot',customer_phone_snapshot,'status',status,'revision',revision,'draftJson',draft_json,'subtotalMinor',subtotal_minor,'discountMinor',discount_minor,'taxMinor',tax_minor,'totalMinor',total_minor,'paymentStatus',payment_status,'createdBy',created_by,'notes',notes,'postRequestId',post_request_id,'createdAt',created_at,'postedAt',posted_at,'voidedAt',voided_at) FROM sales WHERE id=?1", sale_id)?,
        "items": json_rows(connection, "SELECT json_object('id',id,'saleId',sale_id,'position',position,'productId',product_id,'productPackageId',product_package_id,'productNameSnapshot',product_name_snapshot,'packageLabelSnapshot',package_label_snapshot,'quantity',quantity,'unitPriceMinor',unit_price_minor,'discountMinor',discount_minor,'saleDiscountMinor',sale_discount_minor,'taxMinor',tax_minor,'lineTotalMinor',line_total_minor,'unitCostMinor',unit_cost_minor,'createdAt',created_at) FROM sale_items WHERE sale_id=?1 ORDER BY position", sale_id)?,
        "allocations": json_rows(connection, "SELECT json_object('id',a.id,'saleItemId',a.sale_item_id,'position',a.position,'batchId',a.batch_id,'quantity',a.quantity,'unitCostMinor',a.unit_cost_minor,'movementId',a.movement_id) FROM sale_item_allocations a JOIN sale_items i ON i.id=a.sale_item_id WHERE i.sale_id=?1 ORDER BY i.position,a.position", sale_id)?,
        "payments": json_rows(connection, "SELECT json_object('id',id,'saleId',sale_id,'paymentMethod',payment_method,'amountMinor',amount_minor,'reference',reference,'createdAt',created_at) FROM sale_payments WHERE sale_id=?1 ORDER BY created_at,id", sale_id)?,
        "requests": json_rows(connection, "SELECT json_object('requestId',request_id,'kind',kind,'targetId',target_id,'payload',payload,'createdAt',created_at) FROM sale_requests WHERE target_id=?1", sale_id)?,
        "movements": json_rows(connection, "SELECT json_object('id',m.id,'branchId',m.branch_id,'productPackageId',m.product_package_id,'batchId',m.batch_id,'movementType',m.movement_type,'quantityDelta',m.quantity_delta,'referenceType',m.reference_type,'referenceId',m.reference_id,'reason',m.reason,'createdBy',m.created_by,'transferId',m.transfer_id,'createdAt',m.created_at) FROM inventory_movements m JOIN sale_item_allocations a ON a.movement_id=m.id JOIN sale_items i ON i.id=a.sale_item_id WHERE i.sale_id=?1", sale_id)?,
        "batches": json_rows(connection, "SELECT json_object('id',b.id,'branchId',b.branch_id,'productPackageId',b.product_package_id,'lotNumber',b.lot_number,'expiryDate',b.expiry_date,'receivedAt',b.received_at,'costPriceMinor',b.cost_price_minor,'supplierReference',b.supplier_reference,'isActive',b.is_active,'createdAt',b.created_at,'updatedAt',b.updated_at) FROM inventory_batches b WHERE b.id IN (SELECT a.batch_id FROM sale_item_allocations a JOIN sale_items i ON i.id=a.sale_item_id WHERE i.sale_id=?1)", sale_id)?,
    });
    emit(
        connection,
        Some(&branch),
        actor.as_deref(),
        "sale_document",
        sale_id,
        &payload,
    )
}

pub fn sale_return(connection: &Connection, return_id: &str) -> Result<(), AppError> {
    let (branch, actor): (String, Option<String>) = connection.query_row(
        "SELECT s.branch_id,s.created_by FROM sale_returns r JOIN sales s ON s.id=r.sale_id WHERE r.id=?1",
        [return_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let payload = json!({
        "header": json_query(connection, "SELECT json_object('id',id,'saleId',sale_id,'status',status,'reason',reason,'linesJson',lines_json,'refundMinor',refund_minor,'createdAt',created_at,'postedAt',posted_at) FROM sale_returns WHERE id=?1", return_id)?,
        "items": json_rows(connection, "SELECT json_object('id',id,'returnId',return_id,'saleItemId',sale_item_id,'quantity',quantity,'refundMinor',refund_minor) FROM sale_return_items WHERE return_id=?1", return_id)?,
        "allocations": json_rows(connection, "SELECT json_object('id',a.id,'returnItemId',a.return_item_id,'originalAllocationId',a.original_allocation_id,'quantity',a.quantity,'movementId',a.movement_id) FROM sale_return_allocations a JOIN sale_return_items i ON i.id=a.return_item_id WHERE i.return_id=?1", return_id)?,
        "refunds": json_rows(connection, "SELECT json_object('id',id,'saleId',sale_id,'returnId',return_id,'voidId',void_id,'amountMinor',amount_minor,'createdAt',created_at) FROM sale_refunds WHERE return_id=?1", return_id)?,
        "movements": json_rows(connection, "SELECT json_object('id',m.id,'branchId',m.branch_id,'productPackageId',m.product_package_id,'batchId',m.batch_id,'movementType',m.movement_type,'quantityDelta',m.quantity_delta,'referenceType',m.reference_type,'referenceId',m.reference_id,'reason',m.reason,'createdBy',m.created_by,'transferId',m.transfer_id,'createdAt',m.created_at) FROM inventory_movements m JOIN sale_return_allocations a ON a.movement_id=m.id JOIN sale_return_items i ON i.id=a.return_item_id WHERE i.return_id=?1", return_id)?,
    });
    emit(
        connection,
        Some(&branch),
        actor.as_deref(),
        "sale_return",
        return_id,
        &payload,
    )
}

pub fn purchase_receipt(connection: &Connection, receipt_id: &str) -> Result<(), AppError> {
    let (branch, actor): (String, Option<String>) = connection.query_row(
        "SELECT branch_id,created_by FROM purchase_receipts WHERE id=?1",
        [receipt_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let payload = json!({
        "receipt": json_query(connection, "SELECT json_object('id',id,'receiptNumber',receipt_number,'purchaseOrderId',purchase_order_id,'branchId',branch_id,'requestId',request_id,'requestPayload',request_payload,'totalQuantity',total_quantity,'status',status,'createdBy',created_by,'createdAt',created_at,'postedAt',posted_at) FROM purchase_receipts WHERE id=?1", receipt_id)?,
        "order": json_query(connection, "SELECT json_object('id',o.id,'purchaseNumber',o.purchase_number,'supplierId',o.supplier_id,'supplierCodeSnapshot',o.supplier_code_snapshot,'supplierNameSnapshot',o.supplier_name_snapshot,'status',o.status,'revision',o.revision,'supplierReference',o.supplier_reference,'expectedDate',o.expected_date,'notes',o.notes,'subtotalMinor',o.subtotal_minor,'discountMinor',o.discount_minor,'taxMinor',o.tax_minor,'totalMinor',o.total_minor,'createRequestId',o.create_request_id,'createdAt',o.created_at,'updatedAt',o.updated_at,'orderedAt',o.ordered_at) FROM purchase_orders o JOIN purchase_receipts r ON r.purchase_order_id=o.id WHERE r.id=?1", receipt_id)?,
        "orderLines": json_rows(connection, "SELECT json_object('id',l.id,'purchaseOrderId',l.purchase_order_id,'position',l.position,'productId',l.product_id,'productPackageId',l.product_package_id,'productNameSnapshot',l.product_name_snapshot,'packageLabelSnapshot',l.package_label_snapshot,'orderedQuantity',l.ordered_quantity,'unitCostMinor',l.unit_cost_minor,'lineTotalMinor',l.line_total_minor,'createdAt',l.created_at,'updatedAt',l.updated_at) FROM purchase_order_lines l JOIN purchase_receipts r ON r.purchase_order_id=l.purchase_order_id WHERE r.id=?1 ORDER BY l.position", receipt_id)?,
        "lines": json_rows(connection, "SELECT json_object('id',id,'receiptId',receipt_id,'purchaseOrderId',purchase_order_id,'branchId',branch_id,'purchaseOrderLineId',purchase_order_line_id,'position',position,'productPackageId',product_package_id,'productNameSnapshot',product_name_snapshot,'packageLabelSnapshot',package_label_snapshot,'quantity',quantity,'lotNumber',lot_number,'expiryDate',expiry_date,'acquisitionCostMinor',acquisition_cost_minor,'batchId',batch_id,'movementId',movement_id,'createdAt',created_at) FROM purchase_receipt_lines WHERE receipt_id=?1 ORDER BY position", receipt_id)?,
        "movements": json_rows(connection, "SELECT json_object('id',m.id,'branchId',m.branch_id,'productPackageId',m.product_package_id,'batchId',m.batch_id,'movementType',m.movement_type,'quantityDelta',m.quantity_delta,'referenceType',m.reference_type,'referenceId',m.reference_id,'reason',m.reason,'createdBy',m.created_by,'transferId',m.transfer_id,'createdAt',m.created_at) FROM inventory_movements m JOIN purchase_receipt_lines l ON l.movement_id=m.id WHERE l.receipt_id=?1", receipt_id)?,
        "batches": json_rows(connection, "SELECT json_object('id',b.id,'branchId',b.branch_id,'productPackageId',b.product_package_id,'lotNumber',b.lot_number,'expiryDate',b.expiry_date,'receivedAt',b.received_at,'costPriceMinor',b.cost_price_minor,'supplierReference',b.supplier_reference,'isActive',b.is_active,'createdAt',b.created_at,'updatedAt',b.updated_at) FROM inventory_batches b JOIN purchase_receipt_lines l ON l.batch_id=b.id WHERE l.receipt_id=?1", receipt_id)?,
    });
    emit(
        connection,
        Some(&branch),
        actor.as_deref(),
        "purchase_receipt",
        receipt_id,
        &payload,
    )
}

pub fn purchase_return(connection: &Connection, return_id: &str) -> Result<(), AppError> {
    let (branch, actor): (String, Option<String>) = connection.query_row(
        "SELECT branch_id,created_by FROM purchase_returns WHERE id=?1",
        [return_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let payload = json!({
        "header": json_query(connection, "SELECT json_object('id',id,'returnNumber',return_number,'purchaseOrderId',purchase_order_id,'receiptId',receipt_id,'branchId',branch_id,'supplierId',supplier_id,'requestId',request_id,'requestPayload',request_payload,'sourcePurchaseRevision',source_purchase_revision,'totalQuantity',total_quantity,'reason',reason,'status',status,'createdBy',created_by,'createdAt',created_at,'postedAt',posted_at) FROM purchase_returns WHERE id=?1", return_id)?,
        "lines": json_rows(connection, "SELECT json_object('id',id,'returnId',return_id,'purchaseOrderId',purchase_order_id,'receiptId',receipt_id,'branchId',branch_id,'receiptLineId',receipt_line_id,'purchaseOrderLineId',purchase_order_line_id,'position',position,'productPackageId',product_package_id,'productNameSnapshot',product_name_snapshot,'packageLabelSnapshot',package_label_snapshot,'previouslyReturnedQuantity',previously_returned_quantity,'quantity',quantity,'lotNumber',lot_number,'expiryDate',expiry_date,'acquisitionCostMinor',acquisition_cost_minor,'batchId',batch_id,'movementId',movement_id,'createdAt',created_at) FROM purchase_return_lines WHERE return_id=?1 ORDER BY position", return_id)?,
        "movements": json_rows(connection, "SELECT json_object('id',m.id,'branchId',m.branch_id,'productPackageId',m.product_package_id,'batchId',m.batch_id,'movementType',m.movement_type,'quantityDelta',m.quantity_delta,'referenceType',m.reference_type,'referenceId',m.reference_id,'reason',m.reason,'createdBy',m.created_by,'transferId',m.transfer_id,'createdAt',m.created_at) FROM inventory_movements m JOIN purchase_return_lines l ON l.movement_id=m.id WHERE l.return_id=?1", return_id)?,
    });
    emit(
        connection,
        Some(&branch),
        actor.as_deref(),
        "purchase_return",
        return_id,
        &payload,
    )
}

pub fn stock_count(connection: &Connection, session_id: &str) -> Result<(), AppError> {
    let (branch, actor): (String, Option<String>) = connection.query_row(
        "SELECT branch_id,created_by FROM stock_count_sessions WHERE id=?1",
        [session_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let payload = json!({
        "session": json_query(connection, "SELECT json_object('id',id,'sessionNumber',session_number,'branchId',branch_id,'status',status,'scopeType',scope_type,'notes',notes,'createdBy',created_by,'revision',revision,'snapshotWatermark',snapshot_watermark,'createdAt',created_at,'startedAt',started_at,'reviewedAt',reviewed_at,'completedAt',completed_at) FROM stock_count_sessions WHERE id=?1", session_id)?,
        "lines": json_rows(connection, "SELECT json_object('id',id,'sessionId',session_id,'branchId',branch_id,'productPackageId',product_package_id,'batchId',batch_id,'expectedQuantity',expected_quantity,'countedQuantity',counted_quantity,'notes',notes,'excludedReason',excluded_reason,'countedAt',counted_at,'revision',revision,'productLabel',product_label,'packageLabel',package_label) FROM stock_count_lines WHERE session_id=?1", session_id)?,
        "results": json_rows(connection, "SELECT json_object('lineId',r.line_id,'liveBaseline',r.live_baseline,'correctionDelta',r.correction_delta,'interveningMovements',r.intervening_movements,'movementId',r.movement_id) FROM stock_count_results r JOIN stock_count_lines l ON l.id=r.line_id WHERE l.session_id=?1", session_id)?,
        "movements": json_rows(connection, "SELECT json_object('id',m.id,'branchId',m.branch_id,'productPackageId',m.product_package_id,'batchId',m.batch_id,'movementType',m.movement_type,'quantityDelta',m.quantity_delta,'referenceType',m.reference_type,'referenceId',m.reference_id,'reason',m.reason,'createdBy',m.created_by,'transferId',m.transfer_id,'createdAt',m.created_at) FROM inventory_movements m JOIN stock_count_results r ON r.movement_id=m.id JOIN stock_count_lines l ON l.id=r.line_id WHERE l.session_id=?1", session_id)?,
    });
    emit(
        connection,
        Some(&branch),
        actor.as_deref(),
        "stock_count",
        session_id,
        &payload,
    )
}

pub fn inventory_document(connection: &Connection, document_id: &str) -> Result<(), AppError> {
    let (branch, actor): (String, Option<String>) = connection.query_row(
        "SELECT branch_id,created_by FROM inventory_documents WHERE id=?1",
        [document_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let payload = json!({
        "document": json_query(connection, "SELECT json_object('id',id,'documentNumber',document_number,'kind',kind,'branchId',branch_id,'destinationBranchId',destination_branch_id,'status',status,'revision',revision,'referenceText',reference_text,'supplierReference',supplier_reference,'notes',notes,'createdBy',created_by,'reversalOf',reversal_of,'reversalReason',reversal_reason,'createdAt',created_at,'updatedAt',updated_at,'postedAt',posted_at) FROM inventory_documents WHERE id=?1", document_id)?,
        "lines": json_rows(connection, "SELECT json_object('id',id,'documentId',document_id,'position',position,'productPackageId',product_package_id,'batchId',batch_id,'action',action,'quantity',quantity,'reason',reason,'lotNumber',lot_number,'expiryDate',expiry_date,'costPriceMinor',cost_price_minor,'supplierReference',supplier_reference,'productLabel',product_label,'packageLabel',package_label) FROM inventory_document_lines WHERE document_id=?1 ORDER BY position", document_id)?,
        "links": json_rows(connection, "SELECT json_object('movementId',movement_id,'documentId',document_id,'lineId',line_id,'reversesMovementId',reverses_movement_id) FROM inventory_document_movements WHERE document_id=?1 ORDER BY line_id,movement_id", document_id)?,
        "movements": json_rows(connection, "SELECT json_object('id',m.id,'branchId',m.branch_id,'productPackageId',m.product_package_id,'batchId',m.batch_id,'movementType',m.movement_type,'quantityDelta',m.quantity_delta,'referenceType',m.reference_type,'referenceId',m.reference_id,'reason',m.reason,'createdBy',m.created_by,'transferId',m.transfer_id,'createdAt',m.created_at) FROM inventory_movements m WHERE m.reference_type='inventory_document' AND m.reference_id=?1", document_id)?,
        "batches": json_rows(connection, "SELECT json_object('id',b.id,'branchId',b.branch_id,'productPackageId',b.product_package_id,'lotNumber',b.lot_number,'expiryDate',b.expiry_date,'receivedAt',b.received_at,'costPriceMinor',b.cost_price_minor,'supplierReference',b.supplier_reference,'isActive',b.is_active,'createdAt',b.created_at,'updatedAt',b.updated_at) FROM inventory_batches b WHERE b.id IN (SELECT m.batch_id FROM inventory_movements m WHERE m.reference_type='inventory_document' AND m.reference_id=?1)", document_id)?,
    });
    emit(
        connection,
        Some(&branch),
        actor.as_deref(),
        "inventory_document",
        document_id,
        &payload,
    )
}

fn text<'a>(row: &'a Value, key: &str) -> Result<&'a str, Failure> {
    row.get(key).and_then(Value::as_str).ok_or_else(invalid)
}

fn exists(connection: &Connection, sql: &str, id: &str) -> Result<bool, Failure> {
    connection
        .query_row(sql, [id], |row| row.get(0))
        .map_err(|_| missing())
}

fn insert_lookup(connection: &Connection, table: &str, row: &Value) -> Result<(), Failure> {
    let id = text(row, "id")?;
    match table {
        "manufacturers" => {
            if exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM manufacturers WHERE id=?1)",
                id,
            )? {
                let same: bool = connection
                    .query_row(
                        "SELECT name=?2 AND normalized_name=?3 FROM manufacturers WHERE id=?1",
                        params![id, text(row, "name")?, text(row, "normalizedName")?],
                        |r| r.get(0),
                    )
                    .map_err(|_| missing())?;
                return if same { Ok(()) } else { Err(conflict()) };
            }
            connection.execute("INSERT INTO manufacturers(id,name,normalized_name,country,phone,email,website,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![id, text(row,"name")?, text(row,"normalizedName")?, row.get("country").and_then(Value::as_str), row.get("phone").and_then(Value::as_str), row.get("email").and_then(Value::as_str), row.get("website").and_then(Value::as_str), row.get("isActive").and_then(Value::as_i64).unwrap_or(1), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
        }
        "categories" => {
            if exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM categories WHERE id=?1)",
                id,
            )? {
                let same: bool = connection
                    .query_row(
                        "SELECT name=?2 FROM categories WHERE id=?1",
                        params![id, text(row, "name")?],
                        |r| r.get(0),
                    )
                    .map_err(|_| missing())?;
                return if same { Ok(()) } else { Err(conflict()) };
            }
            connection.execute("INSERT INTO categories(id,name,normalized_name,description,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![id, text(row,"name")?, text(row,"normalizedName")?, row.get("description").and_then(Value::as_str), row.get("isActive").and_then(Value::as_i64).unwrap_or(1), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
        }
        "routes" => {
            if exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM routes WHERE id=?1)",
                id,
            )? {
                let same: bool = connection
                    .query_row(
                        "SELECT name=?2 FROM routes WHERE id=?1",
                        params![id, text(row, "name")?],
                        |r| r.get(0),
                    )
                    .map_err(|_| missing())?;
                return if same { Ok(()) } else { Err(conflict()) };
            }
            connection.execute("INSERT INTO routes(id,name,normalized_name,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6)", params![id, text(row,"name")?, text(row,"normalizedName")?, row.get("isActive").and_then(Value::as_i64).unwrap_or(1), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
        }
        "active_ingredients" => {
            if exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM active_ingredients WHERE id=?1)",
                id,
            )? {
                let same: bool = connection
                    .query_row(
                        "SELECT name=?2 FROM active_ingredients WHERE id=?1",
                        params![id, text(row, "name")?],
                        |r| r.get(0),
                    )
                    .map_err(|_| missing())?;
                return if same { Ok(()) } else { Err(conflict()) };
            }
            connection.execute("INSERT INTO active_ingredients(id,name,normalized_name,description,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![id, text(row,"name")?, text(row,"normalizedName")?, row.get("description").and_then(Value::as_str), row.get("isActive").and_then(Value::as_i64).unwrap_or(1), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

fn require_package(connection: &Connection, id: &str) -> Result<(), Failure> {
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM product_packages WHERE id=?1)",
        id,
    )? {
        Ok(())
    } else {
        Err(missing())
    }
}

fn upsert_batch(connection: &Connection, row: &Value) -> Result<(), Failure> {
    let id = text(row, "id")?;
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM inventory_batches WHERE id=?1)",
        id,
    )? {
        return Ok(());
    }
    require_package(connection, text(row, "productPackageId")?)?;
    connection.execute("INSERT INTO inventory_batches(id,branch_id,product_package_id,lot_number,expiry_date,received_at,cost_price_minor,supplier_reference,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)", params![id, text(row,"branchId")?, text(row,"productPackageId")?, row.get("lotNumber").and_then(Value::as_str), row.get("expiryDate").and_then(Value::as_str), row.get("receivedAt").and_then(Value::as_str), row.get("costPriceMinor").and_then(Value::as_i64), row.get("supplierReference").and_then(Value::as_str), row.get("isActive").and_then(Value::as_i64).unwrap_or(1), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
    Ok(())
}

fn insert_movement(connection: &Connection, row: &Value) -> Result<(), Failure> {
    let id = text(row, "id")?;
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM inventory_movements WHERE id=?1)",
        id,
    )? {
        return Ok(());
    }
    connection.execute("INSERT INTO inventory_movements(id,branch_id,product_package_id,batch_id,movement_type,quantity_delta,reference_type,reference_id,reason,created_by,transfer_id,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)", params![id, text(row,"branchId")?, text(row,"productPackageId")?, text(row,"batchId")?, text(row,"movementType")?, row.get("quantityDelta").and_then(Value::as_i64).ok_or_else(invalid)?, row.get("referenceType").and_then(Value::as_str), row.get("referenceId").and_then(Value::as_str), text(row,"reason")?, row.get("createdBy").and_then(Value::as_str), row.get("transferId").and_then(Value::as_str), text(row,"createdAt")?]).map_err(|_| Failure { reason: "REVISION_CONFLICT", detail: "Applying the remote transaction would violate the canonical stock balance." })?;
    Ok(())
}

fn apply_catalog_product(
    connection: &Connection,
    entity: &str,
    payload: &Value,
) -> Result<(), Failure> {
    let product = payload.get("product").ok_or_else(invalid)?;
    if text(product, "id")? != entity {
        return Err(invalid());
    }
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM products WHERE id=?1)",
        entity,
    )? {
        let same: bool = connection.query_row(
            "SELECT coalesce(commercial_name_en,'')=coalesce(?2,'') AND coalesce(commercial_name_ar,'')=coalesce(?3,'') AND is_active=?4 FROM products WHERE id=?1",
            params![entity, product.get("commercialNameEn").and_then(Value::as_str), product.get("commercialNameAr").and_then(Value::as_str), product.get("isActive").and_then(Value::as_i64).unwrap_or(1)],
            |row| row.get(0),
        ).map_err(|_| missing())?;
        return if same { Ok(()) } else { Err(conflict()) };
    }
    connection.execute("INSERT INTO products(id,commercial_name_en,commercial_name_ar,normalized_name_en,normalized_name_ar,scientific_name,normalized_scientific_name,manufacturer_id,category_id,route_id,description,notes,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)", params![entity, product.get("commercialNameEn").and_then(Value::as_str), product.get("commercialNameAr").and_then(Value::as_str), product.get("normalizedNameEn").and_then(Value::as_str), product.get("normalizedNameAr").and_then(Value::as_str), product.get("scientificName").and_then(Value::as_str), product.get("normalizedScientificName").and_then(Value::as_str), product.get("manufacturerId").and_then(Value::as_str), product.get("categoryId").and_then(Value::as_str), product.get("routeId").and_then(Value::as_str), product.get("description").and_then(Value::as_str), product.get("notes").and_then(Value::as_str), product.get("isActive").and_then(Value::as_i64).unwrap_or(1), text(product,"createdAt")?, text(product,"updatedAt")?]).map_err(|_| missing())?;
    for row in payload
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        let id = text(row, "id")?;
        if !exists(
            connection,
            "SELECT EXISTS(SELECT 1 FROM product_packages WHERE id=?1)",
            id,
        )? {
            connection.execute("INSERT INTO product_packages(id,product_id,package_label,pack_size,unit_name,units_per_package,strength_text,is_default,is_active,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)", params![id, entity, text(row,"packageLabel")?, row.get("packSize").and_then(Value::as_str), row.get("unitName").and_then(Value::as_str), row.get("unitsPerPackage").and_then(Value::as_i64), row.get("strengthText").and_then(Value::as_str), row.get("isDefault").and_then(Value::as_i64).unwrap_or(0), row.get("isActive").and_then(Value::as_i64).unwrap_or(1), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
        }
    }
    for row in payload
        .get("barcodes")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        let id = text(row, "id")?;
        if !exists(
            connection,
            "SELECT EXISTS(SELECT 1 FROM barcodes WHERE id=?1)",
            id,
        )? {
            connection.execute("INSERT INTO barcodes(id,product_package_id,barcode,is_primary,created_at) VALUES(?1,?2,?3,?4,?5)", params![id, text(row,"productPackageId")?, text(row,"barcode")?, row.get("isPrimary").and_then(Value::as_i64).unwrap_or(0), text(row,"createdAt")?]).map_err(|_| missing())?;
        }
    }
    for row in payload
        .get("prices")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        let id = text(row, "id")?;
        if !exists(
            connection,
            "SELECT EXISTS(SELECT 1 FROM product_price_history WHERE id=?1)",
            id,
        )? {
            connection.execute("INSERT INTO product_price_history(id,product_package_id,selling_price_minor,cost_price_minor,effective_from,effective_to,reason,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![id, text(row,"productPackageId")?, row.get("sellingPriceMinor").and_then(Value::as_i64).unwrap_or(0), row.get("costPriceMinor").and_then(Value::as_i64), text(row,"effectiveFrom")?, row.get("effectiveTo").and_then(Value::as_str), row.get("reason").and_then(Value::as_str), text(row,"createdAt")?]).map_err(|_| missing())?;
        }
    }
    for row in payload
        .get("ingredients")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        let id = text(row, "id")?;
        if !exists(
            connection,
            "SELECT EXISTS(SELECT 1 FROM product_active_ingredients WHERE id=?1)",
            id,
        )? {
            connection.execute("INSERT INTO product_active_ingredients(id,product_id,active_ingredient_id,strength_text,position,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![id, entity, text(row,"activeIngredientId")?, row.get("strengthText").and_then(Value::as_str), row.get("position").and_then(Value::as_i64).unwrap_or(0), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
        }
    }
    Ok(())
}

fn apply_sale(connection: &Connection, entity: &str, payload: &Value) -> Result<(), Failure> {
    let sale = payload.get("sale").ok_or_else(invalid)?;
    let target_status = text(sale, "status")?;
    let target_revision = sale
        .get("revision")
        .and_then(Value::as_i64)
        .ok_or_else(invalid)?;
    if text(sale, "id")? != entity || target_status != "POSTED" || target_revision < 2 {
        return Err(invalid());
    }
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM sales WHERE id=?1)",
        entity,
    )? {
        let same: bool = connection
            .query_row(
                "SELECT sale_number=?2 AND status=?3 FROM sales WHERE id=?1",
                params![entity, text(sale, "saleNumber")?, text(sale, "status")?],
                |row| row.get(0),
            )
            .map_err(|_| missing())?;
        return if same { Ok(()) } else { Err(immutable()) };
    }
    for row in payload
        .get("batches")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        upsert_batch(connection, row)?;
    }
    for row in payload
        .get("movements")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        insert_movement(connection, row)?;
    }
    connection.execute("INSERT INTO sales(id,sale_number,branch_id,customer_id,customer_name,customer_code_snapshot,customer_phone_snapshot,status,revision,draft_json,subtotal_minor,discount_minor,tax_minor,total_minor,payment_status,created_by,notes,post_request_id,created_at,posted_at,voided_at) VALUES(?1,?2,?3,?4,?5,?6,?7,'DRAFT',?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,NULL,NULL)", params![entity, text(sale,"saleNumber")?, text(sale,"branchId")?, sale.get("customerId").and_then(Value::as_str), sale.get("customerName").and_then(Value::as_str), sale.get("customerCodeSnapshot").and_then(Value::as_str), sale.get("customerPhoneSnapshot").and_then(Value::as_str), target_revision - 1, text(sale,"draftJson")?, sale.get("subtotalMinor").and_then(Value::as_i64).unwrap_or(0), sale.get("discountMinor").and_then(Value::as_i64).unwrap_or(0), sale.get("taxMinor").and_then(Value::as_i64).unwrap_or(0), sale.get("totalMinor").and_then(Value::as_i64).unwrap_or(0), text(sale,"paymentStatus")?, sale.get("createdBy").and_then(Value::as_str), sale.get("notes").and_then(Value::as_str), sale.get("postRequestId").and_then(Value::as_str), text(sale,"createdAt")?]).map_err(|_| missing())?;
    for row in payload
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        connection.execute("INSERT INTO sale_items(id,sale_id,position,product_id,product_package_id,product_name_snapshot,package_label_snapshot,quantity,unit_price_minor,discount_minor,sale_discount_minor,tax_minor,line_total_minor,unit_cost_minor,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)", params![text(row,"id")?, entity, row.get("position").and_then(Value::as_i64).unwrap_or(0), text(row,"productId")?, text(row,"productPackageId")?, text(row,"productNameSnapshot")?, text(row,"packageLabelSnapshot")?, row.get("quantity").and_then(Value::as_i64).unwrap_or(0), row.get("unitPriceMinor").and_then(Value::as_i64).unwrap_or(0), row.get("discountMinor").and_then(Value::as_i64).unwrap_or(0), row.get("saleDiscountMinor").and_then(Value::as_i64).unwrap_or(0), row.get("taxMinor").and_then(Value::as_i64).unwrap_or(0), row.get("lineTotalMinor").and_then(Value::as_i64).unwrap_or(0), row.get("unitCostMinor").and_then(Value::as_i64), text(row,"createdAt")?]).map_err(|_| missing())?;
    }
    for row in payload
        .get("allocations")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        connection.execute("INSERT INTO sale_item_allocations(id,sale_item_id,position,batch_id,quantity,unit_cost_minor,movement_id) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![text(row,"id")?, text(row,"saleItemId")?, row.get("position").and_then(Value::as_i64).unwrap_or(0), text(row,"batchId")?, row.get("quantity").and_then(Value::as_i64).unwrap_or(0), row.get("unitCostMinor").and_then(Value::as_i64), text(row,"movementId")?]).map_err(|_| missing())?;
    }
    for row in payload
        .get("payments")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        connection.execute("INSERT INTO sale_payments(id,sale_id,payment_method,amount_minor,reference,created_at) VALUES(?1,?2,?3,?4,?5,?6)", params![text(row,"id")?, entity, text(row,"paymentMethod")?, row.get("amountMinor").and_then(Value::as_i64).unwrap_or(0), row.get("reference").and_then(Value::as_str), text(row,"createdAt")?]).map_err(|_| missing())?;
    }
    connection
        .execute(
            "UPDATE sales SET status='POSTED',revision=revision+1,posted_at=?2 WHERE id=?1",
            params![entity, text(sale, "postedAt")?],
        )
        .map_err(|_| conflict())?;
    Ok(())
}

fn apply_purchase_receipt(
    connection: &Connection,
    entity: &str,
    payload: &Value,
) -> Result<(), Failure> {
    let receipt = payload.get("receipt").ok_or_else(invalid)?;
    let order = payload.get("order").ok_or_else(invalid)?;
    if text(receipt, "id")? != entity || text(receipt, "purchaseOrderId")? != text(order, "id")? {
        return Err(invalid());
    }
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM purchase_receipts WHERE id=?1)",
        entity,
    )? {
        let same: bool = connection
            .query_row(
                "SELECT receipt_number=?2 AND status=?3 FROM purchase_receipts WHERE id=?1",
                params![
                    entity,
                    text(receipt, "receiptNumber")?,
                    text(receipt, "status")?
                ],
                |row| row.get(0),
            )
            .map_err(|_| missing())?;
        return if same { Ok(()) } else { Err(immutable()) };
    }

    let order_id = text(order, "id")?;
    let target_revision = order
        .get("revision")
        .and_then(Value::as_i64)
        .ok_or_else(invalid)?;
    let target_status = text(order, "status")?;
    if !exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM purchase_orders WHERE id=?1)",
        order_id,
    )? {
        if target_revision < 3 || !matches!(target_status, "PARTIALLY_RECEIVED" | "RECEIVED") {
            return Err(invalid());
        }
        let initial_revision = target_revision - 2;
        connection.execute("INSERT INTO purchase_orders(id,purchase_number,supplier_id,supplier_code_snapshot,supplier_name_snapshot,status,revision,supplier_reference,expected_date,notes,subtotal_minor,discount_minor,tax_minor,total_minor,create_request_id,created_at,updated_at,ordered_at) VALUES(?1,?2,?3,?4,?5,'DRAFT',?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)", params![order_id,text(order,"purchaseNumber")?,text(order,"supplierId")?,order.get("supplierCodeSnapshot").and_then(Value::as_str),order.get("supplierNameSnapshot").and_then(Value::as_str),initial_revision,order.get("supplierReference").and_then(Value::as_str),order.get("expectedDate").and_then(Value::as_str),order.get("notes").and_then(Value::as_str),order.get("subtotalMinor").and_then(Value::as_i64).ok_or_else(invalid)?,order.get("discountMinor").and_then(Value::as_i64).unwrap_or(0),order.get("taxMinor").and_then(Value::as_i64).unwrap_or(0),order.get("totalMinor").and_then(Value::as_i64).ok_or_else(invalid)?,text(order,"createRequestId")?,text(order,"createdAt")?,text(order,"updatedAt")?,order.get("orderedAt").and_then(Value::as_str)]).map_err(|_| missing())?;
        for row in payload
            .get("orderLines")
            .and_then(Value::as_array)
            .ok_or_else(invalid)?
        {
            connection.execute("INSERT INTO purchase_order_lines(id,purchase_order_id,position,product_id,product_package_id,product_name_snapshot,package_label_snapshot,ordered_quantity,unit_cost_minor,line_total_minor,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)", params![text(row,"id")?,order_id,row.get("position").and_then(Value::as_i64).unwrap_or(0),text(row,"productId")?,text(row,"productPackageId")?,text(row,"productNameSnapshot")?,text(row,"packageLabelSnapshot")?,row.get("orderedQuantity").and_then(Value::as_i64).ok_or_else(invalid)?,row.get("unitCostMinor").and_then(Value::as_i64).ok_or_else(invalid)?,row.get("lineTotalMinor").and_then(Value::as_i64).ok_or_else(invalid)?,text(row,"createdAt")?,text(row,"updatedAt")?]).map_err(|_| missing())?;
        }
        connection
            .execute(
                "UPDATE purchase_orders SET status='ORDERED',revision=revision+1 WHERE id=?1",
                [order_id],
            )
            .map_err(|_| missing())?;
    } else {
        let current_revision: i64 = connection
            .query_row(
                "SELECT revision FROM purchase_orders WHERE id=?1",
                [order_id],
                |row| row.get(0),
            )
            .map_err(|_| missing())?;
        if current_revision + 1 != target_revision {
            return Err(immutable());
        }
    }

    connection.execute("INSERT INTO purchase_receipts(id,receipt_number,purchase_order_id,branch_id,request_id,request_payload,status,total_quantity,created_by,created_at) VALUES(?1,?2,?3,?4,?5,?6,'POSTING',?7,?8,?9)", params![entity,text(receipt,"receiptNumber")?,order_id,text(receipt,"branchId")?,text(receipt,"requestId")?,text(receipt,"requestPayload")?,receipt.get("totalQuantity").and_then(Value::as_i64).ok_or_else(invalid)?,receipt.get("createdBy").and_then(Value::as_str),text(receipt,"createdAt")?]).map_err(|_| missing())?;
    for row in payload
        .get("batches")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        upsert_batch(connection, row)?;
    }
    for row in payload
        .get("movements")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        insert_movement(connection, row)?;
    }
    for row in payload
        .get("lines")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        connection.execute("INSERT INTO purchase_receipt_lines(id,receipt_id,purchase_order_id,branch_id,purchase_order_line_id,position,product_package_id,product_name_snapshot,package_label_snapshot,quantity,lot_number,expiry_date,acquisition_cost_minor,batch_id,movement_id,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)", params![text(row,"id")?,entity,order_id,text(row,"branchId")?,text(row,"purchaseOrderLineId")?,row.get("position").and_then(Value::as_i64).unwrap_or(0),text(row,"productPackageId")?,text(row,"productNameSnapshot")?,text(row,"packageLabelSnapshot")?,row.get("quantity").and_then(Value::as_i64).ok_or_else(invalid)?,row.get("lotNumber").and_then(Value::as_str),row.get("expiryDate").and_then(Value::as_str),row.get("acquisitionCostMinor").and_then(Value::as_i64).ok_or_else(invalid)?,text(row,"batchId")?,text(row,"movementId")?,text(row,"createdAt")?]).map_err(|_| missing())?;
    }
    connection
        .execute(
            "UPDATE purchase_receipts SET status='POSTED',posted_at=?2 WHERE id=?1",
            params![entity, text(receipt, "postedAt")?],
        )
        .map_err(|_| missing())?;
    connection
        .execute(
            "UPDATE purchase_orders SET status=?2,revision=revision+1,updated_at=?3 WHERE id=?1",
            params![order_id, target_status, text(order, "updatedAt")?],
        )
        .map_err(|_| missing())?;
    Ok(())
}

fn apply_purchase_return(
    connection: &Connection,
    entity: &str,
    payload: &Value,
) -> Result<(), Failure> {
    let header = payload.get("header").ok_or_else(invalid)?;
    if text(header, "id")? != entity {
        return Err(invalid());
    }
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM purchase_returns WHERE id=?1)",
        entity,
    )? {
        let same: bool = connection
            .query_row(
                "SELECT return_number=?2 AND status=?3 FROM purchase_returns WHERE id=?1",
                params![
                    entity,
                    text(header, "returnNumber")?,
                    text(header, "status")?
                ],
                |row| row.get(0),
            )
            .map_err(|_| missing())?;
        return if same { Ok(()) } else { Err(immutable()) };
    }
    connection.execute("INSERT INTO purchase_returns(id,return_number,purchase_order_id,receipt_id,branch_id,supplier_id,request_id,request_payload,source_purchase_revision,status,reason,total_quantity,created_by,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,'POSTING',?10,?11,?12,?13)", params![entity,text(header,"returnNumber")?,text(header,"purchaseOrderId")?,text(header,"receiptId")?,text(header,"branchId")?,text(header,"supplierId")?,text(header,"requestId")?,text(header,"requestPayload")?,header.get("sourcePurchaseRevision").and_then(Value::as_i64).ok_or_else(invalid)?,text(header,"reason")?,header.get("totalQuantity").and_then(Value::as_i64).ok_or_else(invalid)?,header.get("createdBy").and_then(Value::as_str),text(header,"createdAt")?]).map_err(|_| missing())?;
    for row in payload
        .get("movements")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        insert_movement(connection, row)?;
    }
    for row in payload
        .get("lines")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        connection.execute("INSERT INTO purchase_return_lines(id,return_id,purchase_order_id,receipt_id,branch_id,receipt_line_id,purchase_order_line_id,position,product_package_id,product_name_snapshot,package_label_snapshot,previously_returned_quantity,quantity,lot_number,expiry_date,acquisition_cost_minor,batch_id,movement_id,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)", params![text(row,"id")?,entity,text(row,"purchaseOrderId")?,text(row,"receiptId")?,text(row,"branchId")?,text(row,"receiptLineId")?,text(row,"purchaseOrderLineId")?,row.get("position").and_then(Value::as_i64).unwrap_or(0),text(row,"productPackageId")?,text(row,"productNameSnapshot")?,text(row,"packageLabelSnapshot")?,row.get("previouslyReturnedQuantity").and_then(Value::as_i64).unwrap_or(0),row.get("quantity").and_then(Value::as_i64).ok_or_else(invalid)?,row.get("lotNumber").and_then(Value::as_str),row.get("expiryDate").and_then(Value::as_str),row.get("acquisitionCostMinor").and_then(Value::as_i64).ok_or_else(invalid)?,text(row,"batchId")?,text(row,"movementId")?,text(row,"createdAt")?]).map_err(|_| missing())?;
    }
    connection
        .execute(
            "UPDATE purchase_returns SET status='POSTED',posted_at=?2 WHERE id=?1",
            params![entity, text(header, "postedAt")?],
        )
        .map_err(|_| missing())?;
    Ok(())
}

fn apply_stock_count(
    connection: &Connection,
    entity: &str,
    payload: &Value,
) -> Result<(), Failure> {
    let session = payload.get("session").ok_or_else(invalid)?;
    let target_revision = session
        .get("revision")
        .and_then(Value::as_i64)
        .ok_or_else(invalid)?;
    if text(session, "id")? != entity
        || text(session, "status")? != "COMPLETED"
        || target_revision < 4
    {
        return Err(invalid());
    }
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM stock_count_sessions WHERE id=?1)",
        entity,
    )? {
        let same: bool = connection
            .query_row(
                "SELECT session_number=?2 AND status='COMPLETED' FROM stock_count_sessions WHERE id=?1",
                params![entity, text(session, "sessionNumber")?],
                |row| row.get(0),
            )
            .map_err(|_| missing())?;
        return if same { Ok(()) } else { Err(immutable()) };
    }

    let branch = text(session, "branchId")?;
    let watermark: i64 = connection
        .query_row(
            "SELECT coalesce(max(rowid),0) FROM inventory_movements",
            [],
            |row| row.get(0),
        )
        .map_err(|_| missing())?;
    connection.execute("INSERT INTO stock_count_sessions(id,session_number,branch_id,status,scope_type,notes,created_by,revision,created_at) VALUES(?1,?2,?3,'DRAFT',?4,?5,?6,1,?7)", params![entity,text(session,"sessionNumber")?,branch,text(session,"scopeType")?,session.get("notes").and_then(Value::as_str),session.get("createdBy").and_then(Value::as_str),text(session,"createdAt")?]).map_err(|_| missing())?;
    connection.execute("UPDATE stock_count_sessions SET status='IN_PROGRESS',revision=revision+1,snapshot_watermark=?2,started_at=?3 WHERE id=?1", params![entity,watermark,text(session,"startedAt")?]).map_err(|_| missing())?;

    for row in payload
        .get("lines")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        if text(row, "sessionId")? != entity || text(row, "branchId")? != branch {
            return Err(invalid());
        }
        let batch = text(row, "batchId")?;
        let expected = row
            .get("expectedQuantity")
            .and_then(Value::as_i64)
            .ok_or_else(invalid)?;
        let local: i64 = connection
            .query_row(
                "SELECT coalesce(sum(quantity_delta),0) FROM inventory_movements WHERE batch_id=?1 AND rowid<=?2",
                params![batch, watermark],
                |result| result.get(0),
            )
            .map_err(|_| missing())?;
        if local != expected {
            return Err(Failure {
                reason: "REVISION_CONFLICT",
                detail: "The stock-count snapshot does not match the receiving device's live batch balance; operator resolution is required.",
            });
        }
        connection.execute("INSERT INTO stock_count_lines(id,session_id,branch_id,product_package_id,batch_id,expected_quantity,notes,revision,product_label,package_label) VALUES(?1,?2,?3,?4,?5,?6,?7,1,?8,?9)", params![text(row,"id")?,entity,branch,text(row,"productPackageId")?,batch,expected,row.get("notes").and_then(Value::as_str),text(row,"productLabel")?,text(row,"packageLabel")?]).map_err(|_| missing())?;
        let line_revision = row
            .get("revision")
            .and_then(Value::as_i64)
            .ok_or_else(invalid)?;
        if line_revision < 2 {
            return Err(invalid());
        }
        for _ in 1..line_revision {
            connection.execute("UPDATE stock_count_lines SET counted_quantity=?2,notes=?3,excluded_reason=?4,counted_at=?5,revision=revision+1 WHERE id=?1", params![text(row,"id")?,row.get("countedQuantity").and_then(Value::as_i64),row.get("notes").and_then(Value::as_str),row.get("excludedReason").and_then(Value::as_str),row.get("countedAt").and_then(Value::as_str)]).map_err(|_| missing())?;
            connection
                .execute(
                    "UPDATE stock_count_sessions SET revision=revision+1 WHERE id=?1",
                    [entity],
                )
                .map_err(|_| missing())?;
        }
    }
    let current_revision: i64 = connection
        .query_row(
            "SELECT revision FROM stock_count_sessions WHERE id=?1",
            [entity],
            |row| row.get(0),
        )
        .map_err(|_| missing())?;
    if current_revision > target_revision - 2 {
        return Err(invalid());
    }
    for _ in current_revision..(target_revision - 2) {
        connection
            .execute(
                "UPDATE stock_count_sessions SET revision=revision+1 WHERE id=?1",
                [entity],
            )
            .map_err(|_| missing())?;
    }
    connection.execute("UPDATE stock_count_sessions SET status='REVIEW',revision=revision+1,reviewed_at=?2 WHERE id=?1", params![entity,text(session,"reviewedAt")?]).map_err(|_| missing())?;

    for row in payload
        .get("movements")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        insert_movement(connection, row)?;
    }
    for row in payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        connection.execute("INSERT INTO stock_count_results(line_id,live_baseline,correction_delta,intervening_movements,movement_id) VALUES(?1,?2,?3,?4,?5)", params![text(row,"lineId")?,row.get("liveBaseline").and_then(Value::as_i64).ok_or_else(invalid)?,row.get("correctionDelta").and_then(Value::as_i64).ok_or_else(invalid)?,row.get("interveningMovements").and_then(Value::as_i64).unwrap_or(0),row.get("movementId").and_then(Value::as_str)]).map_err(|_| conflict())?;
    }
    connection.execute("UPDATE stock_count_sessions SET status='COMPLETED',revision=revision+1,completed_at=?2 WHERE id=?1", params![entity,text(session,"completedAt")?]).map_err(|_| conflict())?;
    Ok(())
}

fn apply_inventory_document(
    connection: &Connection,
    entity: &str,
    payload: &Value,
) -> Result<(), Failure> {
    let document = payload.get("document").ok_or_else(invalid)?;
    let target_revision = document
        .get("revision")
        .and_then(Value::as_i64)
        .ok_or_else(invalid)?;
    if text(document, "id")? != entity
        || text(document, "status")? != "POSTED"
        || target_revision < 3
    {
        return Err(invalid());
    }
    if exists(
        connection,
        "SELECT EXISTS(SELECT 1 FROM inventory_documents WHERE id=?1)",
        entity,
    )? {
        let same: bool = connection
            .query_row(
                "SELECT document_number=?2 AND status='POSTED' FROM inventory_documents WHERE id=?1",
                params![entity, text(document, "documentNumber")?],
                |row| row.get(0),
            )
            .map_err(|_| missing())?;
        return if same { Ok(()) } else { Err(immutable()) };
    }
    let kind = text(document, "kind")?;
    connection.execute("INSERT INTO inventory_documents(id,document_number,kind,branch_id,destination_branch_id,status,revision,reference_text,supplier_reference,notes,created_by,reversal_of,reversal_reason,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,'DRAFT',1,?6,?7,?8,?9,?10,?11,?12,?13)", params![entity,text(document,"documentNumber")?,kind,text(document,"branchId")?,document.get("destinationBranchId").and_then(Value::as_str),document.get("referenceText").and_then(Value::as_str),document.get("supplierReference").and_then(Value::as_str),document.get("notes").and_then(Value::as_str),document.get("createdBy").and_then(Value::as_str),document.get("reversalOf").and_then(Value::as_str),document.get("reversalReason").and_then(Value::as_str),text(document,"createdAt")?,text(document,"updatedAt")?]).map_err(|_| missing())?;
    for row in payload
        .get("lines")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        if text(row, "documentId")? != entity {
            return Err(invalid());
        }
        connection.execute("INSERT INTO inventory_document_lines(id,document_id,position,product_package_id,batch_id,action,quantity,reason,lot_number,expiry_date,cost_price_minor,supplier_reference,product_label,package_label) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)", params![text(row,"id")?,entity,row.get("position").and_then(Value::as_i64).unwrap_or(0),text(row,"productPackageId")?,row.get("batchId").and_then(Value::as_str),text(row,"action")?,row.get("quantity").and_then(Value::as_i64).ok_or_else(invalid)?,text(row,"reason")?,row.get("lotNumber").and_then(Value::as_str),row.get("expiryDate").and_then(Value::as_str),row.get("costPriceMinor").and_then(Value::as_i64),row.get("supplierReference").and_then(Value::as_str),text(row,"productLabel")?,text(row,"packageLabel")?]).map_err(|_| missing())?;
    }
    for _ in 1..(target_revision - 2) {
        connection
            .execute(
                "UPDATE inventory_documents SET revision=revision+1 WHERE id=?1",
                [entity],
            )
            .map_err(|_| missing())?;
    }
    connection
        .execute(
            "UPDATE inventory_documents SET status='READY',revision=revision+1 WHERE id=?1",
            [entity],
        )
        .map_err(|_| missing())?;
    for row in payload
        .get("batches")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        upsert_batch(connection, row)?;
    }
    if kind == "TRANSFER" {
        for row in payload
            .get("movements")
            .and_then(Value::as_array)
            .ok_or_else(invalid)?
        {
            let transfer = row
                .get("transferId")
                .and_then(Value::as_str)
                .ok_or_else(invalid)?;
            if !exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM inventory_transfers WHERE id=?1)",
                transfer,
            )? {
                return Err(missing());
            }
        }
    } else {
        for row in payload
            .get("movements")
            .and_then(Value::as_array)
            .ok_or_else(invalid)?
        {
            insert_movement(connection, row)?;
        }
    }
    for row in payload
        .get("links")
        .and_then(Value::as_array)
        .ok_or_else(invalid)?
    {
        connection.execute("INSERT INTO inventory_document_movements(movement_id,document_id,line_id,reverses_movement_id) VALUES(?1,?2,?3,?4)", params![text(row,"movementId")?,entity,text(row,"lineId")?,row.get("reversesMovementId").and_then(Value::as_str)]).map_err(|_| missing())?;
    }
    connection.execute("UPDATE inventory_documents SET status='POSTED',revision=revision+1,posted_at=?2 WHERE id=?1", params![entity,text(document,"postedAt")?]).map_err(|_| conflict())?;
    Ok(())
}

pub fn apply(
    connection: &Connection,
    kind: &str,
    entity: &str,
    payload: &Value,
) -> Result<(), Failure> {
    match kind {
        "catalog_lookup" => insert_lookup(
            connection,
            payload
                .get("table")
                .and_then(Value::as_str)
                .ok_or_else(invalid)?,
            payload.get("row").ok_or_else(invalid)?,
        ),
        "catalog_product" => apply_catalog_product(connection, entity, payload),
        "partner_record" => {
            let table = payload
                .get("table")
                .and_then(Value::as_str)
                .ok_or_else(invalid)?;
            let row = payload.get("row").ok_or_else(invalid)?;
            let id = text(row, "id")?;
            if id != entity {
                return Err(invalid());
            }
            match table {
                "suppliers" => {
                    if exists(
                        connection,
                        "SELECT EXISTS(SELECT 1 FROM suppliers WHERE id=?1)",
                        id,
                    )? {
                        return Ok(());
                    }
                    connection.execute("INSERT INTO suppliers(id,code,display_name,contact_person,primary_phone,secondary_phone,email,address,tax_registration_id,notes,is_active,revision,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)", params![id, text(row,"code")?, text(row,"displayName")?, row.get("contactPerson").and_then(Value::as_str), row.get("primaryPhone").and_then(Value::as_str), row.get("secondaryPhone").and_then(Value::as_str), row.get("email").and_then(Value::as_str), row.get("address").and_then(Value::as_str), row.get("taxRegistrationId").and_then(Value::as_str), row.get("notes").and_then(Value::as_str), row.get("isActive").and_then(Value::as_i64).unwrap_or(1), row.get("revision").and_then(Value::as_i64).unwrap_or(1), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
                }
                "customers" => {
                    if exists(
                        connection,
                        "SELECT EXISTS(SELECT 1 FROM customers WHERE id=?1)",
                        id,
                    )? {
                        return Ok(());
                    }
                    connection.execute("INSERT INTO customers(id,code,display_name,primary_phone,secondary_phone,email,address,notes,is_active,revision,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)", params![id, text(row,"code")?, text(row,"displayName")?, row.get("primaryPhone").and_then(Value::as_str), row.get("secondaryPhone").and_then(Value::as_str), row.get("email").and_then(Value::as_str), row.get("address").and_then(Value::as_str), row.get("notes").and_then(Value::as_str), row.get("isActive").and_then(Value::as_i64).unwrap_or(1), row.get("revision").and_then(Value::as_i64).unwrap_or(1), text(row,"createdAt")?, text(row,"updatedAt")?]).map_err(|_| missing())?;
                }
                _ => return Err(invalid()),
            }
            Ok(())
        }
        "sale_document" => apply_sale(connection, entity, payload),
        "sale_return" => {
            let header = payload.get("header").ok_or_else(invalid)?;
            if text(header, "id")? != entity || text(header, "status")? != "POSTED" {
                return Err(invalid());
            }
            if !exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM sales WHERE id=?1)",
                text(header, "saleId")?,
            )? {
                return Err(missing());
            }
            if exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM sale_returns WHERE id=?1)",
                entity,
            )? {
                return Ok(());
            }
            connection.execute("INSERT INTO sale_returns(id,sale_id,status,reason,lines_json,refund_minor,created_at,posted_at) VALUES(?1,?2,'DRAFT',?3,?4,?5,?6,NULL)", params![entity, text(header,"saleId")?, text(header,"reason")?, text(header,"linesJson")?, header.get("refundMinor").and_then(Value::as_i64).unwrap_or(0), text(header,"createdAt")?]).map_err(|_| missing())?;
            for row in payload
                .get("items")
                .and_then(Value::as_array)
                .ok_or_else(invalid)?
            {
                if text(row, "returnId")? != entity {
                    return Err(invalid());
                }
                connection.execute("INSERT INTO sale_return_items(id,return_id,sale_item_id,quantity,refund_minor) VALUES(?1,?2,?3,?4,?5)", params![text(row,"id")?,entity,text(row,"saleItemId")?,row.get("quantity").and_then(Value::as_i64).ok_or_else(invalid)?,row.get("refundMinor").and_then(Value::as_i64).ok_or_else(invalid)?]).map_err(|_| conflict())?;
            }
            for row in payload
                .get("movements")
                .and_then(Value::as_array)
                .ok_or_else(invalid)?
            {
                insert_movement(connection, row)?;
            }
            for row in payload
                .get("allocations")
                .and_then(Value::as_array)
                .ok_or_else(invalid)?
            {
                connection.execute("INSERT INTO sale_return_allocations(id,return_item_id,original_allocation_id,quantity,movement_id) VALUES(?1,?2,?3,?4,?5)", params![text(row,"id")?,text(row,"returnItemId")?,text(row,"originalAllocationId")?,row.get("quantity").and_then(Value::as_i64).ok_or_else(invalid)?,text(row,"movementId")?]).map_err(|_| conflict())?;
            }
            for row in payload
                .get("refunds")
                .and_then(Value::as_array)
                .ok_or_else(invalid)?
            {
                if text(row, "saleId")? != text(header, "saleId")?
                    || row.get("returnId").and_then(Value::as_str) != Some(entity)
                {
                    return Err(invalid());
                }
                connection.execute("INSERT INTO sale_refunds(id,sale_id,return_id,void_id,amount_minor,created_at) VALUES(?1,?2,?3,NULL,?4,?5)", params![text(row,"id")?,text(row,"saleId")?,entity,row.get("amountMinor").and_then(Value::as_i64).ok_or_else(invalid)?,text(row,"createdAt")?]).map_err(|_| conflict())?;
            }
            connection
                .execute(
                    "UPDATE sale_returns SET status='POSTED',posted_at=?2 WHERE id=?1",
                    params![entity, text(header, "postedAt")?],
                )
                .map_err(|_| conflict())?;
            Ok(())
        }
        "purchase_receipt" => apply_purchase_receipt(connection, entity, payload),
        "purchase_return" => apply_purchase_return(connection, entity, payload),
        "inventory_document" => apply_inventory_document(connection, entity, payload),
        "stock_count" => apply_stock_count(connection, entity, payload),
        _ => Err(Failure {
            reason: "UNSUPPORTED_ENTITY",
            detail: "No verified domain handler exists for this remote entity type.",
        }),
    }
}
