use gilrs::{Button, Event, Gilrs};
use macroquad::prelude::*;

#[macroquad::main("FightingBase")]
async fn main() {
    let mut x = screen_width() / 2.0;
    let mut y = screen_height() / 2.0;

    let mut gilrs = Gilrs::new().unwrap();
    let mut active_gamepad = None;
    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    loop {
        clear_background(WHITE);

        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
        }
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::DPadRight) {
                x += 1.0;
            }
            if gamepad.is_pressed(Button::DPadLeft) {
                x -= 1.0;
            }
            if gamepad.is_pressed(Button::DPadDown) {
                y += 1.0;
            }
            if gamepad.is_pressed(Button::DPadUp) {
                y -= 1.0;
            }
        }

        draw_circle(x, y, 15.0, YELLOW);
        draw_text(
            "move the ball with a controller DPad",
            20.0,
            20.0,
            25.0,
            DARKGRAY,
        );
        next_frame().await
    }
}
