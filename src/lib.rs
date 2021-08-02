mod pgx_uuid;
mod uuid_v6;

use crate::pgx_uuid::Uuid;
use pgx::*;
use std::sync::atomic;
use std::time::{SystemTime, UNIX_EPOCH};

pg_module_magic!();

#[derive(Copy, Clone)]
struct NodeID([u8; 6]);
unsafe impl pgx::PGXSharedMemory for NodeID {}

impl Default for NodeID {
    fn default() -> Self {
        NodeID(gen_random_node_id())
    }
}

static ATOMIC: PgAtomic<atomic::AtomicU16> = PgAtomic::new();
static NODE_ID: PgLwLock<NodeID> = PgLwLock::new();

struct Context;
impl ::uuid::v1::ClockSequence for Context {
    fn generate_sequence(&self, _: u64, _: u32) -> u16 {
        ATOMIC.get().fetch_add(1, atomic::Ordering::SeqCst)
    }
}

#[pg_guard]
pub extern "C" fn _PG_init() {
    pg_shmem_init!(ATOMIC);
    pg_shmem_init!(NODE_ID);
}

#[pg_extern]
fn gen_uuid_v1() -> Uuid {
    Uuid::new(::uuid::Uuid::new_v1(gen_timestamp(), NODE_ID.share().0.as_ref()).unwrap())
}

#[pg_extern]
fn gen_uuid_v1_random_node() -> Uuid {
    Uuid::new(::uuid::Uuid::new_v1(gen_timestamp(), &gen_random_node_id()).unwrap())
}

#[pg_extern]
fn gen_uuid_v4() -> Uuid {
    Uuid::new(::uuid::Uuid::new_v4())
}

#[pg_extern]
fn gen_uuid_v6() -> Uuid {
    Uuid::new(uuid_v6::new_v6(gen_timestamp(), NODE_ID.share().0.as_ref()).unwrap())
}

#[pg_extern]
fn gen_uuid_v6_random_node() -> Uuid {
    Uuid::new(uuid_v6::new_v6(gen_timestamp(), &gen_random_node_id()).unwrap())
}

#[pg_extern]
fn gen_uuid_nil() -> Uuid {
    Uuid::new(::uuid::Uuid::nil())
}

fn gen_timestamp() -> ::uuid::v1::Timestamp {
    let now = SystemTime::now();
    let dur = now.duration_since(UNIX_EPOCH).unwrap();
    ::uuid::v1::Timestamp::from_unix(Context {}, dur.as_secs(), dur.subsec_nanos())
}

fn gen_random_node_id() -> [u8; 6] {
    let mut node_id = [0u8; 6];
    getrandom::getrandom(&mut node_id)
        .unwrap_or_else(|err| panic!("could not retrieve random bytes for uuid: {}", err));
    node_id[0] |= 0x01;

    node_id
}

#[cfg(any(test, feature = "pg_test"))]
mod tests {
    // use crate::*;
    // use pgx::*;

    // #[pg_test]
    // fn test_hello_gen_random_uuid_nil() {
    //     assert_eq!(uuid::Uuid::nil(), crate::gen_random_uuid_nil().0);
    // }
    //
    // #[pg_extern]
    // fn return_uuid() -> Uuid {
    //     Uuid(uuid::Uuid::nil())
    // }

    // #[pg_test]
    // fn test_return_uuid() {
    //     let uuid =
    //         Spi::get_one::<Uuid>("SELECT tests.return_uuid();").expect("SPI result was null");
    //     assert_eq!(uuid, Uuid(uuid::Uuid::nil()))
    // }
    //
    // #[pg_test]
    // fn test_return_null_uuid() {
    //     let uuid = Spi::get_one::<Uuid>("SELECT NULL::uuid;");
    //     assert_eq!(uuid, None)
    // }
    //
    // #[pg_test]
    // fn test_return_uuid_v4() {
    //     let uuid =
    //         Spi::get_one::<Uuid>("SELECT gen_random_uuid_v4();").expect("SPI result was null");
    //     assert_eq!(uuid, Uuid(uuid::Uuid::nil()))
    // }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
