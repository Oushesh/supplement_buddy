// src/seed.rs
use sea_orm::*;
use crate::entities::{prelude::*, supplements};

pub async fn run(db: &DatabaseConnection) -> Result<(), DbErr> {
    println!("Seeding database...");

    let supplements = vec![
        ("Whey Protein", "Optimum Nutrition", "Protein"),
        ("Creatine", "MuscleTech", "Performance"),
        ("Multivitamin", "Nature Made", "Health"),
    ];

    for (name, brand, category) in supplements {
        // Check if it already exists so we don't double-seed
        let exists = Supplements::find()
            .filter(supplements::Column::Name.eq(name))
            .one(db)
            .await?;

        if exists.is_none() {
            let item = supplements::ActiveModel {
                name: Set(name.to_owned()),
                brand: Set(brand.to_owned()),
                category: Set(category.to_owned()),
                ..Default::default()
            };
            item.insert(db).await?;
            println!("  - Inserted: {}", name);
        }
    }

    println!("Seeding complete!");
    Ok(())
}

//This one requires seeding.

