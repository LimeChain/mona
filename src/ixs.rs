#[cfg(feature = "v1")]
pub mod swap_v1;

use pinocchio::AccountView;

/// Read the SPL token amount (bytes 64..72) from a token account.
#[inline(always)]
pub fn token_bal(acc: &AccountView) -> u64 {
    let data = unsafe { acc.borrow_unchecked() };
    u64::from_le_bytes(data[64..72].try_into().unwrap())
}
