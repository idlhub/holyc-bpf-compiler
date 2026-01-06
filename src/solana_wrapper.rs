/// Solana program wrapper for HolyC-compiled code
///
/// This module provides the runtime integration between HolyC-compiled
/// BPF bytecode and the Solana program interface.

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Solana entrypoint macro generates the required program entry function
entrypoint!(process_instruction);

/// Main entry point for the Solana program
///
/// This wrapper receives Solana's standard parameters and delegates
/// to the HolyC-compiled BPF code.
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("HolyC-BPF Program: Processing instruction");
    msg!("Program ID: {}", program_id);
    msg!("Accounts: {}", accounts.len());
    msg!("Instruction data: {} bytes", instruction_data.len());

    // Call the HolyC-compiled entrypoint
    // The HolyC function signature should match:
    // U64 entrypoint(CAccountInfo* accounts, U64 account_count, U8* data, U64 data_len)

    let result = unsafe {
        holyc_entrypoint(
            accounts.as_ptr() as *const u8,
            accounts.len() as u64,
            instruction_data.as_ptr(),
            instruction_data.len() as u64,
        )
    };

    if result == 0 {
        msg!("HolyC-BPF Program: Success");
        Ok(())
    } else {
        msg!("HolyC-BPF Program: Error code 0x{:x}", result);
        Err(ProgramError::Custom(result as u32))
    }
}

/// External function that will be provided by the HolyC-compiled BPF code
extern "C" {
    fn holyc_entrypoint(
        accounts: *const u8,
        account_count: u64,
        instruction_data: *const u8,
        data_len: u64,
    ) -> u64;
}

/// Helper functions for HolyC code to call Solana runtime functions
#[no_mangle]
pub extern "C" fn solana_log(message: *const u8, len: u64) {
    if message.is_null() || len == 0 {
        return;
    }

    unsafe {
        let slice = std::slice::from_raw_parts(message, len as usize);
        if let Ok(s) = std::str::from_utf8(slice) {
            msg!("{}", s);
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_read_u64_le(data: *const u8, offset: u64) -> u64 {
    if data.is_null() {
        return 0;
    }

    unsafe {
        let ptr = data.add(offset as usize) as *const u64;
        u64::from_le(*ptr)
    }
}

#[no_mangle]
pub extern "C" fn solana_write_u64_le(data: *mut u8, offset: u64, value: u64) {
    if data.is_null() {
        return;
    }

    unsafe {
        let ptr = data.add(offset as usize) as *mut u64;
        *ptr = value.to_le();
    }
}

#[no_mangle]
pub extern "C" fn solana_memcpy(dst: *mut u8, src: *const u8, len: u64) {
    if dst.is_null() || src.is_null() || len == 0 {
        return;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(src, dst, len as usize);
    }
}

#[no_mangle]
pub extern "C" fn solana_memset(dst: *mut u8, value: u8, len: u64) {
    if dst.is_null() || len == 0 {
        return;
    }

    unsafe {
        std::ptr::write_bytes(dst, value, len as usize);
    }
}

/// Account Info structure compatible with HolyC CAccountInfo
#[repr(C)]
pub struct CAccountInfo {
    pub key: [u8; 32],         // Pubkey (32 bytes)
    pub lamports: u64,
    pub data_len: u64,
    pub data: *mut u8,
    pub owner: [u8; 32],       // Pubkey (32 bytes)
    pub is_signer: u8,         // Bool as u8
    pub is_writable: u8,       // Bool as u8
}

impl CAccountInfo {
    /// Convert Solana AccountInfo to HolyC CAccountInfo
    pub fn from_account_info(account: &AccountInfo) -> Self {
        Self {
            key: account.key.to_bytes(),
            lamports: **account.lamports.borrow(),
            data_len: account.data_len() as u64,
            data: account.data.borrow_mut().as_mut_ptr(),
            owner: account.owner.to_bytes(),
            is_signer: account.is_signer as u8,
            is_writable: account.is_writable as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_account_info_size() {
        assert_eq!(std::mem::size_of::<CAccountInfo>(), 104);
    }

    #[test]
    fn test_read_write_u64() {
        let mut buffer = [0u8; 16];
        let ptr = buffer.as_mut_ptr();

        solana_write_u64_le(ptr, 0, 0xdeadbeefcafebabe);
        let value = solana_read_u64_le(ptr, 0);

        assert_eq!(value, 0xdeadbeefcafebabe);
    }
}
