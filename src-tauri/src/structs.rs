use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Order {
    pub order_number: String,
    pub part_number: String,
    pub due_quantity: f64,
    pub assn_number: String,
}

#[derive(Serialize)]
pub struct PrintOrder {
    pub order_number: String,
    pub part_number: String,
    pub due_quantity: f64,
    pub assn_number: String,
    pub print_type: String,
    pub notes: String,
}


#[derive(Deserialize)]
pub struct PrintOrderRow {
    id: i32,
    pub print_type: String,
    pub notes: String,
}