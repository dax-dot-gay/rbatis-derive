# rbatis-derive
A derive macro to simplify implementing schemas in [RBatis](https://crates.io/crates/rbatis)

**Basic Usage:**

```rust
use derive_schema::Schema;
use serde::{Serialize, Deserialize};

// Should at least have `Serialize`, `Deserialize`, and `Schema`
#[derive(Clone, Debug, Serialize, Deserialize, Schema)]
// Define the table name for this schema. 
// If omitted, will use the snake_case variant of the struct's name (in this case, "example_model")
#[schema(table(name = "example_model_table"))]
pub struct ExampleModel {
    // The primary key field is auto-detected based on the field name (`id`).
    // All schemas should have an `id` field, due to limitations within RBatis.
    // Per-field options can be configured with the `#[field(...)]` attribute.
    // Fields with `#[field(..., select, ...)]` will automatically add `select_with_{name}` and `select_one_with_{name}` methods to the model implementation.
    #[field(select)]
    pub id: rbatis::rbdc::Uuid,
    
    // `unique` and `not_null` can be added to pass the respective constraints to the database.
    #[field(select, unique, not_null)]
    pub name: String,

    // `sql_type` can be added to override the automatic SQL type resolution (this may break database compatibility!)
    #[field(sql_type = "INT")]
    pub count: u64
}
```

Along with implementing the default CRUD methods for the model, this macro will also add the following methods:

- `ExampleModel::fields()`: Returns a `Vec<String>` containing the column names of this schema
- `ExampleModel::field_type(field: impl Into<String>, mapper: &dyn rbatis::table_sync::ColumnMapper)`: Returns the SQL type of the specified field, if the field exists.
- `ExampleModel::field_constraints(field: impl Into<String>, mapper: &dyn rbatis::table_sync::ColumnMapper)`: Returns the types and constraints of the specified field, if the field exists.
- `ExampleModel::sync(rb: &#rbatis::rbatis::RBatis, mapper: &dyn rbatis::table_sync::ColumnMapper).await`: Synchronizes the schema with the database using `rbatis::table_sync`
- `ExampleModel::save(&self, database: &#rbatis::RBatis).await`: Updates or inserts the model into the database
- `ExampleModel::delete(&self, database: &#rbatis::RBatis).await`: Deletes this model from the database
