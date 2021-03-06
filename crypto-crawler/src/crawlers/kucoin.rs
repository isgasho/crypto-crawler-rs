use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

use std::{collections::HashMap, time::Duration};

use super::utils::{check_args, fetch_symbols_retry};
use crate::{msg::Message, MessageType};
use crypto_markets::MarketType;
use crypto_ws_client::*;
use log::*;
use serde_json::Value;

const EXCHANGE_NAME: &str = "kucoin";
// See https://docs.kucoin.cc/#request-rate-limit
const MAX_SUBSCRIPTIONS_PER_CONNECTION: usize = 300;

fn extract_symbol(json: &str) -> String {
    let obj = serde_json::from_str::<HashMap<String, Value>>(&json).unwrap();
    let topic = obj.get("topic").unwrap().as_str().unwrap();

    let colon_pos = topic.rfind(':').unwrap();
    (&topic[(colon_pos + 1)..]).to_string()
}

#[rustfmt::skip]
gen_crawl_event!(crawl_trade_spot, KuCoinSpotWSClient, MessageType::Trade, subscribe_trade);
#[rustfmt::skip]
gen_crawl_event!(crawl_trade_swap, KuCoinSwapWSClient, MessageType::Trade, subscribe_trade);
#[rustfmt::skip]

#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_spot, KuCoinSpotWSClient, MessageType::L2Event, subscribe_orderbook);
#[rustfmt::skip]
gen_crawl_event!(crawl_l2_event_swap, KuCoinSwapWSClient, MessageType::L2Event, subscribe_orderbook);

#[rustfmt::skip]
gen_crawl_event!(crawl_l3_event_spot, KuCoinSpotWSClient, MessageType::L3Event, subscribe_l3_orderbook);
#[rustfmt::skip]
gen_crawl_event!(crawl_l3_event_swap, KuCoinSwapWSClient, MessageType::L3Event, subscribe_l3_orderbook);

pub(crate) fn crawl_trade(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) -> Option<std::thread::JoinHandle<()>> {
    match market_type {
        MarketType::Spot => crawl_trade_spot(market_type, symbols, on_msg, duration),
        MarketType::InverseSwap | MarketType::LinearSwap | MarketType::InverseFuture => {
            crawl_trade_swap(market_type, symbols, on_msg, duration)
        }
        _ => panic!("KuCoin does NOT have the {} market type", market_type),
    }
}

pub(crate) fn crawl_l2_event(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) -> Option<std::thread::JoinHandle<()>> {
    match market_type {
        MarketType::Spot => crawl_l2_event_spot(market_type, symbols, on_msg, duration),
        MarketType::InverseSwap | MarketType::LinearSwap | MarketType::InverseFuture => {
            crawl_l2_event_swap(market_type, symbols, on_msg, duration)
        }
        _ => panic!("KuCoin does NOT have the {} market type", market_type),
    }
}

pub(crate) fn crawl_l3_event(
    market_type: MarketType,
    symbols: Option<&[String]>,
    on_msg: Arc<Mutex<dyn FnMut(Message) + 'static + Send>>,
    duration: Option<u64>,
) -> Option<std::thread::JoinHandle<()>> {
    match market_type {
        MarketType::Spot => crawl_l3_event_spot(market_type, symbols, on_msg, duration),
        MarketType::InverseSwap | MarketType::LinearSwap | MarketType::InverseFuture => {
            crawl_l3_event_swap(market_type, symbols, on_msg, duration)
        }
        _ => panic!("KuCoin does NOT have the {} market type", market_type),
    }
}
