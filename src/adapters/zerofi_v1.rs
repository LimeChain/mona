use pinocchio::{
    cpi::{invoke_unchecked, CpiAccount},
    instruction::{InstructionAccount, InstructionView},
    AccountView,
};

use crate::cons::zerofi::{ACCS_LEN, ARGS_LEN};

/// ZeroFi V1 swap args: selector(1) + amount_in(8) + min_out(8) = 17 bytes.
#[repr(C, packed)]
pub struct SwapArgs {
    pub selector: u8,
    pub amount_in: [u8; 8],
    pub min_out: [u8; 8],
}

impl SwapArgs {
    pub fn new(amount_in: u64) -> Self {
        Self { selector: 0x06, amount_in: amount_in.to_le_bytes(), min_out: 0u64.to_le_bytes() }
    }

    pub fn as_bytes(&self) -> &[u8; ARGS_LEN] {
        unsafe { &*(self as *const Self as *const [u8; ARGS_LEN]) }
    }
}

/// Remaining layout (10 accounts, direction-dependent — resolved off-chain):
///   0  program          (readonly)
///   1  market           (writable)
///   2  cfg_in           (writable)
///   3  ta_in            (writable)
///   4  cfg_out          (writable)
///   5  ta_out           (writable)
///   6  usr_ta_in        (writable)
///   7  usr_ta_out       (writable)
///   8  token_prog       (readonly)
///   9  sysvar_ixs       (readonly)
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, _a_to_b: bool) {
    let args = SwapArgs::new(amount_in);

    let ix_accs = [
        InstructionAccount::writable(rem[1].address()),       // market
        InstructionAccount::writable(rem[2].address()),       // cfg_in
        InstructionAccount::writable(rem[3].address()),       // ta_in
        InstructionAccount::writable(rem[4].address()),       // cfg_out
        InstructionAccount::writable(rem[5].address()),       // ta_out
        InstructionAccount::writable(rem[6].address()),       // usr_ta_in
        InstructionAccount::writable(rem[7].address()),       // usr_ta_out
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::readonly(rem[8].address()),       // token_prog
        InstructionAccount::readonly(rem[9].address()),       // sysvar_ixs
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
        CpiAccount::from(payer),
        CpiAccount::from(&rem[8]),
        CpiAccount::from(&rem[9]),
    ];

    unsafe { invoke_unchecked(&ix, &cpi) };
}
