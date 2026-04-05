use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

use crate::entities::supplement;
use crate::migration::Migrator;

/// Connect to the database and run all pending migrations.
/// For file-based SQLite databases, creates the file if it does not already exist.
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    ensure_sqlite_file_exists(database_url);

    let opt = ConnectOptions::new(database_url.to_string())
        .max_connections(5)
        .to_owned();

    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

/// Public alias used by integration tests so they can set up an in-memory DB.
pub async fn run_migrations_for_test(db: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}

/// Seeds the database with a small set of sample supplements.
/// Skips insertion if the table already contains data, so it is safe to call
/// multiple times.
pub async fn seed_database(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::{EntityTrait, PaginatorTrait};
    use crate::entities::supplement::Entity as SupplementEntity;

    let existing = SupplementEntity::find().count(db).await?;
    if existing > 0 {
        tracing::info!("Database already has {existing} supplement(s) — skipping seed.");
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();

    let seeds: Vec<(&str, &str, &str, &str, &str, &str)> = vec![
        (
            "Vitamin C",
            "HealthPlus",
            "Vitamins",
            "Supports immune health and acts as a powerful antioxidant.",
            "Ascorbic Acid 500 mg",
            "1 capsule",
        ),
        (
            "Fish Oil",
            "OmegaBest",
            "Fatty Acids",
            "High-potency omega-3 supplement for cardiovascular and brain support.",
            "EPA 360 mg, DHA 240 mg, Fish Oil 1000 mg",
            "1 softgel",
        ),
        (
            "Magnesium Glycinate",
            "PureForm",
            "Minerals",
            "Highly bioavailable magnesium for muscle relaxation and sleep quality.",
            "Magnesium (as Glycinate) 200 mg",
            "2 capsules",
        ),
        (
            "Vitamin D3",
            "SunNutrition",
            "Vitamins",
            "Essential fat-soluble vitamin for bone health and immune function.",
            "Cholecalciferol (Vitamin D3) 2000 IU",
            "1 softgel",
        ),
        (
            "Creatine Monohydrate",
            "AthleteEdge",
            "Sports Nutrition",
            "Clinically proven to improve strength, power, and muscle mass.",
            "Creatine Monohydrate 5 g",
            "1 scoop (5 g)",
        ),
        (
            "Zinc",
            "NutriCore",
            "Minerals",
            "Supports immune defence, wound healing, and DNA synthesis.",
            "Zinc (as Gluconate) 15 mg",
            "1 tablet",
        ),
        (
            "Ashwagandha KSM-66",
            "AdaptogenLabs",
            "Adaptogens",
            "Clinically studied root extract for stress reduction and cortisol balance.",
            "Ashwagandha root extract (KSM-66) 600 mg",
            "1 capsule",
        ),
        (
            "Coenzyme Q10",
            "MitoEnergy",
            "Antioxidants",
            "Supports cellular energy production and cardiovascular health.",
            "Ubiquinone (CoQ10) 100 mg",
            "1 softgel",
        ),
    ];

    for (name, brand, category, description, ingredients, serving_size) in &seeds {
        let row = supplement::ActiveModel {
            name: Set(name.to_string()),
            brand: Set(brand.to_string()),
            category: Set(category.to_string()),
            description: Set(description.to_string()),
            ingredients: Set(ingredients.to_string()),
            serving_size: Set(serving_size.to_string()),
            splade_vector: Set(None),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
            ..Default::default()
        };
        row.insert(db).await?;
        tracing::info!("Seeded supplement: {name}");
    }

    tracing::info!("Seeding complete — {} supplements inserted.", seeds.len());
    Ok(())
}
/// does not yet exist. This mirrors the `create_if_missing` behaviour from sqlx
/// that sea-orm's `ConnectOptions` does not currently expose.
fn ensure_sqlite_file_exists(database_url: &str) {
    // Extract the filesystem path from common SQLite URL formats:
    //   sqlite://./relative/path.db  ->  ./relative/path.db
    //   sqlite:///absolute/path.db   ->  /absolute/path.db
    //   sqlite::memory:              ->  skip (in-memory)
    let path_str = if let Some(p) = database_url.strip_prefix("sqlite://./") {
        format!("./{p}")
    } else if let Some(p) = database_url.strip_prefix("sqlite:///") {
        format!("/{p}")
    } else {
        return; // in-memory, bare file, or unknown format — leave to the driver
    };

    if path_str.is_empty() {
        return;
    }

    let path = std::path::Path::new(&path_str);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        let _ = std::fs::File::create(path);
    }
}
