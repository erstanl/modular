mod acrc;
mod adsr;
mod ahrd;
mod shared;

use acrc::{acrc, acrc_loop};
use adsr::adsr;
use ahrd::ahrd;

pub use self::acrc::{AcrcLoopState, AcrcState};
pub use self::adsr::AdsrState;
pub use self::ahrd::AhrdState;

#[derive(Copy, Clone)]
pub enum TriggerAction {
    None,
    GateRise,
    GateFall,
    Trigger,
}

pub struct EnvelopeState {
    pub mode: EnvelopeMode,
    pub time: u32,
    pub last_value: u16,
}

#[derive(Copy, Clone)]
pub enum EnvelopeMode {
    Adsr(AdsrState),
    Acrc(AcrcState),
    AcrcLoop(AcrcLoopState),
    AhrdLoop(AhrdState),
}

pub fn ui_show_mode(state: &EnvelopeMode) -> u8 {
    match state {
        EnvelopeMode::Adsr(_) => 0b1000 as u8,
        EnvelopeMode::Acrc(_) => 0b0100,
        EnvelopeMode::AcrcLoop(_) => 0b0010,
        EnvelopeMode::AhrdLoop(_) => 0b0001,
    }
    .reverse_bits()
}

pub fn ui_show_stage(state: &EnvelopeMode) -> u8 {
    match state {
        EnvelopeMode::Adsr(phase) => match phase {
            AdsrState::Wait => 0b0000 as u8,
            AdsrState::Attack => 0b1000,
            AdsrState::Decay => 0b0100,
            AdsrState::Sustain => 0b0010,
            AdsrState::Release => 0b0001,
        },
        EnvelopeMode::Acrc(phase) => match phase {
            AcrcState::Wait => 0b0000,
            AcrcState::Attack => 0b1100,
            AcrcState::Release => 0b0011,
        },
        EnvelopeMode::AcrcLoop(phase) => match phase {
            AcrcLoopState::Attack => 0b1100,
            AcrcLoopState::Release => 0b0011,
        },
        EnvelopeMode::AhrdLoop(phase) => match phase {
            AhrdState::Attack => 0b1000,
            AhrdState::Hold => 0b0100,
            AhrdState::Release => 0b0010,
            AhrdState::Delay => 0b0001,
        },
    }
    .reverse_bits()
}

#[derive(Copy, Clone)]
struct Fraction<T> {
    numerator: T,
    denominator: T,
}

pub fn update(state: &mut EnvelopeState, trigger: TriggerAction, cv: &[u16; 4]) -> (u16, bool) {
    let (value, rollover) = match state.mode {
        EnvelopeMode::Adsr(ref mut phase) => {
            adsr(phase, &mut state.time, state.last_value, trigger, cv)
        }
        EnvelopeMode::Acrc(ref mut phase) => {
            acrc(phase, &mut state.time, state.last_value, trigger, cv)
        }
        EnvelopeMode::AcrcLoop(ref mut phase) => acrc_loop(phase, &mut state.time, cv),
        EnvelopeMode::AhrdLoop(ref mut phase) => ahrd(phase, &mut state.time, trigger, cv),
    };

    debug_assert!(value <= MAX_DAC_VALUE);
    state.last_value = value;

    (value, rollover)
}

const MAX_DAC_VALUE: u16 = 4095;
