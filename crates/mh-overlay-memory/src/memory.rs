use std::io;

use libc::{c_void, iovec, pid_t};

use crate::error::MemoryError;

/// Read-only view of another process' address space.
///
/// Holds only the pid: `process_vm_readv` needs no open handle and the kernel
/// re-validates access on every call, so a process that exits mid-run surfaces as a
/// [`MemoryError::Read`] instead of a dangling reference.
#[derive(Debug, Clone, Copy)]
pub struct Memory {
    pid: pid_t,
}

impl Memory {
    pub fn new(pid: i32) -> Self {
        Self { pid }
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// Reads exactly `buffer.len()` bytes, failing if the kernel returns fewer.
    pub fn read_into(&self, address: u64, buffer: &mut [u8]) -> Result<(), MemoryError> {
        if buffer.is_empty() {
            return Ok(());
        }

        let local = iovec {
            iov_base: buffer.as_mut_ptr().cast::<c_void>(),
            iov_len: buffer.len(),
        };
        let remote = iovec {
            iov_base: address as *mut c_void,
            iov_len: buffer.len(),
        };

        // SAFETY: `local` describes a live, writable buffer owned by the caller for its whole
        // length. `remote` is never dereferenced in this process; the kernel only copies from
        // the target pid's address space, and a partial copy is reported as a short read.
        let read = unsafe { libc::process_vm_readv(self.pid, &local, 1, &remote, 1, 0) };

        if read < 0 {
            return Err(MemoryError::Read {
                address,
                length: buffer.len(),
                source: io::Error::last_os_error(),
            });
        }

        let read = read as usize;
        if read != buffer.len() {
            return Err(MemoryError::ShortRead {
                address,
                expected: buffer.len(),
                read,
            });
        }

        Ok(())
    }

    /// Fixed-size read whose length is known at compile time.
    pub fn read_bytes<const N: usize>(&self, address: u64) -> Result<[u8; N], MemoryError> {
        let mut buffer = [0u8; N];
        self.read_into(address, &mut buffer)?;
        Ok(buffer)
    }

    pub fn read_u8(&self, address: u64) -> Result<u8, MemoryError> {
        Ok(u8::from_le_bytes(self.read_bytes::<1>(address)?))
    }

    pub fn read_u16(&self, address: u64) -> Result<u16, MemoryError> {
        Ok(u16::from_le_bytes(self.read_bytes::<2>(address)?))
    }

    pub fn read_u32(&self, address: u64) -> Result<u32, MemoryError> {
        Ok(u32::from_le_bytes(self.read_bytes::<4>(address)?))
    }

    pub fn read_i32(&self, address: u64) -> Result<i32, MemoryError> {
        Ok(i32::from_le_bytes(self.read_bytes::<4>(address)?))
    }

    pub fn read_u64(&self, address: u64) -> Result<u64, MemoryError> {
        Ok(u64::from_le_bytes(self.read_bytes::<8>(address)?))
    }

    pub fn read_f32(&self, address: u64) -> Result<f32, MemoryError> {
        Ok(f32::from_le_bytes(self.read_bytes::<4>(address)?))
    }

    /// Reads `count` 64-bit pointers in one call: a 64-entry array costs one syscall instead of
    /// 64, and a hole anywhere in the range is an error like `read_u64` would be.
    pub fn read_pointers(&self, address: u64, count: usize) -> Result<Vec<u64>, MemoryError> {
        let bytes = self.read_vec(address, count * 8)?;

        Ok(bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|chunk| u64::from_le_bytes(*chunk))
            .collect())
    }

    /// Reads `count` floats in one call, with the same deal as [`Self::read_pointers`].
    pub fn read_f32_array(&self, address: u64, count: usize) -> Result<Vec<f32>, MemoryError> {
        let bytes = self.read_vec(address, count * 4)?;

        Ok(bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|chunk| f32::from_le_bytes(*chunk))
            .collect())
    }

    /// Reads a NUL-terminated string, decoding up to `max_length` bytes.
    pub fn read_cstring(&self, address: u64, max_length: usize) -> Result<String, MemoryError> {
        let bytes = self.read_vec(address, max_length)?;
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
    }

    pub fn read_vec(&self, address: u64, length: usize) -> Result<Vec<u8>, MemoryError> {
        let mut buffer = vec![0u8; length];
        self.read_into(address, &mut buffer)?;
        Ok(buffer)
    }

    /// Deref-then-offset chain: `addr = *(addr) + offset` per step (HunterPie's `ReadAsync`).
    ///
    /// The returned value is an address, not the contents stored there.
    pub fn deref_chain(&self, address: u64, offsets: &[i32]) -> Result<u64, MemoryError> {
        let mut current = address;

        for (step, offset) in offsets.iter().enumerate() {
            let pointer = self.read_u64(current)?;
            if pointer == 0 {
                return Err(MemoryError::NullPointer { step });
            }
            current = pointer.wrapping_add_signed(i64::from(*offset));
        }

        Ok(current)
    }

    /// Offset-then-deref chain: `addr = *(addr + offset)` per step (HunterPie's `ReadPtrAsync`).
    ///
    /// This is the shape most `Offset` entries in the `.map` files expect.
    pub fn follow_ptr_chain(&self, address: u64, offsets: &[i32]) -> Result<u64, MemoryError> {
        let mut current = address;

        for (step, offset) in offsets.iter().enumerate() {
            let slot = current.wrapping_add_signed(i64::from(*offset));
            let pointer = self.read_u64(slot)?;
            if pointer == 0 {
                return Err(MemoryError::NullPointer { step });
            }
            current = pointer;
        }

        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_own_memory_through_proc_works() {
        let memory = Memory::new(std::process::id() as i32);
        let value: u64 = 0x1234_5678_9ABC_DEF0;

        let read = memory.read_u64(&value as *const u64 as u64).unwrap();

        assert_eq!(read, value);
    }

    #[test]
    fn an_empty_buffer_is_not_a_read() {
        let memory = Memory::new(std::process::id() as i32);
        let mut buffer: [u8; 0] = [];

        assert!(memory.read_into(0, &mut buffer).is_ok());
    }

    #[test]
    fn a_chain_stops_at_the_first_null_pointer() {
        let memory = Memory::new(std::process::id() as i32);
        let null: u64 = 0;

        let error = memory
            .follow_ptr_chain(&null as *const u64 as u64, &[0])
            .unwrap_err();

        assert!(matches!(error, MemoryError::NullPointer { step: 0 }));
    }
}
