//! Internal helper functions for data serialization.

/// Deserializes a byte array into a list of byte slices.
///
/// This is an internal helper function used by the SDK to deserialize data
/// returned from the FastEdge runtime.
///
/// # Format
///
/// The serialized format consists of:
/// - 4 bytes: number of items (little-endian u32)
/// - For each item: 4 bytes for size (little-endian u32)
/// - Followed by the actual data for all items
///
/// # Arguments
///
/// * `bytes` - The serialized byte array
///
/// # Returns
///
/// A vector of byte slices, each representing one item from the serialized list.
pub(crate) fn deserialize_list(bytes: &[u8]) -> Vec<&[u8]> {
    let mut list = Vec::new();
    if bytes.len() < 4 {
        return list;
    }
    let size = u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[0..4]).unwrap()) as usize;
    let mut p = 4 + size * 4;
    for n in 0..size {
        let s = 4 + n * 4;
        assert!(bytes.len() > (s + 4));
        let size = u32::from_le_bytes(<[u8; 4]>::try_from(&bytes[s..s + 4]).unwrap()) as usize;
        assert!(bytes.len() > (p + size));
        let value = &bytes[p..p + size];
        p += size + 1;
        list.push(value);
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialize_list(list: Vec<&[u8]>) -> Vec<u8> {
        let size = list.iter().fold(4, |size, v| size + v.len() + 5);

        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&(list.len() as i32).to_le_bytes());

        for value in &list {
            bytes.extend_from_slice(&(value.len() as i32).to_le_bytes());
        }

        for value in list {
            bytes.extend(value);
            bytes.push(0);
        }
        bytes
    }

    #[test]
    fn test_serialize_list() {
        let list = vec![
            b"hello".to_vec(),
            b"world".to_vec(),
            b"8734258932949023402343242312382183912390213932134".to_vec(),
            b"X".to_vec(),
        ];
        let serialized = serialize_list(list.iter().map(|v| v.as_slice()).collect());
        let deserialized = deserialize_list(&serialized);
        assert_eq!(4, deserialized.len());
        assert_eq!(list, deserialized);
    }
}
