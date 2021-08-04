mod uuid_v6;

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
    Uuid::from_bytes(
        *::uuid::Uuid::new_v1(gen_timestamp(), NODE_ID.share().0.as_ref())
            .unwrap()
            .as_bytes(),
    )
}

#[pg_extern]
fn gen_uuid_v1_random_node() -> Uuid {
    Uuid::from_bytes(
        *::uuid::Uuid::new_v1(gen_timestamp(), &gen_random_node_id())
            .unwrap()
            .as_bytes(),
    )
}

#[pg_extern]
fn gen_uuid_v4() -> Uuid {
    Uuid::from_bytes(*::uuid::Uuid::new_v4().as_bytes())
}

#[pg_extern]
fn gen_uuid_v6() -> Uuid {
    Uuid::from_bytes(
        *uuid_v6::new_v6(gen_timestamp(), NODE_ID.share().0.as_ref())
            .unwrap()
            .as_bytes(),
    )
}

#[pg_extern]
fn gen_uuid_v6_random_node() -> Uuid {
    Uuid::from_bytes(
        *uuid_v6::new_v6(gen_timestamp(), &gen_random_node_id())
            .unwrap()
            .as_bytes(),
    )
}

#[pg_extern]
fn gen_uuid_nil() -> Uuid {
    Uuid::from_bytes(*::uuid::Uuid::nil().as_bytes())
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
    const MAX_DIFF_NANOS: u64 = 1_000_000_000; // 1 second

    use crate::*;

    macro_rules! assert_ts {
        ($expression:expr) => {
            if let Some(ts) = $expression {
                let cur_ts = super::gen_timestamp();
                assert!(
                    cur_ts.to_unix_nanos() - ts.to_unix_nanos() <= MAX_DIFF_NANOS,
                    "UUID timestamp should be within 1 second from now, but the actual difference is {}ns",
                    cur_ts.to_unix_nanos() - ts.to_unix_nanos(),
                );
                assert_eq!(
                    cur_ts.to_rfc4122().1 - ts.to_rfc4122().1,
                    1,
                    "clock sequence should increase by 1"
                )
            } else {
                panic!("Expected timestamp, got None");
            }
        };
    }

    #[pg_test]
    fn test_gen_uuid_v1() {
        let res = Spi::get_one::<Uuid>("SELECT gen_uuid_v1();").expect("SPI result was null");
        let uuid = ::uuid::Uuid::from_bytes(*res.as_bytes());

        assert_eq!(uuid.get_version_num(), 1, "returns UUID v1");
        assert_eq!(uuid.get_variant(), Some(::uuid::Variant::RFC4122));
        assert_ts!(uuid.to_timestamp());

        assert_eq!(
            &uuid.as_bytes()[10..16],
            NODE_ID.share().0.as_ref(),
            "node ID should be shared for the process",
        );
    }

    #[pg_test]
    fn test_gen_uuid_v1_random_node() {
        let res =
            Spi::get_one::<Uuid>("SELECT gen_uuid_v1_random_node();").expect("SPI result was null");
        let uuid = ::uuid::Uuid::from_bytes(*res.as_bytes());

        assert_eq!(uuid.get_version_num(), 1, "returns UUID v1");
        assert_eq!(uuid.get_variant(), Some(::uuid::Variant::RFC4122));
        assert_ts!(uuid.to_timestamp());

        assert_ne!(
            &uuid.as_bytes()[10..16],
            NODE_ID.share().0.as_ref(),
            "node ID should be random",
        );
    }

    #[pg_test]
    fn test_gen_uuid_v4() {
        let res = Spi::get_one::<Uuid>("SELECT gen_uuid_v4();").expect("SPI result was null");
        let uuid = ::uuid::Uuid::from_bytes(*res.as_bytes());

        assert_eq!(uuid.get_version_num(), 4, "returns UUID v4");
        assert_eq!(uuid.get_variant(), Some(::uuid::Variant::RFC4122));

        assert_ne!(
            &uuid.as_bytes()[10..16],
            NODE_ID.share().0.as_ref(),
            "node ID should be shared for the process",
        );
    }

    #[pg_test]
    fn test_gen_uuid_nil() {
        let res = Spi::get_one::<Uuid>("SELECT gen_uuid_nil();").expect("SPI result was null");
        let uuid = ::uuid::Uuid::from_bytes(*res.as_bytes());

        assert_eq!(uuid, ::uuid::Uuid::nil());
    }

    #[pg_test]
    fn test_gen_uuid_v6() {
        let res = Spi::get_one::<Uuid>("SELECT gen_uuid_v6();").expect("SPI result was null");
        let uuid = ::uuid::Uuid::from_bytes(*res.as_bytes());

        assert_eq!(uuid.get_version_num(), 6, "returns UUID v6");
        assert_eq!(uuid.get_variant(), Some(::uuid::Variant::RFC4122));
        // assert_ts!(uuid.to_timestamp());

        assert_eq!(
            &uuid.as_bytes()[10..16],
            NODE_ID.share().0.as_ref(),
            "node ID should be shared for the process",
        );
    }

    #[pg_test]
    fn test_gen_uuid_v6_random_node() {
        let res =
            Spi::get_one::<Uuid>("SELECT gen_uuid_v6_random_node();").expect("SPI result was null");
        let uuid = ::uuid::Uuid::from_bytes(*res.as_bytes());

        assert_eq!(uuid.get_version_num(), 6, "returns UUID v6");
        assert_eq!(uuid.get_variant(), Some(::uuid::Variant::RFC4122));
        // assert_ts!(uuid.to_timestamp());

        assert_ne!(
            &uuid.as_bytes()[10..16],
            NODE_ID.share().0.as_ref(),
            "node ID should be random",
        );
    }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec!["shared_preload_libraries = 'fluuid'"]
    }
}
