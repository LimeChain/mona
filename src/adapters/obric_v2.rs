use pinocchio::{
    cpi::{invoke_unchecked, CpiAccount},
    instruction::{InstructionAccount, InstructionView},
    AccountView,
};

use crate::cons::obric::{ACCS_LEN, ARGS_LEN, SWAP_SELECTOR};

/// Obric V2 swap args: selector(8) + x_to_y(1) + amount_in(8) + min_out(8) = 25 bytes.
#[repr(C, packed)]
pub struct SwapArgs {
    pub selector: [u8; 8],
    pub x_to_y: u8,
    pub amount_in: [u8; 8],
    pub min_out: [u8; 8],
}

impl SwapArgs {
    pub fn new(amount_in: u64, a_to_b: bool) -> Self {
        Self { selector: *SWAP_SELECTOR, x_to_y: a_to_b as u8, amount_in: amount_in.to_le_bytes(), min_out: 1u64.to_le_bytes() }
    }

    pub fn as_bytes(&self) -> &[u8; ARGS_LEN] {
        unsafe { &*(self as *const Self as *const [u8; ARGS_LEN]) }
    }
}

/// rem layout (12 accounts, user ATAs always in x/y order):
///   0  program          (readonly)
///   1  market           (writable)
///   2  second_ref_oracle(readonly)
///   3  third_ref_oracle (readonly)
///   4  reserve_x        (writable)
///   5  reserve_y        (writable)
///   6  user_ta_x        (writable)
///   7  user_ta_y        (writable)
///   8  ref_oracle       (writable)
///   9  x_price_feed     (readonly)
///  10  y_price_feed     (readonly)
///  11  token_prog       (readonly)
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, a_to_b: bool) {
    let args = SwapArgs::new(amount_in, a_to_b);

    let ix_accs = [
        InstructionAccount::writable(rem[1].address()),       // market
        InstructionAccount::readonly(rem[2].address()),       // second_ref_oracle
        InstructionAccount::readonly(rem[3].address()),       // third_ref_oracle
        InstructionAccount::writable(rem[4].address()),       // reserve_x
        InstructionAccount::writable(rem[5].address()),       // reserve_y
        InstructionAccount::writable(rem[6].address()),       // user_ta_x
        InstructionAccount::writable(rem[7].address()),       // user_ta_y
        InstructionAccount::writable(rem[8].address()),       // ref_oracle
        InstructionAccount::readonly(rem[9].address()),       // x_price_feed
        InstructionAccount::readonly(rem[10].address()),      // y_price_feed
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::readonly(rem[11].address()),      // token_prog
    ];

    let ix = InstructionView { program_id: rem[0].address(), data: args.as_bytes(), accounts: &ix_accs };

    let cpi: [CpiAccount; ACCS_LEN] = [
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
        CpiAccount::from(payer),
        CpiAccount::from(&rem[11]),
    ];

    unsafe { invoke_unchecked(&ix, &cpi) };
}
