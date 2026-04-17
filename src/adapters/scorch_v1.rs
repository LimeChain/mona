use crate::cons::scorch::{ACCS_LEN, ARGS_LEN, SWAP_SELECTOR};
use pinocchio::cpi::{invoke_unchecked, CpiAccount};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::AccountView;

// Scorch V1 swap args: selector(1) + param(17) + amount_in(8) + min_out(8) = 34 bytes.
#[repr(C, packed)]
pub struct SwapArgs {
    pub selector: [u8; 1],
    pub scorch_param: [u8; 17],
    pub amount_in: [u8; 8],
    pub min_out: [u8; 8],
}

impl SwapArgs {
    pub fn new(amount_in: u64) -> Self {
        Self {
            selector: *SWAP_SELECTOR,
            scorch_param: [0u8; 17], // patched by caller from scorch_param key
            amount_in: amount_in.to_le_bytes(),
            min_out: 1u64.to_le_bytes(),
        }
    }

    pub fn as_bytes(&self) -> &[u8; ARGS_LEN] {
        unsafe { &*(self as *const Self as *const [u8; ARGS_LEN]) }
    }
}

/// Remaining layout (18 accounts):
///    0  program          (readonly)
///    1  market           (readonly)
///    2  user_ata_a       (writable)
///    3  user_ata_b       (writable)
///    4  market_ta_a      (writable)
///    5  market_ta_b      (writable)
///    6  mint_a           (readonly)
///    7  mint_b           (readonly)
///    8  token_prog       (readonly)
///    9  token_prog       (readonly)
///   10  memo_prog        (readonly)
///   11  core_prog        (readonly)
///   12  acc1             (readonly)
///   13  state_a          (writable)
///   14  state_b          (writable)
///   15  state_c          (writable)
///   16  sysvar_ixs       (readonly)
///   17  scorch_param     (readonly) - encoded in first 16 bytes of key
///
///   CPI to Scorch (18 accounts): payer injected at position 1.
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, _a_to_b: bool) {
    let mut args = SwapArgs::new(amount_in);

    // extract params from the scorch_param account key (first 17 bytes)
    args.scorch_param
        .copy_from_slice(&rem[17].address().as_ref()[..17]);

    let ix_accs = [
        InstructionAccount::readonly(rem[1].address()), // market
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::writable(rem[2].address()), // user_ata_a
        InstructionAccount::writable(rem[3].address()), // user_ata_b
        InstructionAccount::writable(rem[4].address()), // market_ta_a
        InstructionAccount::writable(rem[5].address()), // market_ta_b
        InstructionAccount::readonly(rem[6].address()), // mint_a
        InstructionAccount::readonly(rem[7].address()), // mint_b
        InstructionAccount::readonly(rem[8].address()), // token_prog
        InstructionAccount::readonly(rem[9].address()), // token_prog
        InstructionAccount::readonly(rem[10].address()), // memo_prog
        InstructionAccount::readonly(rem[11].address()), // core_prog
        InstructionAccount::readonly(rem[12].address()), // acc1
        InstructionAccount::writable(rem[13].address()), // state_a
        InstructionAccount::writable(rem[14].address()), // state_b
        InstructionAccount::writable(rem[15].address()), // state_c
        InstructionAccount::readonly(rem[16].address()), // sysvar_ixs
    ];

    let ix = InstructionView {
        program_id: rem[0].address(),
        data: args.as_bytes(),
        accounts: &ix_accs,
    };

    let cpi: [CpiAccount; ACCS_LEN - 1] = [
        CpiAccount::from(&rem[1]),
        CpiAccount::from(payer),
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
        CpiAccount::from(&rem[13]),
        CpiAccount::from(&rem[14]),
        CpiAccount::from(&rem[15]),
        CpiAccount::from(&rem[16]),
    ];

    unsafe { invoke_unchecked(&ix, &cpi) }
}
