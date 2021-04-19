use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use cosmwasm_std::{from_slice, Addr, Coin, Order, StdResult, Storage, Uint128};
use cw0::Expiration;
use cw_storage_plus::{I64Key, Item, Map, PrimaryKey, U128Key};

use crate::msg::{Params, PlanContent};

/// (plan-id, subscriber-address)
pub type SubscriptionKey<'a> = (U128Key, &'a str);

/// Store contract params
pub const PARAMS: Item<Params> = Item::new("params");

/// Store the self-incremental unique ids for plans
pub const PLAN_ID: Item<Uint128> = Item::new("planid");
/// Store the plans, `plan-id -> Plan`
pub const PLANS: Map<U128Key, Plan> = Map::new("plans");
/// Store the subscriptions, `(plan-id, subscriber) -> Subscription`
pub const SUBSCRIPTIONS: Map<SubscriptionKey, Subscription> = Map::new("plan-subs");

// /// Subscription queue ordered by expiration time
// /// (expiration-time, subscription-id) -> ()
// pub const Q_EXPIRATION: Map<(i64, Uint128), ()> = Map::new("subs-expiration");
/// Subscription queue ordered by next_collection_time
/// (next-collection-time, plan-id, subscriber) -> ()
pub const Q_COLLECTION: Map<(I64Key, SubscriptionKey), ()> = Map::new("subs-collection");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uint128,
    pub owner: Addr,
    pub content: PlanContent,
    pub deposit: Vec<Coin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub expires: Expiration,
    pub last_collection_time: Option<i64>,
    pub next_collection_time: i64,
    pub deposit: Vec<Coin>,
}

pub fn gen_plan_id(store: &mut dyn Storage) -> StdResult<Uint128> {
    let mut plan_id = PLAN_ID.may_load(store)?.unwrap_or(0u64.into());
    plan_id = plan_id.wrapping_add(1u64.into());
    // ensure id not used
    while store
        .get(&PLANS.key(U128Key::from(plan_id.u128())))
        .is_some()
    {
        plan_id = plan_id.wrapping_add(1u64.into());
    }
    PLAN_ID.save(store, &plan_id)?;
    Ok(plan_id)
}

/// PANIC: if deserialization failed caused by corrupted storage
pub fn iter_subscriptions_by_plan<'a>(
    store: &'a dyn Storage,
    plan_id: Uint128,
) -> impl Iterator<Item = StdResult<(Addr, Subscription)>> + 'a {
    SUBSCRIPTIONS
        .prefix(plan_id.u128().into())
        .range(store, None, None, Order::Ascending)
        .map(|mpair| mpair.map(|(k, v)| (Addr::unchecked(String::from_utf8(k).unwrap()), v)))
}

/// PANIC: if deserialization failed caused by corrupted storage
pub fn iter_collectible_subscriptions<'a>(
    store: &'a dyn Storage,
    now: i64,
) -> impl Iterator<Item = (Uint128, Addr, Subscription)> + 'a {
    let maxkey = (
        I64Key::from(now.checked_add(1).unwrap()),
        (U128Key::from(0), ""),
    );
    store
        .range(
            None,
            Some(&PrimaryKey::joined_key(&maxkey)),
            Order::Ascending,
        )
        .map(|(k, v)| {
            let (end, s) = decode_key_step(&k).unwrap();
            let token_id = u128::from_be_bytes(s.try_into().unwrap()).into();
            let (_end, s) = decode_key_step(&k[end..]).unwrap();
            let addr = Addr::unchecked(String::from_utf8(s.to_owned()).unwrap());
            (token_id, addr, from_slice::<Subscription>(&v).unwrap())
        })
}

/// decode key, depends on the implemention details in cw-storage-plus
fn decode_key_step(buf: &[u8]) -> Option<(usize, &[u8])> {
    if buf.len() < 2 {
        return None;
    }
    let end = u16::from_be_bytes([buf[0], buf[1]]) as usize + 2;
    if buf.len() < end {
        return None;
    }
    Some((end, &buf[2..end]))
}
