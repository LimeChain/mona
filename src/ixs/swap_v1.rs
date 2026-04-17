/// SwapV1 — flat account layout, each adapter owns its full account slice.
///
/// Instruction data (after selector):
///
///   flags=0x01 (chained)                       flags=0x02 (split)
///   ─────────────────────────────────          ─────────────────────────────────
///   ┌──────────────────────┐                   ┌──────────────────────┐
///   │ flags           (1B) │                   │ flags           (1B) │
///   │ amount_in       (8B) │                   │ num_steps       (1B) │
///   │ amount_out_min  (8B) │                   ├──────────────────────┤
///   │ num_steps       (1B) │                   │ step[0].dex     (1B) │
///   ├──────────────────────┤                   │ step[0].a_to_b  (1B) │
///   │ step[0].dex     (1B) │                   │ step[0].amt_in  (8B) │
///   │ step[0].a_to_b  (1B) │                   │ step[0].out_min (8B) │
///   │ ...                  │                   │ ...                  │
///   └──────────────────────┘                   └──────────────────────┘
///   header: 18B, step: 2B each                 header: 2B, step: 18B each
///
///   accounts:
///   ┌──────────────────────────────────────────────┐
///   │ [0]  payer (signer, writable)                │
///   │ [1..N₀]  hop 0 remaining accounts            │
///   │ [N₀..N₁] hop 1 remaining accounts            │
///   │ ...                                          │
///   └──────────────────────────────────────────────┘
use pinocchio::{AccountView, Address, ProgramResult};

use crate::{
    adapters,
    cons::{Dex, MAX_HOPS},
    errs,
    ixs::token_bal,
};

#[repr(C)]
pub struct SwapV1Chained {
    pub amount_in: [u8; 8],
    pub amount_out_min: [u8; 8],
    pub num_steps: u8,
}

#[repr(C)]
pub struct RoutePlanStepV1 {
    pub swap: u8,
    pub a_to_b: u8,
}

#[repr(C)]
pub struct RoutePlanStepV1S {
    pub swap: u8,
    pub a_to_b: u8,
    pub amount_in: [u8; 8],
    pub amount_out_min: [u8; 8],
}

impl RoutePlanStepV1 {
    pub fn a_to_b(&self) -> bool {
        self.a_to_b != 0
    }
}

impl RoutePlanStepV1S {
    pub fn a_to_b(&self) -> bool {
        self.a_to_b != 0
    }

    pub fn amount_in(&self) -> u64 {
        u64::from_le_bytes(self.amount_in)
    }

    pub fn amount_out_min(&self) -> u64 {
        u64::from_le_bytes(self.amount_out_min)
    }
}

impl SwapV1Chained {
    pub const HDR_LEN: usize = core::mem::size_of::<Self>();

    pub fn amount_in(&self) -> u64 {
        u64::from_le_bytes(self.amount_in)
    }

    pub fn amount_out_min(&self) -> u64 {
        u64::from_le_bytes(self.amount_out_min)
    }

    pub fn route_plan(&self) -> &[RoutePlanStepV1] {
        let n = self.num_steps as usize;
        let ptr = unsafe { (self as *const Self).add(1) as *const RoutePlanStepV1 };
        unsafe { core::slice::from_raw_parts(ptr, n) }
    }
}

/// Dispatch a single swap_v1 CPI.
#[inline(always)]
fn dispatch(payer: &AccountView, rem: &[AccountView], amount_in: u64, a_to_b: bool, dex: Dex) {
    let handler: fn(&AccountView, &[AccountView], u64, bool) = match dex {
        Dex::AlphaQ => adapters::alphaq_v1::swap_v1,
        Dex::Aquifer => adapters::aquifer_v1::swap_v1,
        Dex::BisonFi => adapters::bisonfi_v1::swap_v1,
        Dex::HumidiFiV2 | Dex::HumidiFiV3 => adapters::humidifi_v1::swap_v3,
        Dex::Obric => adapters::obric_v2::swap_v1,
        Dex::Scorch => adapters::scorch_v1::swap_v1,
        Dex::SolFi => adapters::solfi_v2::swap_v1,
        Dex::Tessera => adapters::tessera_v1::swap_v1,
        Dex::ZeroFi => adapters::zerofi_v1::swap_v1,
    };

    handler(payer, rem, amount_in, a_to_b)
}

/// Resolve accounts, dispatch swap, return (new offset, output delta).
#[inline(always)]
fn exec_hop(
    payer: &AccountView,
    accs: &[AccountView],
    offset: usize,
    wire_dex: u8,
    a_to_b: bool,
    amount_in: u64,
    step_i: u8,
) -> Result<(usize, u64), pinocchio::error::ProgramError> {
    let dex = Dex::from_u8(wire_dex).ok_or(errs::unsupported_dex())?;
    let end = offset + dex.rem_accs_len_v1();
    if end > accs.len() {
        return Err(errs::not_enough_accs(step_i, dex as u8));
    }

    let rem = &accs[offset..end];
    let dst_ta_idx = offset + dex.dst_ta_offset(a_to_b);
    let dst_before = token_bal(&accs[dst_ta_idx]);

    dispatch(payer, rem, amount_in, a_to_b, dex);

    Ok((end, token_bal(&accs[dst_ta_idx]).saturating_sub(dst_before)))
}

pub fn exec(_prog_id: &Address, accs: &[AccountView], data: &[u8]) -> ProgramResult {
    if data.is_empty() {
        return Err(errs::ix_data_too_short());
    }

    let flags = data[0];
    let payer = &accs[0];
    let mut offset = 1usize;

    match flags {
        0x01 => {
            if data.len() < 1 + SwapV1Chained::HDR_LEN {
                return Err(errs::ix_data_too_short());
            }

            let ix = unsafe { &*(data[1..].as_ptr() as *const SwapV1Chained) };

            let num_steps = ix.num_steps as usize;
            if num_steps == 0 || num_steps > MAX_HOPS {
                return Err(errs::route_plan_is_inadequate());
            }

            let steps_start = 1 + SwapV1Chained::HDR_LEN;
            let step_size = core::mem::size_of::<RoutePlanStepV1>();
            if data.len() < steps_start + num_steps * step_size {
                return Err(errs::ix_data_too_short());
            }

            let mut amount_in = ix.amount_in();
            let mut dst_after = 0u64;

            for (i, step) in ix.route_plan().iter().enumerate() {
                let (end, delta) = exec_hop(payer, accs, offset, step.swap, step.a_to_b(), amount_in, i as u8)?;
                dst_after += delta;
                amount_in = delta;
                offset = end;
            }

            let out_min = ix.amount_out_min();
            if out_min > 0 && dst_after < out_min {
                return Err(errs::output_below_min());
            }
        }
        0x02 => {
            if data.len() < 2 {
                return Err(errs::ix_data_too_short());
            }

            let num_steps = data[1] as usize;
            if num_steps == 0 || num_steps > MAX_HOPS {
                return Err(errs::route_plan_is_inadequate());
            }

            let steps_start = 2;
            let step_size = core::mem::size_of::<RoutePlanStepV1S>();
            if data.len() < steps_start + num_steps * step_size {
                return Err(errs::ix_data_too_short());
            }

            let steps = unsafe { core::slice::from_raw_parts(data[steps_start..].as_ptr() as *const RoutePlanStepV1S, num_steps) };

            for (i, step) in steps.iter().enumerate() {
                let (end, delta) = exec_hop(payer, accs, offset, step.swap, step.a_to_b(), step.amount_in(), i as u8)?;
                let out_min = step.amount_out_min();
                if out_min > 0 && delta < out_min {
                    return Err(errs::output_below_min());
                }
                offset = end;
            }
        }
        _ => return Err(errs::unknown_flags()),
    }

    Ok(())
}
