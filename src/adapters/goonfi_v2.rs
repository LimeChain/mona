use pinocchio::{
    cpi::{invoke_unchecked, CpiAccount},
    instruction::{InstructionAccount, InstructionView},
    AccountView,
};

use crate::cons::goonfi_v2::{ACCS_LEN, ARGS_LEN, SWAP_SELECTOR};

/// GoonFi V2 swap args: selector(1) + is_user_bid(1) + bump(1) + amount_in(8) + min_out(8) = 19 bytes.
#[repr(C, packed)]
pub struct SwapArgs {
    pub selector: [u8; 1],
    pub is_user_bid: u8,
    pub bump: u8,
    pub amount_in: [u8; 8],
    pub min_out: [u8; 8],
}

impl SwapArgs {
    pub fn new(amount_in: u64, a_to_b: bool) -> Self {
        Self {
            selector: *SWAP_SELECTOR,
            is_user_bid: if a_to_b { 0 } else { 1 },
            bump: 0, // filled from goonfi_param key
            amount_in: amount_in.to_le_bytes(),
            min_out: 1u64.to_le_bytes(),
        }
    }

    pub fn as_bytes(&self) -> &[u8; ARGS_LEN] {
        unsafe { &*(self as *const Self as *const [u8; ARGS_LEN]) }
    }
}

/// Remaining layout (13 accounts, user ATAs always in a/b order):
///   0  program          (readonly)
///   1  market           (writable)
///   2  user_ata_a       (writable)
///   3  user_ata_b       (writable)
///   4  market_ta_a      (writable)
///   5  market_ta_b      (writable)
///   6  mint_a           (readonly)
///   7  mint_b           (readonly)
///   8  blacklist        (readonly)
///   9  goonfi_param     (readonly) — first byte of key encodes blacklist bump
///  10  sysvar_ixs       (readonly)
///  11  token_prog_a  (readonly)
///  12  token_prog_b (readonly)
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, a_to_b: bool) {
    let mut args = SwapArgs::new(amount_in, a_to_b);

    // extract bump from goonfi_param key (first byte)
    args.bump = rem[9].address().as_ref()[0];

    let ix_accs = [
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::writable(rem[1].address()),       // market
        InstructionAccount::writable(rem[2].address()),       // user_ata_a
        InstructionAccount::writable(rem[3].address()),       // user_ata_b
        InstructionAccount::writable(rem[4].address()),       // market_ta_a
        InstructionAccount::writable(rem[5].address()),       // market_ta_b
        InstructionAccount::readonly(rem[6].address()),       // mint_a
        InstructionAccount::readonly(rem[7].address()),       // mint_b
        InstructionAccount::readonly(rem[8].address()),       // blacklist
        InstructionAccount::readonly(rem[9].address()),       // goonfi_param
        InstructionAccount::readonly(rem[10].address()),      // sysvar_ixs
        InstructionAccount::readonly(rem[11].address()),      // token_prog_a
        InstructionAccount::readonly(rem[12].address()),      // token_prog_b
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
