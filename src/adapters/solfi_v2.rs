use pinocchio::{
    cpi::{invoke_unchecked, CpiAccount},
    instruction::{InstructionAccount, InstructionView},
    AccountView,
};

use crate::cons::solfi::{ACCS_LEN, ARGS_LEN, SWAP_SELECTOR};

/// SolFi V2 swap args: selector (1) + amount_in (8) + min_out (8) + direction (1) = 18 bytes.
#[repr(C, packed)]
pub struct SwapArgs {
    pub selector: [u8; 1],
    pub amount_in: [u8; 8],
    pub min_out: [u8; 8],
    pub direction: u8,
}

impl SwapArgs {
    pub fn new(amount_in: u64, a_to_b: bool) -> Self {
        Self { selector: *SWAP_SELECTOR, amount_in: amount_in.to_le_bytes(), min_out: 1u64.to_le_bytes(), direction: if a_to_b { 0 } else { 1 } }
    }

    pub fn as_bytes(&self) -> &[u8; ARGS_LEN] {
        unsafe { &*(self as *const Self as *const [u8; ARGS_LEN]) }
    }
}

/// Expected remaining layout (13 accounts):
///   0  program_id      (readonly, executable)
///   1  market          (writable)
///   2  oracle          (readonly)
///   3  global_cfg      (readonly)
///   4  market_ta_a     (writable)
///   5  market_ta_b     (writable)
///   6  usr_ta_a        (writable)
///   7  usr_ta_b        (writable)
///   8  mint_a          (readonly)
///   9  mint_b          (readonly)
///  10  token_prog_a    (readonly)
///  11  token_prog_b    (readonly)
///  12  sysvar_ixs      (readonly)
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, a_to_b: bool) {
    let args = SwapArgs::new(amount_in, a_to_b);

    let ix_accs = [
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::writable(rem[1].address()),       // market
        InstructionAccount::readonly(rem[2].address()),       // oracle
        InstructionAccount::readonly(rem[3].address()),       // global_cfg
        InstructionAccount::writable(rem[4].address()),       // market_ta_a
        InstructionAccount::writable(rem[5].address()),       // market_ta_b
        InstructionAccount::writable(rem[6].address()),       // usr_ta_a
        InstructionAccount::writable(rem[7].address()),       // usr_ta_b
        InstructionAccount::readonly(rem[8].address()),       // mint_a
        InstructionAccount::readonly(rem[9].address()),       // mint_b
        InstructionAccount::readonly(rem[10].address()),      // token_prog_a
        InstructionAccount::readonly(rem[11].address()),      // token_prog_b
        InstructionAccount::readonly(rem[12].address()),      // sysvar_ixs
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
        CpiAccount::from(&rem[9]),
        CpiAccount::from(&rem[10]),
        CpiAccount::from(&rem[11]),
        CpiAccount::from(&rem[12]),
    ];

    unsafe { invoke_unchecked(&ix, &cpi) };
}
