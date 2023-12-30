use avr_progmem::progmem;
use embedded_graphics::pixelcolor::BinaryColor;

use crate::{
    clock::ClockConfig,
    display_buffer::{Justify, MiniBuffer, TextColor},
    font::PRO_FONT_29_NUMERIC,
    menu::{menu_state::EditingState, MenuUpdate},
    render_nubers::u8_to_str_b10,
};

progmem! {
    static progmem BPM_TEXT_IMG:  [u8; 19] = *include_bytes!("../../../assets/bpm_text.bin");
}

#[inline(never)]
pub fn render_bpm_page<DI, SIZE>(
    editing: EditingState,
    clock_state: &ClockConfig,
    menu_update: &MenuUpdate,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let mut buffer: [u8; 3] = [0u8; 3];
    let text = u8_to_str_b10(&mut buffer, clock_state.bpm);
    let mut mini_buffer = MiniBuffer::<64, 40>::new();

    if editing == EditingState::Editing {
        mini_buffer.fast_fill(0, 4, 64, 32, BinaryColor::On);
    }

    mini_buffer.fast_draw_ascii_text(
        Justify::Center(32),
        Justify::Center(20),
        text,
        &PRO_FONT_29_NUMERIC,
        match editing {
            EditingState::Editing => &TextColor::BinaryOffTransparent,
            EditingState::Navigating => &TextColor::BinaryOn,
        },
    );
    if *menu_update == MenuUpdate::SwitchScreens {
        display.clear().unwrap();
    }
    mini_buffer.blit(display, 32, 8).unwrap();
    drop(mini_buffer);
    if *menu_update == MenuUpdate::SwitchScreens {
        let mut bpm_buffer = MiniBuffer::<19, 8>::new();
        let img = BPM_TEXT_IMG.load();
        bpm_buffer.fast_draw_image(0, 0, 19, 8, &img, &TextColor::BinaryOn);
        bpm_buffer.blit(display, 54, 48).unwrap();
    }
}
