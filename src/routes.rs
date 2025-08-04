use std::sync::{Arc,Mutex};
use actix_web::{cookie::time::macros::date, delete, get, post, web::{Data,Json}, HttpResponse, Responder};
use crate::{orderbook::{self, OrderBook}, types::{CreateOrder,DeleteOrder}};

#[get("/depth")]
pub async fn get_depth(orderbook: Data<Arc<Mutex<OrderBook>>>) -> impl Responder{
    let orderbook = orderbook.lock().unwrap();
    let depth = orderbook.get_depth();
    HttpResponse::Ok().json(depth)
}

#[post("/order")]
pub async fn create_order(orderbook: Data<Arc<Mutex<OrderBook>>>, order: Json<CreateOrder>) -> impl Responder{
    let mut orderbook = orderbook.lock().unwrap();
    let orderbook = orderbook.create_order(order.0);
    HttpResponse::Ok().json(orderbook)
}

#[delete("/order")]
pub async fn delete_order(orderbook: Data<Arc<Mutex<OrderBook>>>,order : Json<DeleteOrder>) -> impl Responder{
    let mut orderbook = orderbook.lock().unwrap();
    let orderbook = orderbook.delete_order(order.0);
    HttpResponse::Ok().json(orderbook)
}

#[get("/trade")]
async fn get_trade(data: Data<Mutex<OrderBook>>) -> impl Responder{
    let orderbook = data.lock().unwrap();
    HttpResponse::Ok().json(&orderbook.trades)
}
