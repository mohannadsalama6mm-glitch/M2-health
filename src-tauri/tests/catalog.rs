use m2_health_lib::{
    db::{
        catalog_validation::*,
        connection::AppDb,
        migrations::{self, MIGRATIONS},
        models::catalog::*,
        repositories::*,
    },
    error::ErrorCode,
};
use rusqlite::{params, Connection};
fn product(db: &AppDb) -> Product {
    products::create(
        db,
        &CreateProduct {
            commercial_name_en: Some(" Test Medicine ".into()),
            commercial_name_ar: Some("دواء تجريبي".into()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn package(db: &AppDb, p: &str) -> ProductPackage {
    product_packages::create(
        db,
        &CreateProductPackage {
            product_id: p.into(),
            package_label: "20 tablets".into(),
            ..Default::default()
        },
    )
    .unwrap()
}
fn price(db: &AppDb, p: &str, amount: i64) -> PackagePrice {
    price_history::set(
        db,
        &SetPackagePrice {
            product_package_id: p.into(),
            selling_price_minor: amount,
            ..Default::default()
        },
    )
    .unwrap()
}
fn scalar(c: &Connection, sql: &str) -> i64 {
    c.query_row(sql, [], |r| r.get(0)).unwrap()
}
fn package_input(label: &str) -> ProductPackageInput {
    ProductPackageInput {
        package_label: label.into(),
        is_active: true,
        ..Default::default()
    }
}

#[test]
fn upgrade_v1_to_v2_preserves_branch_and_reruns() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("upgrade.db");
    {
        let mut c = Connection::open(&path).unwrap();
        migrations::apply(&mut c, &MIGRATIONS[..1]).unwrap();
        c.execute(
            "INSERT INTO branches(id,code,name) VALUES (?1,'EXISTING','Existing Pharmacy')",
            [uuid::Uuid::new_v4().to_string()],
        )
        .unwrap();
    }
    let db = AppDb::open(&path).unwrap();
    let branch = branches::ensure_default_branch(&db).unwrap();
    assert_eq!(branch.code, "EXISTING");
    assert!(products::list(&db, &ProductQuery::default())
        .unwrap()
        .items
        .is_empty());
    let mut c = db.lock().unwrap();
    migrations::run(&mut c).unwrap();
    assert_eq!(scalar(&c, "SELECT count(*) FROM _schema_version"), 4);
    assert_eq!(scalar(&c, "SELECT count(*) FROM branches"), 1);
    for table in [
        "manufacturers",
        "categories",
        "routes",
        "active_ingredients",
        "products",
        "product_active_ingredients",
        "product_packages",
        "barcodes",
        "product_price_history",
    ] {
        assert_eq!(scalar(&c, &format!("SELECT count(*) FROM {table}")), 0);
    }
}
#[test]
fn actual_migration_002_rolls_back_on_conflicting_schema() {
    let mut c = Connection::open_in_memory().unwrap();
    migrations::apply(&mut c, &MIGRATIONS[..1]).unwrap();
    c.execute_batch("INSERT INTO branches(id,code,name) VALUES('preserved','MAIN','Keep'); CREATE TABLE categories(sentinel TEXT); INSERT INTO categories VALUES('keep');").unwrap();
    assert!(migrations::run(&mut c).is_err());
    assert_eq!(scalar(&c, "SELECT count(*) FROM _schema_version"), 1);
    assert_eq!(
        scalar(
            &c,
            "SELECT count(*) FROM sqlite_master WHERE name='manufacturers'"
        ),
        0
    );
    assert_eq!(scalar(&c, "SELECT count(*) FROM branches"), 1);
    assert_eq!(scalar(&c, "SELECT count(*) FROM categories"), 1);
}
#[test]
fn lookups_preserve_display_and_reject_case_space_duplicates() {
    let db = AppDb::in_memory().unwrap();
    let m = manufacturers::create(
        &db,
        &CreateManufacturer {
            name: "  Apex   PHARMA ".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(m.name, "  Apex   PHARMA ");
    assert_eq!(m.normalized_name, "apex pharma");
    assert_eq!(
        manufacturers::create(
            &db,
            &CreateManufacturer {
                name: "apex pharma".into(),
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
    let cat = categories::create(
        &db,
        &CreateCategory {
            name: "Drug Class".into(),
            description: None,
        },
    )
    .unwrap();
    assert!(categories::create(
        &db,
        &CreateCategory {
            name: " drug  CLASS ".into(),
            description: None
        }
    )
    .is_err());
    let route = routes::create(
        &db,
        &CreateRoute {
            name: "UNKNOWN".into(),
        },
    )
    .unwrap();
    assert!(routes::create(
        &db,
        &CreateRoute {
            name: "unknown".into()
        }
    )
    .is_err());
    let ingredient = active_ingredients::create(
        &db,
        &CreateActiveIngredient {
            name: "مادة فعالة".into(),
            description: None,
        },
    )
    .unwrap();
    assert!(active_ingredients::create(
        &db,
        &CreateActiveIngredient {
            name: " مادة  فعالة ".into(),
            description: None
        }
    )
    .is_err());
    assert_eq!(manufacturers::list(&db, false).unwrap().len(), 1);
    assert_eq!(categories::list(&db, false).unwrap()[0].id, cat.id);
    assert_eq!(routes::list(&db, false).unwrap()[0].id, route.id);
    assert_eq!(
        active_ingredients::list(&db, false).unwrap()[0].id,
        ingredient.id
    );
}
#[test]
fn lookup_deactivation_retains_records_and_timestamps() {
    let db = AppDb::in_memory().unwrap();
    let m = manufacturers::create(
        &db,
        &CreateManufacturer {
            name: "Maker".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let inactive = manufacturers::set_active(&db, &m.id, false).unwrap();
    assert!(inactive.updated_at > m.updated_at);
    assert!(manufacturers::list(&db, false).unwrap().is_empty());
    assert_eq!(manufacturers::list(&db, true).unwrap().len(), 1);
    let c = categories::create(
        &db,
        &CreateCategory {
            name: "Class".into(),
            ..Default::default()
        },
    )
    .unwrap();
    categories::set_active(&db, &c.id, false).unwrap();
    assert!(categories::list(&db, false).unwrap().is_empty());
    let r = routes::create(
        &db,
        &CreateRoute {
            name: "ROUTE".into(),
        },
    )
    .unwrap();
    routes::set_active(&db, &r.id, false).unwrap();
    assert!(routes::list(&db, false).unwrap().is_empty());
    let i = active_ingredients::create(
        &db,
        &CreateActiveIngredient {
            name: "Ingredient".into(),
            ..Default::default()
        },
    )
    .unwrap();
    active_ingredients::set_active(&db, &i.id, false).unwrap();
    assert!(active_ingredients::list(&db, false).unwrap().is_empty());
}
#[test]
fn bilingual_nullable_identity_and_foreign_keys() {
    let db = AppDb::in_memory().unwrap();
    let p = product(&db);
    assert_eq!(p.commercial_name_en.as_deref(), Some(" Test Medicine "));
    assert_eq!(p.commercial_name_ar.as_deref(), Some("دواء تجريبي"));
    assert_eq!(p.normalized_name_en.as_deref(), Some("test medicine"));
    assert!(p.scientific_name.is_none() && p.manufacturer_id.is_none());
    assert!(products::create(&db, &CreateProduct::default()).is_err());
    assert!(products::create(
        &db,
        &CreateProduct {
            commercial_name_ar: Some("   ".into()),
            ..Default::default()
        }
    )
    .is_err());
    assert!(products::create(
        &db,
        &CreateProduct {
            commercial_name_ar: Some("عربي فقط".into()),
            ..Default::default()
        }
    )
    .is_ok());
    for field in ["manufacturer_id", "category_id", "route_id"] {
        let mut input = CreateProduct {
            commercial_name_en: Some("Bad relationship".into()),
            ..Default::default()
        };
        let missing = Some(uuid::Uuid::new_v4().to_string());
        match field {
            "manufacturer_id" => input.manufacturer_id = missing,
            "category_id" => input.category_id = missing,
            _ => input.route_id = missing,
        };
        assert_eq!(
            products::create(&db, &input).unwrap_err().code,
            ErrorCode::Validation
        );
    }
}
#[test]
fn detail_aggregates_explicit_relations_and_multiple_ingredients() {
    let db = AppDb::in_memory().unwrap();
    let m = manufacturers::create(
        &db,
        &CreateManufacturer {
            name: "Maker".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let c = categories::create(
        &db,
        &CreateCategory {
            name: "Class".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let r = routes::create(
        &db,
        &CreateRoute {
            name: "ORAL.SOLID".into(),
        },
    )
    .unwrap();
    let p = products::create(
        &db,
        &CreateProduct {
            commercial_name_en: Some("Combination".into()),
            scientific_name: Some("Original A+B text".into()),
            manufacturer_id: Some(m.id.clone()),
            category_id: Some(c.id.clone()),
            route_id: Some(r.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    for (index, name) in ["Ingredient A", "Ingredient B"].iter().enumerate() {
        let i = active_ingredients::create(
            &db,
            &CreateActiveIngredient {
                name: (*name).into(),
                ..Default::default()
            },
        )
        .unwrap();
        let link = LinkProductIngredient {
            product_id: p.id.clone(),
            active_ingredient_id: i.id,
            position: index as i64,
            strength_text: Some("Unparsed strength".into()),
        };
        product_ingredients::link(&db, &link).unwrap();
        assert!(product_ingredients::link(&db, &link).is_err());
    }
    let pk = package(&db, &p.id);
    let other = package(&db, &p.id);
    let price = price(&db, &pk.id, 2250);
    barcodes::add(
        &db,
        &AddBarcode {
            product_package_id: pk.id.clone(),
            barcode: "0012345678901".into(),
            is_primary: true,
        },
    )
    .unwrap();
    let d = products::detail(&db, &p.id).unwrap();
    assert_eq!(d.manufacturer.unwrap().id, m.id);
    assert_eq!(d.category.unwrap().id, c.id);
    assert_eq!(d.route.unwrap().id, r.id);
    assert_eq!(d.active_ingredients.len(), 2);
    assert_eq!(d.active_ingredients[0].position, 0);
    assert_eq!(
        d.product.scientific_name.as_deref(),
        Some("Original A+B text")
    );
    assert_eq!(d.packages.len(), 2);
    let priced = d.packages.iter().find(|p| p.package.id == pk.id).unwrap();
    assert_eq!(priced.current_price.as_ref().unwrap().id, price.id);
    assert_eq!(priced.barcodes[0].barcode, "0012345678901");
    assert!(d
        .packages
        .iter()
        .find(|p| p.package.id == other.id)
        .unwrap()
        .current_price
        .is_none());
    let dto = serde_json::to_value(products::detail(&db, &p.id).unwrap()).unwrap();
    assert!(dto["product"].get("commercialNameEn").is_some());
    assert!(dto.get("activeIngredients").is_some());
}
#[test]
fn packages_barcodes_and_default_constraints() {
    let db = AppDb::in_memory().unwrap();
    let p = product(&db);
    let pk = package(&db, &p.id);
    assert!(barcodes::list(&db, &pk.id).unwrap().is_empty());
    let other = package(&db, &p.id);
    assert_eq!(product_packages::list(&db, &p.id, false).unwrap().len(), 2);
    for (n, primary) in [("0000123", true), ("BOX-ABC", false)] {
        barcodes::add(
            &db,
            &AddBarcode {
                product_package_id: pk.id.clone(),
                barcode: n.into(),
                is_primary: primary,
            },
        )
        .unwrap();
    }
    assert_eq!(barcodes::list(&db, &pk.id).unwrap().len(), 2);
    assert!(barcodes::add(
        &db,
        &AddBarcode {
            product_package_id: other.id,
            barcode: "0000123".into(),
            is_primary: false
        }
    )
    .is_err());
    assert!(barcodes::add(
        &db,
        &AddBarcode {
            product_package_id: pk.id,
            barcode: "SECOND".into(),
            is_primary: true
        }
    )
    .is_err());
    let input = CreateProductPackage {
        product_id: p.id.clone(),
        package_label: "Default".into(),
        is_default: true,
        ..Default::default()
    };
    let default = product_packages::create(&db, &input).unwrap();
    assert!(product_packages::create(&db, &input).is_err());
    assert!(
        !product_packages::set_active(&db, &default.id, false)
            .unwrap()
            .is_default
    );
    assert!(product_packages::create(&db, &input).is_ok());
    assert!(product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: p.id,
            package_label: "Bad".into(),
            units_per_package: Some(0),
            ..Default::default()
        }
    )
    .is_err());
    assert!(product_packages::create(
        &db,
        &CreateProductPackage {
            product_id: uuid::Uuid::new_v4().to_string(),
            package_label: "Missing".into(),
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn append_prices_preserve_values_and_close_intervals() {
    let db = AppDb::in_memory().unwrap();
    let p = product(&db);
    let pk = package(&db, &p.id);
    assert!(price_history::current(&db, &pk.id).unwrap().is_none());
    let a = price(&db, &pk.id, 2250);
    let b = price(&db, &pk.id, 12500);
    let c = price(&db, &pk.id, 12501);
    let h = price_history::history(&db, &pk.id).unwrap();
    assert_eq!(h.len(), 3);
    assert_eq!(h[0].id, c.id);
    assert_eq!(h[1].id, b.id);
    assert_eq!(h[2].id, a.id);
    assert_eq!(h[2].selling_price_minor, 2250);
    assert_eq!(h[2].effective_to.as_ref(), Some(&b.effective_from));
    assert_eq!(h[1].effective_to.as_ref(), Some(&c.effective_from));
    assert!(b.effective_from > a.effective_from);
    assert_eq!(
        price_history::current(&db, &pk.id).unwrap().unwrap().id,
        c.id
    );
    assert!(price_history::set(
        &db,
        &SetPackagePrice {
            product_package_id: pk.id.clone(),
            selling_price_minor: -1,
            ..Default::default()
        }
    )
    .is_err());
    assert_eq!(price_history::history(&db, &pk.id).unwrap().len(), 3);
    let lock = db.lock().unwrap();
    assert!(lock
        .execute(
            "UPDATE product_price_history SET selling_price_minor=0 WHERE id=?1",
            [&a.id]
        )
        .is_err());
    assert!(lock
        .execute("DELETE FROM product_price_history WHERE id=?1", [&a.id])
        .is_err());
    assert!(lock.execute("INSERT INTO product_price_history(id,product_package_id,selling_price_minor,effective_from,effective_to) VALUES (?1,?2,1,?3,?4)",params![uuid::Uuid::new_v4().to_string(),pk.id,a.effective_from,c.effective_from]).is_err());
}
#[test]
fn price_insert_failure_rolls_back_closure() {
    let db = AppDb::in_memory().unwrap();
    let p = product(&db);
    let pk = package(&db, &p.id);
    let old = price(&db, &pk.id, 100);
    db.lock().unwrap().execute_batch("CREATE TRIGGER test_reject_price BEFORE INSERT ON product_price_history BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(price_history::set(
        &db,
        &SetPackagePrice {
            product_package_id: pk.id.clone(),
            selling_price_minor: 200,
            ..Default::default()
        }
    )
    .is_err());
    assert_eq!(
        price_history::current(&db, &pk.id).unwrap().unwrap().id,
        old.id
    );
    assert_eq!(price_history::history(&db, &pk.id).unwrap().len(), 1);
}
#[test]
fn money_parser_is_exact_and_rejects_uncertain_source_values() {
    for (text, expected) in [
        ("22.50", 2250),
        ("125.00", 12500),
        ("0", 0),
        (" 1.2 ", 120),
        ("90071992547409.91", MAX_SAFE_MINOR),
    ] {
        assert_eq!(egp_to_minor(text).unwrap(), expected);
    }
    for text in [
        "-1",
        "1.234",
        "1e2",
        "NaN",
        "1,000",
        "",
        ".50",
        "1.",
        "90071992547409.92",
    ] {
        assert!(egp_to_minor(text).is_err(), "{text}");
    }
}
#[test]
fn conservative_normalization_does_not_merge_arabic_spelling_or_scientific_punctuation() {
    assert_eq!(normalize("  APEX\t Pharma  "), "apex pharma");
    assert_eq!(normalize("أَلف  باء"), "أَلف باء");
    assert_ne!(normalize("ألف"), normalize("الف"));
    assert_ne!(normalize("دواء+أ"), normalize("دواء أ"));
}
#[test]
fn inactive_product_package_and_search_behavior() {
    let db = AppDb::in_memory().unwrap();
    let p = product(&db);
    let pk = package(&db, &p.id);
    barcodes::add(
        &db,
        &AddBarcode {
            product_package_id: pk.id.clone(),
            barcode: "000001234".into(),
            is_primary: true,
        },
    )
    .unwrap();
    for query in ["test", "دواء", "000001234"] {
        assert_eq!(
            products::list(
                &db,
                &ProductQuery {
                    search: Some(query.into()),
                    ..Default::default()
                }
            )
            .unwrap()
            .total,
            1
        );
    }
    assert_eq!(
        products::list(
            &db,
            &ProductQuery {
                search: Some("%".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        0
    );
    assert_eq!(
        products::list(
            &db,
            &ProductQuery {
                limit: Some(1),
                offset: Some(1),
                ..Default::default()
            }
        )
        .unwrap()
        .items
        .len(),
        0
    );
    assert!(products::list(
        &db,
        &ProductQuery {
            limit: Some(201),
            ..Default::default()
        }
    )
    .is_err());
    price(&db, &pk.id, 100);
    product_packages::set_active(&db, &pk.id, false).unwrap();
    assert!(product_packages::list(&db, &p.id, false)
        .unwrap()
        .is_empty());
    assert_eq!(product_packages::list(&db, &p.id, true).unwrap().len(), 1);
    assert!(price_history::set(
        &db,
        &SetPackagePrice {
            product_package_id: pk.id.clone(),
            selling_price_minor: 200,
            ..Default::default()
        }
    )
    .is_err());
    assert!(price_history::current(&db, &pk.id).unwrap().is_some());
    product_packages::set_active(&db, &pk.id, true).unwrap();
    products::set_active(&db, &p.id, false).unwrap();
    assert_eq!(
        products::list(&db, &ProductQuery::default()).unwrap().total,
        0
    );
    assert_eq!(
        products::list(
            &db,
            &ProductQuery {
                include_inactive: true,
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        1
    );
    assert!(!products::detail(&db, &p.id).unwrap().product.is_active);
    assert!(price_history::set(
        &db,
        &SetPackagePrice {
            product_package_id: pk.id,
            selling_price_minor: 200,
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn direct_database_constraints_enforce_integer_money_and_relations() {
    let db = AppDb::in_memory().unwrap();
    let p = product(&db);
    let pk = package(&db, &p.id);
    let c = db.lock().unwrap();
    assert!(c.execute("INSERT INTO product_price_history(id,product_package_id,selling_price_minor,effective_from) VALUES('bad',?1,22.5,'2026-01-01T00:00:00.000Z')",[&pk.id]).is_err());
    assert!(c
        .execute(
            "INSERT INTO barcodes(id,product_package_id,barcode) VALUES ('x','missing','123')",
            []
        )
        .is_err());
    assert!(c
        .execute("DELETE FROM products WHERE id=?1", [&p.id])
        .is_err());
    assert!(c
        .execute("INSERT INTO products(id) VALUES('nameless')", [])
        .is_err());
}
#[test]
fn concurrent_price_changes_leave_one_current_and_complete_history() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("prices.db");
    let one = AppDb::open(&path).unwrap();
    let p = product(&one);
    let pk = package(&one, &p.id);
    let two = AppDb::open(&path).unwrap();
    let id = pk.id.clone();
    let first = std::thread::spawn(move || price(&one, &id, 100));
    let id = pk.id.clone();
    let second = std::thread::spawn(move || price(&two, &id, 200));
    first.join().unwrap();
    second.join().unwrap();
    let db = AppDb::open(&path).unwrap();
    let history = price_history::history(&db, &pk.id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(
        history.iter().filter(|p| p.effective_to.is_none()).count(),
        1
    );
    assert_eq!(
        history[1].effective_to.as_ref(),
        Some(&history[0].effective_from)
    );
}
#[test]
fn representative_csv_rows_fit_without_import_or_medical_parsing() {
    let profile: serde_json::Value =
        serde_json::from_str(include_str!("../../docs/catalog-reference-profile.json")).unwrap();
    assert_eq!(profile["rowCount"], 25070);
    let samples = profile["representativeRows"].as_array().unwrap();
    assert_eq!(samples.len(), 6);
    for row in samples {
        let db = AppDb::in_memory().unwrap();
        let text = |key: &str| {
            row[key]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .map(str::to_owned)
        };
        let manufacturer = text("manufacturer").map(|name| {
            manufacturers::create(
                &db,
                &CreateManufacturer {
                    name,
                    ..Default::default()
                },
            )
            .unwrap()
            .id
        });
        let category = text("drug_class").map(|name| {
            categories::create(
                &db,
                &CreateCategory {
                    name,
                    ..Default::default()
                },
            )
            .unwrap()
            .id
        });
        let route =
            text("route").map(|name| routes::create(&db, &CreateRoute { name }).unwrap().id);
        let p = products::create(
            &db,
            &CreateProduct {
                commercial_name_en: text("commercial_name_en"),
                commercial_name_ar: text("commercial_name_ar"),
                scientific_name: text("scientific_name"),
                manufacturer_id: manufacturer,
                category_id: category,
                route_id: route,
                ..Default::default()
            },
        )
        .unwrap();
        // The source has no package structure. An explicitly synthetic test package only tests price representability.
        let pk = package(&db, &p.id);
        let amount = egp_to_minor(row["price_egp"].as_str().unwrap()).unwrap();
        price(&db, &pk.id, amount);
        let detail = products::detail(&db, &p.id).unwrap();
        assert_eq!(
            detail.product.commercial_name_ar,
            text("commercial_name_ar")
        );
        assert_eq!(detail.product.scientific_name, text("scientific_name"));
        assert!(detail.active_ingredients.is_empty());
        assert!(detail.packages[0].barcodes.is_empty());
    }
}
#[test]
fn create_full_atomically_persists_all_related_rows() {
    let db = AppDb::in_memory().unwrap();
    let m = manufacturers::create(
        &db,
        &CreateManufacturer {
            name: "Maker".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let i = active_ingredients::create(
        &db,
        &CreateActiveIngredient {
            name: "Ibuprofen".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let product = products::create_full(
        &db,
        &CreateProductFull {
            commercial_name_en: Some("Brufen".into()),
            manufacturer_id: Some(m.id.clone()),
            active_ingredients: vec![ProductIngredientInput {
                active_ingredient_id: i.id.clone(),
                strength_text: Some("400 mg".into()),
            }],
            packages: vec![ProductPackageInput {
                units_per_package: Some(20),
                is_default: true,
                barcodes: vec![ProductBarcodeInput {
                    barcode: "6291041500109".into(),
                    is_primary: true,
                }],
                selling_price_minor: Some(2250),
                ..package_input("20 tablets")
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let d = products::detail(&db, &product.id).unwrap();
    assert_eq!(d.manufacturer.unwrap().id, m.id);
    assert_eq!(d.active_ingredients.len(), 1);
    assert_eq!(d.active_ingredients[0].active_ingredient.id, i.id);
    assert_eq!(d.packages.len(), 1);
    assert_eq!(d.packages[0].package.units_per_package, Some(20));
    assert!(d.packages[0].package.is_default);
    assert_eq!(d.packages[0].barcodes[0].barcode, "6291041500109");
    assert_eq!(
        d.packages[0]
            .current_price
            .as_ref()
            .unwrap()
            .selling_price_minor,
        2250
    );
}
#[test]
fn create_full_failures_roll_back_the_whole_product() {
    let db = AppDb::in_memory().unwrap();
    let err = products::create_full(
        &db,
        &CreateProductFull {
            commercial_name_en: Some("Ghost".into()),
            active_ingredients: vec![ProductIngredientInput {
                active_ingredient_id: uuid::Uuid::new_v4().to_string(),
                strength_text: None,
            }],
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
    assert_eq!(
        products::list(&db, &ProductQuery::default()).unwrap().total,
        0
    );
    assert_eq!(
        products::create_full(&db, &CreateProductFull::default())
            .unwrap_err()
            .code,
        ErrorCode::Validation
    );
    assert_eq!(
        products::list(&db, &ProductQuery::default()).unwrap().total,
        0
    );
    products::create_full(
        &db,
        &CreateProductFull {
            commercial_name_en: Some("Ghost package".into()),
            packages: vec![ProductPackageInput::default()],
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(
        products::list(&db, &ProductQuery::default()).unwrap().total,
        0
    );
}
#[test]
fn barcodes_cannot_cross_products_or_duplicate_within_one() {
    let db = AppDb::in_memory().unwrap();
    let mk = |name: &str, barcode: &str, primary: bool| {
        products::create_full(
            &db,
            &CreateProductFull {
                commercial_name_en: Some(name.into()),
                packages: vec![ProductPackageInput {
                    barcodes: vec![ProductBarcodeInput {
                        barcode: barcode.into(),
                        is_primary: primary,
                    }],
                    ..package_input(name)
                }],
                ..Default::default()
            },
        )
    };
    let first = mk("First", "SHARED", true).unwrap();
    assert_eq!(
        mk("Second", "SHARED", false).unwrap_err().code,
        ErrorCode::Conflict
    );
    assert_eq!(
        products::list(&db, &ProductQuery::default()).unwrap().total,
        1
    );
    let err = products::create_full(
        &db,
        &CreateProductFull {
            commercial_name_en: Some("Third".into()),
            packages: vec![ProductPackageInput {
                barcodes: vec![
                    ProductBarcodeInput {
                        barcode: "DUPLICATE".into(),
                        is_primary: true,
                    },
                    ProductBarcodeInput {
                        barcode: "DUPLICATE".into(),
                        is_primary: false,
                    },
                ],
                ..package_input("Third")
            }],
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::Validation);
    assert_eq!(
        products::list(&db, &ProductQuery::default()).unwrap().total,
        1
    );
    let second = mk("Second", "ANOTHER", true).unwrap();
    let err = products::update_full(
        &db,
        &UpdateProductFull {
            id: second.id.clone(),
            commercial_name_en: Some("Second".into()),
            is_active: true,
            packages: vec![ProductPackageInput {
                barcodes: vec![ProductBarcodeInput {
                    barcode: "SHARED".into(),
                    is_primary: false,
                }],
                ..package_input("Second-edit")
            }],
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::Conflict);
    assert_eq!(
        products::detail(&db, &first.id).unwrap().packages[0].barcodes[0].barcode,
        "SHARED"
    );
}
#[test]
fn update_full_rejects_foreign_packages_and_appends_prices() {
    let db = AppDb::in_memory().unwrap();
    let a = products::create_full(
        &db,
        &CreateProductFull {
            commercial_name_en: Some("A".into()),
            packages: vec![package_input("A-box")],
            ..Default::default()
        },
    )
    .unwrap();
    let b = products::create_full(
        &db,
        &CreateProductFull {
            commercial_name_en: Some("B".into()),
            packages: vec![package_input("B-box")],
            ..Default::default()
        },
    )
    .unwrap();
    let b_pkg = products::detail(&db, &b.id).unwrap().packages[0]
        .package
        .id
        .clone();
    assert_eq!(
        products::update_full(
            &db,
            &UpdateProductFull {
                id: a.id.clone(),
                commercial_name_en: Some("A".into()),
                is_active: true,
                packages: vec![ProductPackageInput {
                    id: Some(b_pkg),
                    ..package_input("Hijacked")
                }],
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    );
    assert_eq!(
        products::update_full(
            &db,
            &UpdateProductFull {
                id: uuid::Uuid::new_v4().to_string(),
                commercial_name_en: Some("Missing".into()),
                is_active: true,
                packages: vec![],
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::NotFound
    );
    products::update_full(
        &db,
        &UpdateProductFull {
            id: a.id.clone(),
            commercial_name_en: Some("A".into()),
            is_active: true,
            packages: vec![ProductPackageInput {
                barcodes: vec![ProductBarcodeInput {
                    barcode: "AAA111".into(),
                    is_primary: true,
                }],
                selling_price_minor: Some(100),
                is_default: true,
                ..package_input("Starter")
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let d = products::detail(&db, &a.id).unwrap();
    let pkg_id = d.packages[0].package.id.clone();
    products::update_full(
        &db,
        &UpdateProductFull {
            id: a.id.clone(),
            commercial_name_en: Some("A".into()),
            is_active: true,
            packages: vec![ProductPackageInput {
                id: Some(pkg_id.clone()),
                barcodes: vec![ProductBarcodeInput {
                    barcode: "BBB222".into(),
                    is_primary: true,
                }],
                selling_price_minor: Some(200),
                is_default: true,
                ..package_input("Starter")
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let d = products::detail(&db, &a.id).unwrap();
    let pk = &d.packages[0];
    assert_eq!(pk.barcodes[0].barcode, "BBB222");
    assert_eq!(pk.current_price.as_ref().unwrap().selling_price_minor, 200);
    let history = price_history::history(&db, &pkg_id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].selling_price_minor, 100);
    assert!(history[0].effective_to.is_none());
    products::update_full(
        &db,
        &UpdateProductFull {
            id: a.id,
            commercial_name_en: Some("A".into()),
            is_active: true,
            packages: vec![ProductPackageInput {
                id: Some(pkg_id.clone()),
                barcodes: vec![ProductBarcodeInput {
                    barcode: "CCC333".into(),
                    is_primary: false,
                }],
                selling_price_minor: Some(200),
                is_default: true,
                ..package_input("Starter")
            }],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(price_history::history(&db, &pkg_id).unwrap().len(), 2);
}
#[test]
fn list_enriches_rows_and_enforces_sort_whitelist() {
    let db = AppDb::in_memory().unwrap();
    let m = manufacturers::create(
        &db,
        &CreateManufacturer {
            name: "Zeta".into(),
            ..Default::default()
        },
    )
    .unwrap();
    products::create_full(
        &db,
        &CreateProductFull {
            commercial_name_en: Some("Alpha".into()),
            manufacturer_id: Some(m.id.clone()),
            packages: vec![ProductPackageInput {
                is_default: true,
                barcodes: vec![ProductBarcodeInput {
                    barcode: "1001".into(),
                    is_primary: true,
                }],
                selling_price_minor: Some(300),
                ..package_input("Small")
            }],
            ..Default::default()
        },
    )
    .unwrap();
    products::create_full(
        &db,
        &CreateProductFull {
            commercial_name_en: Some("BETA".into()),
            packages: vec![ProductPackageInput {
                is_default: true,
                barcodes: vec![ProductBarcodeInput {
                    barcode: "1002".into(),
                    is_primary: true,
                }],
                selling_price_minor: Some(100),
                ..package_input("Big")
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let price_desc = products::list(
        &db,
        &ProductQuery {
            sort: Some("price".into()),
            sort_direction: Some("desc".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(price_desc.items[0].name_en.as_deref(), Some("Alpha"));
    assert_eq!(price_desc.items[1].name_en.as_deref(), Some("BETA"));
    let by_name = products::list(
        &db,
        &ProductQuery {
            sort: Some("name".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(by_name.items[0].name_en.as_deref(), Some("Alpha"));
    assert_eq!(by_name.items[0].manufacturer_name.as_deref(), Some("Zeta"));
    assert_eq!(by_name.items[0].barcode.as_deref(), Some("1001"));
    assert_eq!(by_name.items[0].selling_price_minor, Some(300));
    assert_eq!(by_name.items[0].package_label.as_deref(), Some("Small"));
    for term in ["alpha", "ZETA", "1002"] {
        assert_eq!(
            products::list(
                &db,
                &ProductQuery {
                    search: Some(term.into()),
                    ..Default::default()
                }
            )
            .unwrap()
            .total,
            1,
            "search {term}"
        );
    }
    let filtered = products::list(
        &db,
        &ProductQuery {
            manufacturer_id: Some(m.id),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(filtered.total, 1);
    assert_eq!(filtered.active_total, 1);
    let both = products::list(
        &db,
        &ProductQuery {
            search: Some("alpha".into()),
            sort: Some("updatedAt".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(both.total, 1);
    let page2 = products::list(
        &db,
        &ProductQuery {
            limit: Some(1),
            offset: Some(1),
            sort: Some("name".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page2.items.len(), 1);
    assert_eq!(page2.total, 2);
    assert_eq!(page2.items[0].name_en.as_deref(), Some("BETA"));
    assert!(products::list(
        &db,
        &ProductQuery {
            sort: Some("id".into()),
            ..Default::default()
        }
    )
    .is_err());
    assert!(products::list(
        &db,
        &ProductQuery {
            sort_direction: Some("sideways".into()),
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn active_total_counts_only_active_products() {
    let db = AppDb::in_memory().unwrap();
    let a = product(&db);
    let b = product(&db);
    products::set_active(&db, &b.id, false).unwrap();
    let page = products::list(&db, &ProductQuery::default()).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.active_total, 1);
    let page = products::list(
        &db,
        &ProductQuery {
            include_inactive: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.active_total, 1);
    assert_eq!(page.items.len(), 2);
    assert!(page.items.iter().any(|i| i.id == a.id));
}
#[test]
fn composite_input_validates_defaults_prices_and_units() {
    let db = AppDb::in_memory().unwrap();
    let p = product(&db);
    assert_eq!(
        products::update_full(
            &db,
            &UpdateProductFull {
                id: p.id.clone(),
                commercial_name_en: Some("X".into()),
                is_active: true,
                packages: vec![
                    ProductPackageInput {
                        is_default: true,
                        ..package_input("One")
                    },
                    ProductPackageInput {
                        is_default: true,
                        ..package_input("Two")
                    }
                ],
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    );
    assert_eq!(
        products::create_full(
            &db,
            &CreateProductFull {
                commercial_name_en: Some("Y".into()),
                packages: vec![ProductPackageInput {
                    is_default: true,
                    is_active: false,
                    ..Default::default()
                }],
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    );
    assert_eq!(
        products::create_full(
            &db,
            &CreateProductFull {
                commercial_name_en: Some("Z".into()),
                packages: vec![ProductPackageInput {
                    selling_price_minor: Some(-5),
                    ..package_input("Priced")
                }],
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    );
    let id = uuid::Uuid::new_v4().to_string();
    assert_eq!(
        products::update_full(
            &db,
            &UpdateProductFull {
                id: p.id,
                commercial_name_en: Some("W".into()),
                is_active: true,
                packages: vec![
                    ProductPackageInput {
                        id: Some(id.clone()),
                        ..package_input("A")
                    },
                    ProductPackageInput {
                        id: Some(id),
                        ..package_input("B")
                    }
                ],
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Validation
    );
}
