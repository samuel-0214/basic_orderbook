use std::{cmp::Reverse, collections::BTreeMap, ffi::FromBytesUntilNulError, vec};
use serde::{Deserialize,Serialize};
use crate::types::{CreateOrder, CreateOrderResponse, DeleteOrder, DeleteOrderResponse, OrderType, Side, Trade};
use ordered_float::OrderedFloat;

#[derive(Clone)]
pub struct OpenOrder{
    pub price: f64,
    pub quantity : f64,
    pub side : Side,
    pub user_id: String,
    pub order_id: String,
    pub filled_quantity: f64,
}

pub struct OrderBook{
    pub bids: BTreeMap<Reverse<OrderedFloat<f64>>, Vec<OpenOrder>>,
    pub asks: BTreeMap<OrderedFloat<f64>, Vec<OpenOrder>>,
    pub order_id_index: u64,
    pub trades: Vec<Trade>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Depth{
    pub price: f64,
    pub quantity: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DepthResponse{
    pub bids: Vec<Depth>,
    pub asks: Vec<Depth>,
}

impl Default for OrderBook{
    fn default() -> Self {
        Self { bids: BTreeMap::new(), asks: BTreeMap::new(), order_id_index: 0, trades: vec![], }
    }
}

impl OrderBook{
    pub fn create_order(&mut self, order: CreateOrder) -> CreateOrderResponse {
    let order_id = self.order_id_index.to_string();
    self.order_id_index += 1;

    match order.order_type {
        OrderType::Market => {
            let filled_quantity = match order.side {
                Side::Buy => self.match_market_buy(order.quantity, &order.user_id),
                Side::Sell => self.match_market_sell(order.quantity, &order.user_id),
            };

            CreateOrderResponse {
                order_id,
                filled_quantity,
                remaining_quantity: order.quantity - filled_quantity,
                average_price: 0.0,
            }
        }

        OrderType::Limit => {
            let open_order = OpenOrder {
                price: order.price,
                quantity: order.quantity,
                side: order.side.clone(),
                user_id: order.user_id,
                order_id: order_id.clone(),
                filled_quantity: 0.0,
            };

            let filled_quantity = match order.side {
                Side::Buy => self.match_limit_buy(open_order),
                Side::Sell => self.match_limit_sell(open_order),
            };

            CreateOrderResponse {
                order_id,
                filled_quantity,
                remaining_quantity: order.quantity - filled_quantity,
                average_price: order.price,
            }
        }
    }
}


    pub fn delete_order(&mut self,order: DeleteOrder){
    if let Some(price) = self.bids.iter().find_map(|(price,orders)|{
        if orders.iter().any(|o|o.order_id == order.order_id){
            Some(price.clone())
        }else{
            None
        }
    }){
        if let Some(orders) = self.bids.get_mut(&price){
            orders.retain(|o|o.order_id != order.order_id);
        }
    }

     if let Some(price) = self.asks.iter().find_map(|(price, orders)| {
            if orders.iter().any(|o| o.order_id == order.order_id) {
                Some(price.clone())
            } else {
                None
            }
        }) {
            if let Some(orders) = self.asks.get_mut(&price) {
                orders.retain(|o| o.order_id != order.order_id);
            }
        }
}

pub fn get_depth(&self) -> DepthResponse {
    let mut bids = Vec::new();
    let mut asks = Vec::new();

    for (price, orders) in self.bids.iter() {
        let total_qty: f64 = orders.iter()
            .map(|o| o.quantity - o.filled_quantity)
            .filter(|q| *q > 0.0)
            .sum();

        if total_qty > 0.0 {
            bids.push(Depth {
                price: price.0.into_inner(),
                quantity: total_qty,
            });
        }
    }

    for (price, orders) in self.asks.iter() {
        let total_qty: f64 = orders.iter()
            .map(|o| o.quantity - o.filled_quantity)
            .filter(|q| *q > 0.0)
            .sum();

        if total_qty > 0.0 {
            asks.push(Depth {
                price: price.into_inner(),
                quantity: total_qty,
            });
        }
    }

    DepthResponse { bids, asks }
}


fn match_market_buy(&mut self, mut quantity: f64, user_id: &str) -> f64 {
    let mut to_remove = vec![];
    let mut filled = 0.0;
    let mut trades_to_record = vec![];

    for (price, orders) in self.asks.iter_mut() {
        for order in orders.iter_mut() {
            if quantity <= 0.0 {
                break;
            }

            let available = order.quantity - order.filled_quantity;
            let trade_qty = quantity.min(available);

            order.filled_quantity += trade_qty;
            quantity -= trade_qty;
            filled += trade_qty;

            trades_to_record.push((
                price.into_inner(),
                trade_qty,
                user_id.to_string(),  
                order.user_id.clone(),
            ));
        }

        orders.retain(|o| o.filled_quantity < o.quantity);
        if orders.is_empty() {
            to_remove.push(*price);
        }

        if quantity <= 0.0 {
            break;
        }
    }

    for price in to_remove {
        self.asks.remove(&price);
    }

    for (price, qty, buyer, seller) in trades_to_record {
        self.record_trade(price, qty, &buyer, &seller);
        println!("TRADE: {} bought {} @ {} from {}", buyer, qty, price, seller);
    }

    if quantity > 0.0 {
        println!("Unfilled buy quantity: {}", quantity);
    }

    filled
}


fn match_market_sell(&mut self, mut quantity: f64, user_id: &str) -> f64 {
    let mut to_remove = vec![];
    let mut filled = 0.0;
    let mut trades_to_record = vec![];

    for (price, orders) in self.bids.iter_mut() {
        for order in orders.iter_mut() {
            if quantity <= 0.0 {
                break;
            }

            let available = order.quantity - order.filled_quantity;
            let trade_qty = quantity.min(available);

            order.filled_quantity += trade_qty;
            quantity -= trade_qty;
            filled += trade_qty;

            trades_to_record.push((
                price.0.into_inner(),
                trade_qty,
                order.user_id.clone(), 
                user_id.to_string(),   
            ));
        }

        orders.retain(|o| o.filled_quantity < o.quantity);
        if orders.is_empty() {
            to_remove.push(*price);
        }

        if quantity <= 0.0 {
            break;
        }
    }

    for price in to_remove {
        self.bids.remove(&price);
    }

    for (price, qty, buyer, seller) in trades_to_record {
        self.record_trade(price, qty, &buyer, &seller);
        println!("TRADE: {} sold {} @ {} to {}", seller, qty, price, buyer);
    }

    if quantity > 0.0 {
        println!("Unfilled sell quantity: {}", quantity);
    }

    filled
}


fn record_trade(&mut self,price:f64,quantity: f64,buyer_id : &str, seller_id: &str){
    let trade = Trade{
        price,
        quantity,
        buyer_id: buyer_id.to_string(),
        seller_id: seller_id.to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };
    self.trades.push(trade);

    println!(
    "Trade: {} bought {} from {} @ {}",
    buyer_id, quantity, seller_id, price
);


}

fn match_limit_buy(&mut self, mut order: OpenOrder) -> f64 {
    let mut filled = 0.0;
    let mut to_remove = vec![];

    let mut trades_to_record = vec![];
    
    let price_levels: Vec<OrderedFloat<f64>> = self
        .asks
        .keys()
        .cloned()
        .filter(|price| price.into_inner() <= order.price)
        .collect();

    for price in price_levels {
        if let Some(orders) = self.asks.get_mut(&price) {
            for existing_order in orders.iter_mut() {
                if order.quantity <= 0.0 {
                    break;
                }

                let available = existing_order.quantity - existing_order.filled_quantity;
                let trade_qty = order.quantity.min(available);

                existing_order.filled_quantity += trade_qty;
                order.quantity -= trade_qty;
                filled += trade_qty;

                trades_to_record.push((
                    price.into_inner(),
                    trade_qty,
                    order.user_id.clone(),
                    existing_order.user_id.clone()
                ));
            }

            orders.retain(|o| o.filled_quantity < o.quantity);

            if orders.is_empty() {
                to_remove.push(price);
            }

            if order.quantity <= 0.0 {
                break;
            }
        }
    }

    for price in to_remove {
        self.asks.remove(&price);
    }

    if order.quantity > 0.0 {
        self.bids
            .entry(Reverse(OrderedFloat(order.price)))
            .or_insert_with(Vec::new)
            .push(OpenOrder {
                quantity: order.quantity,
                ..order
            });
    }

    for (price, qty, buyer, seller) in trades_to_record{
        self.record_trade(price, qty, &buyer, &seller);
    }

    filled
}

fn match_limit_sell(&mut self, mut order: OpenOrder) -> f64 {
    let mut filled = 0.0;
    let mut to_remove = vec![];

    let mut trades_to_record = vec![];

    let price_levels: Vec<Reverse<OrderedFloat<f64>>> = self
        .bids
        .keys()
        .cloned()
        .filter(|price| price.0.into_inner() >= order.price)
        .collect();

    for price in price_levels {
        if let Some(orders) = self.bids.get_mut(&price) {
            for existing_order in orders.iter_mut() {
                if order.quantity <= 0.0 {
                    break;
                }

                let available = existing_order.quantity - existing_order.filled_quantity;
                let trade_qty = order.quantity.min(available);

                existing_order.filled_quantity += trade_qty;
                order.quantity -= trade_qty;
                filled += trade_qty;

                trades_to_record.push((
                    price.0.into_inner(),
                    trade_qty,
                    existing_order.user_id.clone(),
                    order.user_id.clone(),
                ));
            }

            orders.retain(|o| o.filled_quantity < o.quantity);
            if orders.is_empty() {
                to_remove.push(price);
            }

            if order.quantity <= 0.0 {
                break;
            }
        }
    }

    for price in to_remove {
        self.bids.remove(&price);
    }

    if order.quantity > 0.0 {
        self.asks
            .entry(OrderedFloat(order.price))
            .or_insert_with(Vec::new)
            .push(OpenOrder {
                quantity: order.quantity,
                ..order
            });
    }

    for (price, qty, buyer, seller) in trades_to_record {
        self.record_trade(price, qty, &buyer, &seller);
    }

    filled
}
}
