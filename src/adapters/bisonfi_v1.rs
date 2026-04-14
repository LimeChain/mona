use pinocchio::{
    cpi::{invoke_unchecked, CpiAccount},
    instruction::{InstructionAccount, InstructionView},
    AccountView,
};

use crate::cons::bisonfi::{ACCS_LEN, ARGS_LEN, SWAP_SELECTOR};

/// BisonFi V1 swap args: selector (1) + amount_in (8) + min_out (8) + b_to_a (1) = 18 bytes.
#[repr(C, packed)]
pub struct SwapArgs {
    pub selector: [u8; 1],
    pub amount_in: [u8; 8],
    pub min_out: [u8; 8],
    pub b_to_a: u8,
}

impl SwapArgs {
    pub fn new(amount_in: u64, a_to_b: bool) -> Self {
        Self { selector: *SWAP_SELECTOR, amount_in: amount_in.to_le_bytes(), min_out: 1u64.to_le_bytes(), b_to_a: if a_to_b { 0 } else { 1 } }
    }

    pub fn as_bytes(&self) -> &[u8; ARGS_LEN] {
        unsafe { &*(self as *const Self as *const [u8; ARGS_LEN]) }
    }
}

/// Expected remaining layout (9 accounts):
///   0  program_id      (readonly, executable)
///   1  market          (writable)
///   2  market_ta_a     (writable)
///   3  market_ta_b     (writable)
///   4  user_ata_a      (writable)
///   5  user_ata_b      (writable)
///   6  token_prog_a (readonly)
///   7  token_prog_b(readonly)
///   8  sysvar_ixs      (readonly)
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, a_to_b: bool) {
    let args = SwapArgs::new(amount_in, a_to_b);

    let ix_accs = [
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::writable(rem[1].address()),       // market
        InstructionAccount::writable(rem[2].address()),       // market_ta_a
        InstructionAccount::writable(rem[3].address()),       // market_ta_b
        InstructionAccount::writable(rem[4].address()),       // user_ata_a
        InstructionAccount::writable(rem[5].address()),       // user_ata_b
        InstructionAccount::readonly(rem[6].address()),       // token_prog_a
        InstructionAccount::readonly(rem[7].address()),       // token_prog_b
        InstructionAccount::readonly(rem[8].address()),       // sysvar_ixs
    ];

    let ix = InstructionView { program_id: rem[0].address(), data: args.as_bytes(), accounts: &ix_accs };

    let cpi: [CpiAccount; ACCS_LEN] = [
        CpiAccount::from(payer),
        CpiAccount::from(&rem[1]),
        CpiAccount::from(&rem[2]),
        CpiAccount::from(&rem[3]),
        CpiAccount::from(&rem[4]),
        CpiAccount::from(&rem[5]),
        CpiAccount::from(&rem[6]),
        CpiAccount::from(&rem[7]),
        CpiAccount::from(&rem[8]),
    ];

    unsafe { invoke_unchecked(&ix, &cpi) };
}
