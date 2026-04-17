use five8_const::decode_32_const as d;

pub const MAX_HOPS: usize = 30;

const OBF_CPI_KEY_SEED: [u8; 32] = [
    58, 255, 47, 255, 226, 186, 235, 195, 123, 131, 245, 8, 11, 233, 132, 219, 225, 40, 79, 119,
    169, 121, 169, 58, 197, 1, 122, 9, 216, 164, 149, 97,
];
pub const OBF_CPI_KEY: u64 = u64::from_le_bytes([
    OBF_CPI_KEY_SEED[0],
    OBF_CPI_KEY_SEED[1],
    OBF_CPI_KEY_SEED[2],
    OBF_CPI_KEY_SEED[3],
    OBF_CPI_KEY_SEED[4],
    OBF_CPI_KEY_SEED[5],
    OBF_CPI_KEY_SEED[6],
    OBF_CPI_KEY_SEED[7],
]);

pub mod solfi {
    pub const ID: [u8; 32] = super::d("SV2EYYJyRz2YhfXwXnhNAevDEui5Q6yrfyo13WtupPF");
    pub const SWAP_SELECTOR: &[u8; 1] = &[0x07];
    pub const ACCS_LEN: usize = 13;
    pub const ARGS_LEN: usize = 18;
}

pub mod obric {
    pub const ID: [u8; 32] = super::d("obriQD1zbpyLz95G5n7nJe6a4DPjpFwa5XYPoNm113y");
    pub const SWAP_SELECTOR: &[u8; 8] = &[0x41, 0x4b, 0x3f, 0x4c, 0xeb, 0x5b, 0x5b, 0x88];
    pub const ACCS_LEN: usize = 12;
    pub const ARGS_LEN: usize = 25;
}

pub mod humidifi {
    pub const ID: [u8; 32] = super::d("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp");
    pub const SWAP_V3_SELECTOR: &[u8; 1] = &[0x0f];
    pub const ACCS_LEN: usize = 11;
    pub const ARGS_LEN: usize = 25;

    pub const ACCS_LEN_V2V3: usize = 15;
    pub const ARGS_LEN_V2V3: usize = 25;
}

pub mod zerofi {
    pub const ID: [u8; 32] = super::d("ZERor4xhbUycZ6gb9ntrhqscUcZmAbQDjEAtCf4hbZY");
    pub const SWAP_SELECTOR: &[u8; 1] = &[0x06];
    pub const ACCS_LEN: usize = 10;
    pub const ARGS_LEN: usize = 17;
}

pub mod tessera {
    pub const ID: [u8; 32] = super::d("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH");
    pub const SWAP_SELECTOR: &[u8; 1] = &[0x10];
    pub const ACCS_LEN: usize = 12;
    pub const ARGS_LEN: usize = 18;
}

pub mod bisonfi {
    pub const ID: [u8; 32] = super::d("BiSoNHVpsVZW2F7rx2eQ59yQwKxzU5NvBcmKshCSUypi");
    pub const SWAP_SELECTOR: &[u8; 1] = &[0x02];
    pub const ACCS_LEN: usize = 9;
    pub const ARGS_LEN: usize = 18;
}

pub mod aquifer {
    pub const ID: [u8; 32] = super::d("AQU1FRd7papthgdrwPTTq5JacJh8YtwEXaBfKU3bTz45");
    pub const SWAP_SELECTOR: &[u8; 1] = &[0x01];
    pub const ACCS_LEN: usize = 16;
    pub const ARGS_LEN: usize = 9;
}

pub mod alphaq {
    pub const ID: [u8; 32] = super::d("ALPHAQmeA7bjrVuccPsYPiCvsi428SNwte66Srvs4pHA");
    pub const SWAP_SELECTOR: &[u8; 1] = &[0x0c];
    pub const ACCS_LEN: usize = 12;
    pub const ARGS_LEN: usize = 18;
}

pub mod scorch {
    pub const ID: [u8; 32] = super::d("SCoRcH8c2dpjvcJD6FiPbCSQyQgu3PcUAWj2Xxx3mqn");
    pub const SWAP_SELECTOR: &[u8; 1] = &[0x02];
    pub const ACCS_LEN: usize = 18;
    pub const ARGS_LEN: usize = 34;
}

/// DEX discriminant;
/// each variant maps to a specific adapter.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Dex {
    AlphaQ = 0,
    Aquifer = 1,
    BisonFi = 2,
    HumidiFiV2 = 3,
    HumidiFiV3 = 4,
    Obric = 5,
    Scorch = 6,
    SolFi = 7,
    Tessera = 8,
    ZeroFi = 9,
}

impl Dex {
    pub const ALL: [Dex; 10] = [
        Dex::AlphaQ,
        Dex::Aquifer,
        Dex::BisonFi,
        Dex::HumidiFiV2,
        Dex::HumidiFiV3,
        Dex::Obric,
        Dex::Scorch,
        Dex::SolFi,
        Dex::Tessera,
        Dex::ZeroFi,
    ];

    /// Number of remaining accounts per hop for swap_v1 (excludes shared payer).
    #[inline(always)]
    pub fn rem_accs_len_v1(&self) -> usize {
        REM_ACCS_LEN_V1[*self as usize]
    }

    /// Destination token account offset within the hop's remaining accounts.
    #[rustfmt::skip]
    pub fn dst_ta_offset(&self, a_to_b: bool) -> usize {
        match self {
            Dex::AlphaQ => if a_to_b { 4 } else { 3 },
            Dex::Aquifer => 3,
            Dex::BisonFi => if a_to_b { 5 } else { 4 },
            Dex::HumidiFiV2 | Dex::HumidiFiV3 => if a_to_b { 5 } else { 4 },
            Dex::Obric => if a_to_b { 7 } else { 6 },
            Dex::Scorch => 3,
            Dex::SolFi => if a_to_b { 7 } else { 6 },
            Dex::Tessera => if a_to_b { 6 } else { 5 },
            Dex::ZeroFi => 7,
        }
    }

    /// Map byte to Dex variant.
    #[inline(always)]
    pub fn from_u8(v: u8) -> Option<Self> {
        if v <= 9 {
            Some(Self::ALL[v as usize])
        } else {
            None
        }
    }
}

const REM_ACCS_LEN_V1: [usize; 10] = [
    alphaq::ACCS_LEN,        // 0  Alphaq
    aquifer::ACCS_LEN,       // 1  Aquifer
    bisonfi::ACCS_LEN,       // 2  Bisonfi
    humidifi::ACCS_LEN_V2V3, // 3  HumidifiV2
    humidifi::ACCS_LEN_V2V3, // 4  HumidifiV3
    obric::ACCS_LEN,         // 5  Obric
    scorch::ACCS_LEN,        // 6  Scorch
    solfi::ACCS_LEN,         // 7  Solfi
    tessera::ACCS_LEN,       // 8  Tessera
    zerofi::ACCS_LEN,        // 9  Zerofi
];
