#![no_std]
///            o8%8888,
///          o88%8888888.
///         8'-    -:8888b
///        8'         8888
///       d8.-=. ,==-.:888b
///       >8 `~` :`~' d8888
///       88         ,88888
///       88b. `-~  ':88888
///       888b ~==~ .:88888
///       88888o--:':::8888
///       `88888| :::' 8888b
///       8888^^'       8888b
///      d888           ,%888b.
///     d88%            %%%8--'-.
///    /88:.__ ,       _%-' ---  -
///        '''::===..-'   =  --.
///
/// Swap instruction variant:
/// - `swap_v1` — flat account layout.

#[cfg(feature = "v1")]
pub mod adapters;
pub mod cons;
pub mod errs;
pub mod ixs;

use pinocchio::{default_allocator, nostd_panic_handler, program_entrypoint, AccountView, Address, ProgramResult};

program_entrypoint!(process_ix);
default_allocator!();
nostd_panic_handler!();

#[cfg(feature = "v1")]
const IX_ROUTE_V1: u8 = 0x01;

pub fn process_ix(program_id: &Address, accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    if data.is_empty() {
        return Err(errs::zero_swap_data());
    }

    let sel = data[0];
    let rest = &data[1..];

    match sel {
        #[cfg(feature = "v1")]
        IX_ROUTE_V1 => ixs::swap_v1::exec(program_id, accounts, rest),
        _ => Err(errs::unsupported_dex()),
    }
}
