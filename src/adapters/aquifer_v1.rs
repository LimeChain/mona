use pinocchio::{
    cpi::{invoke_unchecked, CpiAccount},
    instruction::{InstructionAccount, InstructionView},
    AccountView,
};

use crate::cons::aquifer::{ACCS_LEN, ARGS_LEN, SWAP_SELECTOR};

/// Aquifer V1 swap args: selector(1) + amount_in(8) = 9 bytes.
#[repr(C, packed)]
pub struct SwapArgs {
    pub selector: [u8; 1],
    pub amount_in: [u8; 8],
}

impl SwapArgs {
    pub fn new(amount_in: u64) -> Self {
        Self { selector: *SWAP_SELECTOR, amount_in: amount_in.to_le_bytes() }
    }

    pub fn as_bytes(&self) -> &[u8; ARGS_LEN] {
        unsafe { &*(self as *const Self as *const [u8; ARGS_LEN]) }
    }
}

/// Remaining layout (16 accounts, direction-dependent ordering resolved off-chain):
///   0  program       (readonly)
///   1  sysvar_ixs    (readonly)
///   2  out_prog      (readonly)
///   3  user_dst_ata  (writable)
///   4  out_mint      (readonly)
///   5  in_prog       (readonly)
///   6  user_src_ata  (writable)
///   7  in_mint       (readonly)
///   8  rand1         (readonly)
///   9  rand2         (writable)
///  10  p10           (readonly)  — oracle (direction-dependent)
///  11  p11           (readonly)  — oracle (direction-dependent)
///  12  p12           (writable)  — market acc (direction-dependent)
///  13  p13           (writable)
///  14  p14           (writable)
///  15  p15           (writable)
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, _a_to_b: bool) {
    let args = SwapArgs::new(amount_in);

    let ix_accs = [
        InstructionAccount::readonly(rem[1].address()),       // sysvar_ixs
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::readonly(rem[2].address()),       // out_prog
        InstructionAccount::writable(rem[3].address()),       // user_dst_ata
        InstructionAccount::readonly(rem[4].address()),       // out_mint
        InstructionAccount::readonly(rem[5].address()),       // in_prog
        InstructionAccount::writable(rem[6].address()),       // user_src_ata
        InstructionAccount::readonly(rem[7].address()),       // in_mint
        InstructionAccount::readonly(rem[8].address()),       // rand1
        InstructionAccount::writable(rem[9].address()),       // rand2
        InstructionAccount::readonly(rem[10].address()),      // p10
        InstructionAccount::readonly(rem[11].address()),      // p11
        InstructionAccount::writable(rem[12].address()),      // p12
        InstructionAccount::writable(rem[13].address()),      // p13
        InstructionAccount::writable(rem[14].address()),      // p14
        InstructionAccount::writable(rem[15].address()),      // p15
    ];

    let ix = InstructionView { program_id: rem[0].address(), data: args.as_bytes(), accounts: &ix_accs };

    let cpi: [CpiAccount; ACCS_LEN] = [
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
    ];

    unsafe { invoke_unchecked(&ix, &cpi) };
}
