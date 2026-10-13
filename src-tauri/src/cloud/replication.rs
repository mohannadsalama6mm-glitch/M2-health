use super::outbox;
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

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
        "receipt": json_query(connection, "SELECT json_object('id',id,'receiptNumber',receipt_number,'purchaseOrderId',purchase_order_id,'branchId',branch_id,'requestId',request_id,'requestPayload',request_payload,'totalQuantity',total_quantity,'status',status,'createdBy',created_by,'postedAt',posted_at) FROM purchase_receipts WHERE id=?1", receipt_id)?,
        "order": json_query(connection, "SELECT json_object('id',o.id,'purchaseNumber',o.purchase_number,'supplierId',o.supplier_id,'status',o.status,'revision',o.revision,'supplierCodeSnapshot',o.supplier_code_snapshot,'supplierNameSnapshot',o.supplier_name_snapshot) FROM purchase_orders o JOIN purchase_receipts r ON r.purchase_order_id=o.id WHERE r.id=?1", receipt_id)?,
        "orderLines": json_rows(connection, "SELECT json_object('id',l.id,'purchaseOrderId',l.purchase_order_id,'position',l.position,'productPackageId',l.product_package_id,'orderedQuantity',l.ordered_quantity) FROM purchase_order_lines l JOIN purchase_receipts r ON r.purchase_order_id=l.purchase_order_id WHERE r.id=?1 ORDER BY l.position", receipt_id)?,
        "lines": json_rows(connection, "SELECT json_object('id',id,'receiptId',receipt_id,'purchaseOrderId',purchase_order_id,'branchId',branch_id,'purchaseOrderLineId',purchase_order_line_id,'position',position,'productPackageId',product_package_id,'productNameSnapshot',product_name_snapshot,'packageLabelSnapshot',package_label_snapshot,'quantity',quantity,'lotNumber',lot_number,'expiryDate',expiry_date,'acquisitionCostMinor',acquisition_cost_minor,'batchId',batch_id,'movementId',movement_id) FROM purchase_receipt_lines WHERE receipt_id=?1 ORDER BY position", receipt_id)?,
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
        "header": json_query(connection, "SELECT json_object('id',id,'returnNumber',return_number,'purchaseOrderId',purchase_order_id,'receiptId',receipt_id,'branchId',branch_id,'supplierId',supplier_id,'totalQuantity',total_quantity,'reason',reason,'status',status,'createdBy',created_by,'postedAt',posted_at) FROM purchase_returns WHERE id=?1", return_id)?,
        "lines": json_rows(connection, "SELECT json_object('id',id,'returnId',return_id,'receiptLineId',receipt_line_id,'quantity',quantity,'batchId',batch_id,'movementId',movement_id) FROM purchase_return_lines WHERE return_id=?1", return_id)?,
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
        "session": json_query(connection, "SELECT json_object('id',id,'sessionNumber',session_number,'branchId',branch_id,'status',status,'revision',revision,'createdBy',created_by,'completedAt',completed_at) FROM stock_count_sessions WHERE id=?1", session_id)?,
        "lines": json_rows(connection, "SELECT json_object('id',id,'sessionId',session_id,'productPackageId',product_package_id,'batchId',batch_id,'expectedQuantity',expected_quantity,'countedQuantity',counted_quantity,'notes',notes,'excludedReason',excluded_reason) FROM stock_count_lines WHERE session_id=?1", session_id)?,
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
        "document": json_query(connection, "SELECT json_object('id',id,'documentNumber',document_number,'branchId',branch_id,'status',status,'createdBy',created_by) FROM inventory_documents WHERE id=?1", document_id)?,
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
    if text(sale, "id")? != entity {
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
    connection.execute("INSERT INTO sales(id,sale_number,branch_id,customer_id,customer_name,customer_code_snapshot,customer_phone_snapshot,status,revision,draft_json,subtotal_minor,discount_minor,tax_minor,total_minor,payment_status,created_by,notes,post_request_id,created_at,posted_at,voided_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)", params![entity, text(sale,"saleNumber")?, text(sale,"branchId")?, sale.get("customerId").and_then(Value::as_str), sale.get("customerName").and_then(Value::as_str), sale.get("customerCodeSnapshot").and_then(Value::as_str), sale.get("customerPhoneSnapshot").and_then(Value::as_str), text(sale,"status")?, sale.get("revision").and_then(Value::as_i64).unwrap_or(1), text(sale,"draftJson")?, sale.get("subtotalMinor").and_then(Value::as_i64).unwrap_or(0), sale.get("discountMinor").and_then(Value::as_i64).unwrap_or(0), sale.get("taxMinor").and_then(Value::as_i64).unwrap_or(0), sale.get("totalMinor").and_then(Value::as_i64).unwrap_or(0), text(sale,"paymentStatus")?, sale.get("createdBy").and_then(Value::as_str), sale.get("notes").and_then(Value::as_str), sale.get("postRequestId").and_then(Value::as_str), text(sale,"createdAt")?, sale.get("postedAt").and_then(Value::as_str), sale.get("voidedAt").and_then(Value::as_str)]).map_err(|_| missing())?;
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
    Ok(())
}

fn apply_document_owned(
    connection: &Connection,
    payload: &Value,
    extra: impl FnOnce(&Connection) -> Result<(), Failure>,
) -> Result<(), Failure> {
    for row in payload
        .get("batches")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        upsert_batch(connection, row)?;
    }
    extra(connection)?;
    for row in payload
        .get("movements")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
    {
        insert_movement(connection, row)?;
    }
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
            if text(header, "id")? != entity {
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
            apply_document_owned(connection, payload, |connection| {
                connection.execute("INSERT INTO sale_returns(id,sale_id,status,reason,lines_json,refund_minor,created_at,posted_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![entity, text(header,"saleId")?, text(header,"status")?, text(header,"reason")?, text(header,"linesJson")?, header.get("refundMinor").and_then(Value::as_i64).unwrap_or(0), text(header,"createdAt")?, header.get("postedAt").and_then(Value::as_str)]).map_err(|_| missing())?;
                Ok(())
            })
        }
        "purchase_receipt" => {
            if exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM purchase_receipts WHERE id=?1)",
                entity,
            )? {
                return Ok(());
            }
            apply_document_owned(connection, payload, |_| Ok(()))
        }
        "purchase_return" => {
            if exists(
                connection,
                "SELECT EXISTS(SELECT 1 FROM purchase_returns WHERE id=?1)",
                entity,
            )? {
                return Ok(());
            }
            apply_document_owned(connection, payload, |_| Ok(()))
        }
        "inventory_document" => apply_document_owned(connection, payload, |_| Ok(())),
        // Count corrections are guarded by a local REVIEW-state session trigger and are
        // computed against the local live balance; replaying them remotely is not a
        // verified operation, so the whole count is quarantined for operator review.
        "stock_count" => Err(Failure {
            reason: "UNSUPPORTED_ENTITY",
            detail: "Remote stock-count completion requires operator review; count corrections are not replayed automatically.",
        }),
        _ => Err(Failure {
            reason: "UNSUPPORTED_ENTITY",
            detail: "No verified domain handler exists for this remote entity type.",
        }),
    }
}
