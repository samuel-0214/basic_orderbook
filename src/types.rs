use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateOrder{
    pub price:f64,
    pub quantity: f64,
    pub user_id: String,
    pub side: Side,
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub enum Side{
    Buy,
    Sell
}

#[derive(Debug,Serialize,Deserialize)]
pub struct DeleteOrder{
    pub order_id:String,
    pub user_id:String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct CreateOrderResponse{
    pub order_id: String,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
    pub average_price: f64
}

#[derive(Debug,Serialize,Deserialize)]
pub struct DeleteOrderResponse{
    pub succes: bool,
    pub filled_quantity:f64,
    pub remaining_quantity:f64,
    pub average_price:f64,
}