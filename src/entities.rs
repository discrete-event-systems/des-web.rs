//! SeaORM entities for the des-web OVERLAY tables (schema/des-web.sql).
//!
//! Contract tables (des_soccer_*, des_fel_elevator_*, …) come from the
//! generated `dd_pg_defs_sea_orm` crate in the pg-defs submodule — never
//! duplicate those here. These two entities mirror schema/des-web.sql, which is
//! their single source of truth; do not infer migrations from this file.

pub mod des_web_sims {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "des_web_sims")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub slug: String,
        pub title: String,
        pub blurb: String,
        pub kind: String,
        pub page_route: String,
        pub source_service: String,
        pub engine: String,
        pub tags: Json,
        pub sort_order: i32,
        pub is_enabled: bool,
        pub created_at: DateTimeWithTimeZone,
        pub updated_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod des_web_routing_solves {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "des_web_routing_solves")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub status: String,
        pub stop_count: i32,
        pub vehicles: i32,
        pub restarts_total: i32,
        pub restarts_done: i32,
        pub improvements: i32,
        pub seed: i64,
        pub best_distance: Option<f64>,
        pub depot_index: i32,
        pub stops: Json,
        pub routes: Json,
        pub created_at: DateTimeWithTimeZone,
        pub updated_at: DateTimeWithTimeZone,
        pub finished_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
