use pinocchio::{
    cpi::{invoke_unchecked, CpiAccount},
    instruction::{InstructionAccount, InstructionView},
    AccountView,
};

use crate::cons::{
    humidifi::{ACCS_LEN_V2V3, ARGS_LEN, SWAP_V3_SELECTOR},
    OBF_CPI_KEY,
};

#[repr(C, packed)]
pub struct SwapArgs {
    pub swap_id: [u8; 8],
    pub amount_in: [u8; 8],
    pub direction: u8,
    pub padding: [u8; 7],
    pub selector: [u8; 1],
}

impl SwapArgs {
    pub fn new(amount_in: u64, a_to_b: bool) -> Self {
        Self {
            swap_id: [0u8; 8], // patched by caller from humidifi_param key
            amount_in: amount_in.to_le_bytes(),
            // is_base_to_quote: 0 when a_to_b (base_in), 1 when !a_to_b (quote_in)
            direction: if a_to_b { 0 } else { 1 },
            padding: [0u8; 7],
            selector: *SWAP_V3_SELECTOR,
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8; ARGS_LEN] {
        unsafe { &mut *(self as *mut Self as *mut [u8; ARGS_LEN]) }
    }
}

fn obfuscate(data: &mut [u8]) {
    let key = OBF_CPI_KEY;
    let mut pos = 0u64;
    for chunk in data.chunks_exact_mut(8) {
        let qword = u64::from_le_bytes(chunk.try_into().unwrap());
        let obfuscated = qword ^ key ^ (0x0001_0001_0001_0001u64).wrapping_mul(pos);
        chunk.copy_from_slice(&obfuscated.to_le_bytes());
        pos += 1;
    }
    let remainder_start = data.len() / 8 * 8;
    if remainder_start < data.len() {
        let pos_mask = (0x0001_0001_0001_0001u64).wrapping_mul(pos);
        let rem_len = data.len() - remainder_start;
        let mut rem = [0u8; 8];
        rem[..rem_len].copy_from_slice(&data[remainder_start..]);
        let qword = u64::from_le_bytes(rem);
        let obfuscated = qword ^ key ^ pos_mask;
        let ob_bytes = obfuscated.to_le_bytes();
        data[remainder_start..].copy_from_slice(&ob_bytes[..rem_len]);
    }
}

/// Expected remaining layout (15 accounts, NOT direction-dependent):
///   0  program          (readonly)
///   1  market           (writable)
///   2  market_ta_a      (writable)
///   3  market_ta_b      (writable)
///   4  user_ata_a       (writable)
///   5  user_ata_b       (writable)
///   6  clock            (readonly)
///   7  token_prog_a     (readonly)
///   8  token_prog_b     (readonly)
///   9  sysvar_ixs       (readonly)
///  10  mint_a           (readonly)
///  11  mint_b           (readonly)
///  12  rand1            (readonly)
///  13  jdf/placeholder  (readonly)
///  14  humidifi_param   (readonly) — swap_id encoded in first 8 bytes of key
///
/// CPI to Humidifi (15 accounts): payer injected at position 0.
pub fn swap_v1(payer: &AccountView, rem: &[AccountView], amount_in: u64, a_to_b: bool) {
    let mut args = SwapArgs::new(amount_in, a_to_b);

    // extract swap_id from the humidifi_param account key (first 8 bytes)
    args.swap_id.copy_from_slice(&rem[14].address().as_ref()[..8]);

    let data = args.as_bytes_mut();
    obfuscate(data);

    let ix_accs = [
        InstructionAccount::writable_signer(payer.address()), // payer
        InstructionAccount::writable(rem[1].address()),       // market
        InstructionAccount::writable(rem[2].address()),       // market_ta_a
        InstructionAccount::writable(rem[3].address()),       // market_ta_b
        InstructionAccount::writable(rem[4].address()),       // user_ata_a
        InstructionAccount::writable(rem[5].address()),       // user_ata_b
        InstructionAccount::readonly(rem[6].address()),       // clock
        InstructionAccount::readonly(rem[7].address()),       // token_prog_a
        InstructionAccount::readonly(rem[8].address()),       // token_prog_b
        InstructionAccount::readonly(rem[9].address()),       // sysvar_ixs
        InstructionAccount::readonly(rem[10].address()),      // mint_a
        InstructionAccount::readonly(rem[11].address()),      // mint_b
        InstructionAccount::readonly(rem[12].address()),      // rand1
        InstructionAccount::readonly(rem[13].address()),      // jdf/placeholder
    ];

    let ix = InstructionView { program_id: rem[0].address(), data: data.as_ref(), accounts: &ix_accs };

    let cpi_accs: [CpiAccount; ACCS_LEN_V2V3 - 1] = [
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
        CpiAccount::from(&rem[13]),
    ];

    unsafe { invoke_unchecked(&ix, &cpi_accs) };
}
