use super::{ImGui, Input, Vec2};
use imgui_winit_support::WinitPlatform;
use winit::{
    dpi::LogicalPosition,
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput as WinitKeyboardInput,
        MouseButton as WinitMouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    platform::desktop::EventLoopExtDesktop,
    window::Window as WinitWindow,
};

pub fn poll_events(
    input_component: &mut Input,
    events_loop: &mut EventLoop<()>,
    winit_window: &WinitWindow,
    imgui: &mut ImGui,
) {
    let keys_pressed_last_frame = input_component.kb_input.pressed_keys.clone();
    let mouse_button_clicked_last_frame = input_component.mouse_input.mouse_pressed;
    input_component.clear_input();

    events_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Callback
        imgui_event_handler(&mut imgui.platform, imgui.imgui.io_mut(), winit_window, &event);

        // Our own Input Handling
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                input_component.end_requested = true;
                *control_flow = ControlFlow::Exit;
            }

            Event::MainEventsCleared => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(logical),
                ..
            } => {
                input_component.new_frame_size = Some(Vec2::new(logical.width as f32, logical.height as f32));
                info!("New Requested Frame Size: {:?}", input_component.new_frame_size);
            }

            Event::DeviceEvent {
                event:
                    DeviceEvent::Key(WinitKeyboardInput {
                        virtual_keycode: Some(code),
                        state,
                        ..
                    }),
                ..
            } => {
                input_component.record_input(state, code, &keys_pressed_last_frame);
            }

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                input_component.mouse_input.mouse_position = Vec2::new(position.x as f32, position.y as f32);
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button,
                        ..
                    },
                ..
            } => {
                let this_button = match button {
                    WinitMouseButton::Left => 0,
                    WinitMouseButton::Right => 1,
                    WinitMouseButton::Middle => 2,
                    WinitMouseButton::Other(num) => num as usize,
                };

                if mouse_button_clicked_last_frame[this_button] == false {
                    input_component.mouse_input.mouse_pressed[this_button] = true;
                    input_component.mouse_input.mouse_held[this_button] = true;
                }
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button,
                        ..
                    },
                ..
            } => {
                let this_button = match button {
                    WinitMouseButton::Left => 0,
                    WinitMouseButton::Right => 1,
                    WinitMouseButton::Middle => 2,
                    WinitMouseButton::Other(num) => num as usize,
                };

                if input_component.mouse_input.mouse_pressed[this_button]
                    || input_component.mouse_input.mouse_held[this_button]
                {
                    input_component.mouse_input.mouse_pressed[this_button] = false;
                    input_component.mouse_input.mouse_held[this_button] = false;

                    input_component.mouse_input.mouse_released[this_button] = true;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::MouseWheel {
                    delta: scroll_delta, ..
                },
                ..
            } => match scroll_delta {
                MouseScrollDelta::PixelDelta(LogicalPosition { y: vertical_move, .. }) => {
                    input_component.mouse_input.mouse_vertical_scroll_delta = -vertical_move as f32;
                }

                MouseScrollDelta::LineDelta(_, vertical_move) => {
                    input_component.mouse_input.mouse_vertical_scroll_delta = -vertical_move;
                }
            },

            #[cfg(target_os = "macos")]
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            WinitKeyboardInput {
                                state,
                                virtual_keycode: Some(code),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                input_component.record_input(state, code, &keys_pressed_last_frame);
            }

            _ => {}
        }
    });
}

pub fn imgui_event_handler<T>(
    platform: &mut WinitPlatform,
    io: &mut imgui::Io,
    window: &WinitWindow,
    event: &Event<'_, T>,
) {
    match *event {
        Event::WindowEvent { window_id, ref event } if window_id == window.id() => {
            imgui_window_event(platform, io, window, event);
        }
        // Track key release events outside our window. If we don't do this,
        // we might never see the release event if some other window gets focus.
        Event::DeviceEvent {
            event:
                DeviceEvent::Key(WinitKeyboardInput {
                    state: ElementState::Released,
                    virtual_keycode: Some(key),
                    ..
                }),
            ..
        } => {
            io.keys_down[key as usize] = false;
        }
        Event::DeviceEvent {
            event: DeviceEvent::ModifiersChanged(modifiers_changed),
            ..
        } => {
            io.key_shift = modifiers_changed.shift();
            io.key_ctrl = modifiers_changed.ctrl();
            io.key_alt = modifiers_changed.alt();
            io.key_super = modifiers_changed.logo();
        }
        _ => (),
    }
}

fn imgui_window_event(
    platform: &mut WinitPlatform,
    io: &mut imgui::Io,
    window: &WinitWindow,
    event: &WindowEvent<'_>,
) {
    match *event {
        WindowEvent::Resized(physical_size) => {
            let logical_size = physical_size.to_logical(window.scale_factor());
            let logical_size = platform.scale_size_from_winit(window, logical_size);
            io.display_size = [logical_size.width as f32, logical_size.height as f32];
        }
        // WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
        //     // platform.hidpi_factor()
        //     // let hidpi_factor = match self.hidpi_mode {
        //     //     ActiveHiDpiMode::Default => scale_factor,
        //     //     ActiveHiDpiMode::Rounded => scale_factor.round(),
        //     //     _ => return,
        //     // };
        //     // // Mouse position needs to be changed while we still have both the old and the new
        //     // // values
        //     // if io.mouse_pos[0].is_finite() && io.mouse_pos[1].is_finite() {
        //     //     io.mouse_pos = [
        //     //         io.mouse_pos[0] * (hidpi_factor / self.hidpi_factor) as f32,
        //     //         io.mouse_pos[1] * (hidpi_factor / self.hidpi_factor) as f32,
        //     //     ];
        //     // }
        //     // self.hidpi_factor = hidpi_factor;
        //     // io.display_framebuffer_scale = [hidpi_factor as f32, hidpi_factor as f32];
        //     // // Window size might change too if we are using DPI rounding
        //     // let logical_size = window.inner_size().to_logical(scale_factor);
        //     // let logical_size = self.scale_size_from_winit(window, logical_size);
        //     // io.display_size = [logical_size.width as f32, logical_size.height as f32];
        // }
        WindowEvent::KeyboardInput {
            input:
                WinitKeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
            ..
        } => {
            let pressed = state == ElementState::Pressed;
            io.keys_down[key as usize] = pressed;
        }

        WindowEvent::ReceivedCharacter(ch) => {
            // Exclude the backspace key ('\u{7f}'). Otherwise we will insert this char and then
            // delete it.
            if ch != '\u{7f}' {
                io.add_input_character(ch)
            }
        }
        WindowEvent::CursorMoved { position, .. } => {
            let position = position.to_logical(window.scale_factor());
            let position = platform.scale_pos_from_winit(window, position);
            io.mouse_pos = [position.x as f32, position.y as f32];
        }

        WindowEvent::MouseWheel {
            delta,
            phase: TouchPhase::Moved,
            ..
        } => match delta {
            MouseScrollDelta::LineDelta(h, v) => {
                io.mouse_wheel_h = h;
                io.mouse_wheel = v;
            }
            MouseScrollDelta::PixelDelta(pos) => {
                match pos.x.partial_cmp(&0.0) {
                    Some(std::cmp::Ordering::Greater) => io.mouse_wheel_h += 1.0,
                    Some(std::cmp::Ordering::Less) => io.mouse_wheel_h -= 1.0,
                    _ => (),
                }
                match pos.y.partial_cmp(&0.0) {
                    Some(std::cmp::Ordering::Greater) => io.mouse_wheel += 1.0,
                    Some(std::cmp::Ordering::Less) => io.mouse_wheel -= 1.0,
                    _ => (),
                }
            }
        },
        WindowEvent::MouseInput { state, button, .. } => {
            let pressed = state == ElementState::Pressed;
            match button {
                WinitMouseButton::Left => io.mouse_down[0] = pressed,
                WinitMouseButton::Right => io.mouse_down[1] = pressed,
                WinitMouseButton::Middle => io.mouse_down[2] = pressed,
                WinitMouseButton::Other(idx @ 0..=4) => io.mouse_down[idx as usize] = pressed,
                _ => (),
            }
        }
        _ => (),
    }
}
