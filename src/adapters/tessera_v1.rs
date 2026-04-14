use pinocchio::{
    cpi::{invoke_unchecked, CpiAccount},
    instruction::{InstructionAccount, InstructionView},
    AccountView,
};

use crate::cons::tessera::{ACCS_LEN, ARGS_LEN, SWAP_SELECTOR};

/// Tessera V1 swap args: selector(1) + side(1) + amount_in(8) + min_out(8) = 18 bytes.
#[repr(C, packed)]
pub struct SwapArgs {
    pub selector: [u8; 1],
    pub side: u8,
    pub amount_in: [u8; 8],
    pub min_out: [u8; 8],
}

impl SwapArgs {
    pub fn new(amount_in: u64, a_to_b: bool) -> Self {
        Self { selector: *SWAP_SELECTOR, side: if a_to_b { 1 } else { 0 }, amount_in: amount_in.to_le_bytes(), min_out: 1u64.to_le_bytes() }
    }

    pub fn as_bytes(&self) -> &[u8; ARGS_LEN] {
        unsafe { &*(self as *const Self as *const [u8; ARGS_LEN]) }
    }
}

/// Remaining layout (12 accounts, user ATAs always in a/b order):
///   0  program          (readonly)
///   1  global_state     (readonly)
///   2  market           (writable)
///   3  vault_a          (writable)
///   4  vault_b          (writable)
///   5  user_ata_a       (writable)
///   6  user_ata_b       (writable)
///   7  mint_a           (readonly)
///   8  mint_b           (readonly)
///   9  token_prog_a     (readonly)
///  10  token_prog_b     (readonly)
///  11  sysvar_ixs       (readonly)
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, a_to_b: bool) {
    let args = SwapArgs::new(amount_in, a_to_b);

    let ix_accs = [
        InstructionAccount::readonly(rem[1].address()),       // global_state
        InstructionAccount::writable(rem[2].address()),       // market
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::writable(rem[3].address()),       // vault_a
        InstructionAccount::writable(rem[4].address()),       // vault_b
        InstructionAccount::writable(rem[5].address()),       // user_ata_a
        InstructionAccount::writable(rem[6].address()),       // user_ata_b
        InstructionAccount::readonly(rem[7].address()),       // mint_a
        InstructionAccount::readonly(rem[8].address()),       // mint_b
        InstructionAccount::readonly(rem[9].address()),       // token_prog_a
        InstructionAccount::readonly(rem[10].address()),      // token_prog_b
        InstructionAccount::readonly(rem[11].address()),      // sysvar_ixs
    ];

    let ix = InstructionView { program_id: rem[0].address(), data: args.as_bytes(), accounts: &ix_accs };

    let cpi: [CpiAccount; ACCS_LEN] = [
        CpiAccount::from(&rem[1]),
        CpiAccount::from(&rem[2]),
        CpiAccount::from(payer),
        CpiAccount::from(&rem[3]),
        CpiAccount::from(&rem[4]),
        CpiAccount::from(&rem[5]),
        CpiAccount::from(&rem[6]),
        CpiAccount::from(&rem[7]),
        CpiAccount::from(&rem[8]),
        CpiAccount::from(&rem[9]),
        CpiAccount::from(&rem[10]),
        CpiAccount::from(&rem[11]),
    ];

    unsafe { invoke_unchecked(&ix, &cpi) };
}
