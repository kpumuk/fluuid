pub fn new_v6(ts: ::uuid::v1::Timestamp, node_id: &[u8]) -> Result<::uuid::Uuid, String> {
    const NODE_ID_LEN: usize = 6;

    let len = node_id.len();
    if len != NODE_ID_LEN {
        Err(format!(
            "Expected node to be {} bytes, got {}",
            NODE_ID_LEN, len
        ))?;
    }

    let (ticks, counter) = ts.to_rfc4122();

    // shift up 4 bits, mask back in the relevant lower part and set the version
    let hi = ((ticks << 4) & 0xFFFFFFFFFFFF0000) | (ticks & 0x0FFF) | 0x6000;

    // 2 bit variant, 14 bits clock sequence, 48 bits node
    let lo = 0x8000 | (counter & 0x3fff);

    Ok(::uuid::Uuid::from_bytes([
        (hi >> 56) as u8,
        ((hi >> 48) & 0x0FF) as u8,
        ((hi >> 40) & 0x0FF) as u8,
        ((hi >> 32) & 0x0FF) as u8,
        ((hi >> 24) & 0x0FF) as u8,
        ((hi >> 16) & 0x0FF) as u8,
        ((hi >> 8) & 0x0FF) as u8,
        (hi & 0x0FF) as u8,
        ((lo >> 8) as u8),
        (lo & 0xFF) as u8,
        node_id[0],
        node_id[1],
        node_id[2],
        node_id[3],
        node_id[4],
        node_id[5],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_v6() {
        let time: u64 = 1_496_854_535;
        let time_fraction: u32 = 812_946_000;
        let node = [1, 2, 3, 4, 5, 6];
        let context = ::uuid::v1::Context::new(0);

        {
            let uuid = new_v6(
                ::uuid::v1::Timestamp::from_unix(&context, time, time_fraction),
                &node,
            )
            .unwrap();

            assert_eq!(uuid.as_bytes()[6] >> 4, 6);
            // assert_eq!(uuid.get_variant(), Some(uuid::Variant::NCS));
            assert_eq!(
                uuid.to_hyphenated().to_string(),
                "1e74ba22-0616-6934-8000-010203040506"
            );
        };

        {
            let uuid2 = new_v6(
                ::uuid::v1::Timestamp::from_unix(&context, time, time_fraction),
                &node,
            )
            .unwrap();

            assert_eq!(
                uuid2.to_hyphenated().to_string(),
                "1e74ba22-0616-6934-8001-010203040506"
            );
        };
    }
}
