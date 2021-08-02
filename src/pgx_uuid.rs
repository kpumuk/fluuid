use pgx::{pg_sys, FromDatum, IntoDatum};
use std::ops::{Deref, DerefMut};

#[derive(Debug, std::cmp::PartialEq)]
pub struct Uuid(uuid::Uuid);

impl IntoDatum for Uuid {
    #[inline]
    fn into_datum(self) -> Option<pg_sys::Datum> {
        let ptr = pgx::PgMemoryContexts::CurrentMemoryContext.palloc_slice::<u8>(16);
        ptr.clone_from_slice(self.0.as_bytes());

        Some(ptr.as_ptr() as pg_sys::Datum)
    }

    #[inline]
    fn type_oid() -> u32 {
        pg_sys::UUIDOID
    }
}

impl FromDatum for Uuid {
    #[inline]
    unsafe fn from_datum(datum: usize, is_null: bool, _typoid: pg_sys::Oid) -> Option<Uuid> {
        if is_null {
            None
        } else if datum == 0 {
            panic!("a uuid Datum as flagged as non-null but the datum is zero");
        } else {
            let bytes = std::slice::from_raw_parts(datum as *const u8, 16);
            let uuid = ::uuid::Uuid::from_slice(bytes).expect("Invalid UUID");
            Some(Uuid(uuid))
        }
    }
}

impl Uuid {
    pub fn new(uuid: uuid::Uuid) -> Self {
        Uuid(uuid)
    }
}

impl Deref for Uuid {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Uuid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(any(test, feature = "pg_test"))]
mod tests {
    use crate::*;
    use pgx::*;

    impl Uuid {
        pub fn parse_str(input: &str) -> Result<Uuid, uuid::Error> {
            let uuid = uuid::Uuid::parse_str(input)?;
            Result::Ok(Uuid(uuid))
        }
    }

    #[pg_test]
    fn test_from_datum() {
        let uuid = Spi::get_one::<Uuid>("SELECT '123e4567-e89b-12d3-a456-426614174000'::uuid;")
            .expect("SPI result was null");
        assert_eq!(
            uuid,
            Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap()
        )
    }

    #[pg_extern]
    fn return_uuid_v4() -> Uuid {
        Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap()
    }

    #[pg_test]
    fn test_into_datum() {
        let result = Spi::get_one::<bool>(
            "SELECT tests.return_uuid_v4() = '123e4567-e89b-12d3-a456-426614174000'::uuid;",
        )
        .expect("SPI result was null");
        assert!(result)
    }
}
