// UNUSED FOR NOW

// TODO Make kernel read process memory for upcoming vanguard anticheat.
// TODO Modularize & minimize rpm calls for kernel.
mod driver_communication;
#[test]
fn test_offsets() {
    assert_eq!(2, 2);
}

pub fn extract_bytes_from_bytes_buffer<
    T: Sized,
>(
    read_buffer: &[u8],
    offset: usize,
) -> Result<Vec<u8>, String> {
    let size = std::mem::size_of::<T>();

    if offset + size > read_buffer.len()
    {
        return Err(
            "Buffer overflow".into()
        );
    }
    let data = &read_buffer
        [offset..offset + size];
    Ok(data.to_vec())
}

use external::{
    error::TAExternalError, read,
    read_buffer,
};
use std::os::raw::c_void;

pub fn read_memory<T>(
    process_handle: *mut c_void,
    base_address: usize,
    offsets: &Vec<usize>,
) -> Result<T, TAExternalError> {
    if offsets.is_empty() {
        return Err(TAExternalError::ReadMemoryFailed(
            external::error::ReadWriteMemoryFailedDetail::ErrorInvalidAddress,
        ));
    }
    let mut pointer_address =
        base_address;
    for offset_index in
        0..offsets.len() - 1
    {
        pointer_address =
            external::read::<usize>(
                process_handle,
                (pointer_address
                    as usize)
                    + offsets
                        [offset_index],
            )?;
    }
    let read_data = external::read::<T>(
        process_handle,
        pointer_address
            + offsets[offsets.len()],
    )?;
    return Ok(read_data);
}
