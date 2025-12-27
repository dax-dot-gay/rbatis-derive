use rbatis::{rbdc::Uuid, table_sync::PGTableMapper};
use rbatis_derive::Schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Schema)]
#[schema(table(name = "test"))]
struct TestModel {
    #[field(select)]
    pub id: Uuid,

    #[field(not_null, unique, select)]
    pub name: String,
    pub description: Option<String>
}


fn main() {
    println!("{:?}", TestModel::fields());
    println!("{:?}", TestModel::field_type("id", &PGTableMapper {}));
    println!("{:?}", TestModel::field_constraints("name", &PGTableMapper {}));
}
