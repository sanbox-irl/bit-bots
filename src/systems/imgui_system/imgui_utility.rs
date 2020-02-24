use super::*;
use imgui::*;

pub const RIGHT_CHEVRON: char = '\u{f105}';
pub const DOWN_CHEVRON: char = '\u{f107}';
pub const ENTITY_ICON: char = '\u{f6d1}';
pub const WARNING_ICON: char = '\u{f071}';
pub const SYNCED_ICON: char = '\u{f00c}';

pub type ImColor = [f32; 4];

pub fn yellow_warning_color() -> ImColor {
    Color::with_u8(253, 229, 109, 255).into()
}

pub fn green_color() -> ImColor {
    Color::with_u8(169, 220, 118, 255).into()
}

pub fn red_warning_color() -> ImColor {
    Color::with_u8(238, 93, 67, 255).into()
}

pub fn base_grey_color() -> ImColor {
    Color::with_u8(202, 205, 210, 255).into()
}

pub fn prefab_light_blue_color() -> ImColor {
    Color::with_u8(122, 203, 222, 255).into()
}

pub fn prefab_blue_color() -> ImColor {
    Color::with_u8(188, 203, 222, 255).into()
}

pub fn typed_text_ui<T: typename::TypeName>() -> String {
    let type_name = T::type_name();
    type_name.split("::").last().unwrap_or(&type_name).to_string()
}

pub fn label_button(ui: &Ui<'_>, label: &ImStr, button: &ImStr) -> bool {
    let button_size_y = ui.calc_text_size(button, true, 10000.0)[1] + 10.0;

    let ret = ui.button(
        button,
        [
            ui.window_content_region_width() * 2.0 / 3.0 - ui.window_content_region_min()[0],
            button_size_y,
        ],
    );

    ui.same_line(0.0);
    ui.text(label);

    ret
}

pub fn pretty_option_debug<T: std::fmt::Debug>(optional_val: &Option<T>) -> ImString {
    match optional_val {
        Some(ov) => im_str!("{:?}", ov),
        None => im_str!("None").to_owned(),
    }
}

pub fn pressed_escape(ui: &Ui<'_>) -> bool {
    pressed_key(ui, Key::Escape)
}

pub fn pressed_key(ui: &Ui<'_>, key: Key) -> bool {
    let key = ui.key_index(key);
    ui.is_key_pressed(key)
}

pub fn typed_enum_selection<
    T: typename::TypeName + std::fmt::Debug + strum::IntoEnumIterator<Iterator = I>,
    I: Iterator<Item = T>,
>(
    ui: &Ui<'_>,
    current_value: &T,
    unique_id: &str,
) -> Option<T> {
    let enum_typed_name = typed_text_ui::<T>();

    let popup_name = im_str!("{}##{}", enum_typed_name, unique_id);
    let pressed = label_button(
        ui,
        &ImString::new(enum_typed_name),
        &im_str!("{:?}##{}", &current_value, unique_id),
    );

    if pressed {
        ui.open_popup(&popup_name);
    }

    let mut variant: Option<T> = None;

    // Popup
    ui.popup(&popup_name, || {
        for this_variant in T::iter() {
            if MenuItem::new(&im_str!("{:?}", this_variant)).build(ui) {
                variant = Some(this_variant);
            }
        }
    });

    variant
}

pub fn typed_enum_selection_option<T, I>(
    ui: &Ui<'_>,
    current_value: &Option<T>,
    unique_id: &str,
) -> Option<Option<T>>
where
    T: typename::TypeName + std::fmt::Debug + strum::IntoEnumIterator<Iterator = I>,
    I: Iterator<Item = T>,
{
    let enum_typed_name = typed_text_ui::<T>();
    typed_enum_selection_option_raw(ui, current_value, &enum_typed_name, unique_id)
}

pub fn typed_enum_selection_option_named<T, I>(
    ui: &Ui<'_>,
    current_value: &Option<T>,
    name: &str,
    unique_id: &str,
) -> Option<Option<T>>
where
    T: std::fmt::Debug + strum::IntoEnumIterator<Iterator = I>,
    I: Iterator<Item = T>,
{
    typed_enum_selection_option_raw(ui, current_value, name, unique_id)
}

fn typed_enum_selection_option_raw<T, I>(
    ui: &Ui<'_>,
    current_value: &Option<T>,
    name: &str,
    uid: &str,
) -> Option<Option<T>>
where
    T: std::fmt::Debug + strum::IntoEnumIterator<Iterator = I>,
    I: Iterator<Item = T>,
{
    let mut variant: Option<Option<T>> = None;

    // Button
    let popup_name = im_str!("{}##{}", name, uid);
    let pressed = label_button(
        ui,
        &ImString::new(name),
        &im_str!("{}##{}", &pretty_option_debug(current_value), uid),
    );

    if pressed {
        ui.open_popup(&popup_name);
    }

    // Popup
    ui.popup(&popup_name, || {
        for this_variant in T::iter() {
            if MenuItem::new(&im_str!("{:?}", this_variant)).build(ui) {
                variant = Some(Some(this_variant));
            }
        }

        if MenuItem::new(im_str!("None")).build(ui) {
            variant = Some(None);
        }
    });

    variant
}

pub fn select_entity_option(
    label: &str,
    current_value: &Option<Entity>,
    uid: &str,
    ui: &Ui<'_>,
    entities: &[Entity],
    name_list: &ComponentList<Name>,
) -> Option<Option<Entity>> {
    let popup = im_str!("{}## Popup {}", label, uid);

    let name_str: ImString = match current_value {
        Some(cv) => {
            if let Some(name) = name_list.get(cv) {
                ImString::new(name.inner().name.clone())
            } else {
                im_str!("Entity ID {}", cv.index())
            }
        }

        None => ImString::new("None".to_string()),
    };

    if label_button(ui, &ImString::new(label), &name_str) {
        ui.open_popup(&popup);
    }

    let mut ret: Option<Option<Entity>> = None;

    // Select a new Associated Entity:
    ui.popup(&popup, || {
        let mut close_popup = false;

        // Somes
        for this_entity in entities {
            let name_imstr = ImString::new(Name::get_name_quick(name_list, this_entity));

            if ui.button(&name_imstr, [0.0, 0.0]) {
                ret = Some(Some(this_entity.clone()));
                close_popup = true;
            }
        }

        // None
        if ui.button(im_str!("None"), [0.0, 0.0]) {
            ret = Some(None);
            close_popup = true;
        }

        if close_popup || imgui_utility::pressed_escape(ui) {
            ui.close_current_popup();
        }
    });

    ret
}

pub fn select_entity(
    label: &str,
    uid: &str,
    ui: &Ui<'_>,
    entities: &[Entity],
    name_list: &ComponentList<Name>,
) -> Option<Entity> {
    let popup = im_str!("{}## Popup {}", label, uid);

    if ui.button(&im_str!("{}##{}", label, uid), [0.0, 0.0]) {
        ui.open_popup(&popup);
    }

    let mut ret: Option<Entity> = None;

    // Select a new Associated Entity:
    ui.popup(&popup, || {
        let mut close_popup = false;

        // Somes
        for this_entity in entities {
            let name_imstr = ImString::new(Name::get_name_quick(name_list, this_entity));

            if ui.button(&name_imstr, [0.0, 0.0]) {
                ret = Some(this_entity.clone());
                close_popup = true;
            }
        }

        // None
        if ui.button(im_str!("None"), [0.0, 0.0]) {
            ret = None;
            close_popup = true;
        }

        if close_popup || imgui_utility::pressed_escape(ui) {
            ui.close_current_popup();
        }
    });

    ret
}

pub fn select_prefab_entity(
    label: &str,
    current_value: &Option<uuid::Uuid>,
    uid: &str,
    ui: &Ui<'_>,
    serialized_prefabs: &PrefabMap,
) -> Option<Option<uuid::Uuid>> {
    let popup = im_str!("{}## Popup {}", label, uid);

    let name_str: ImString = match current_value {
        Some(cv) => match serialized_prefabs.get(cv) {
            Some(prefab) => match &prefab.root_entity().name {
                Some(sc) => ImString::new(sc.inner.name.clone()),
                None => im_str!("Prefab Uuid {}", prefab.root_id()),
            },
            None => {
                error!(
                    "Prefab has current value {:?} but no prefab matches that ID!",
                    current_value
                );
                im_str!("None").to_owned()
            }
        },

        None => im_str!("None").to_owned(),
    };

    if label_button(ui, &ImString::new(label), &name_str) {
        ui.open_popup(&popup);
    }

    let mut ret: Option<Option<uuid::Uuid>> = None;

    // Select a new Associated Entity:
    ui.popup(&popup, || {
        let mut close_popup = false;

        // Somes
        for prefab in serialized_prefabs.values() {
            let name_imstr = match &prefab.root_entity().name {
                Some(sc) => ImString::new(sc.inner.name.clone()),
                None => im_str!("Prefab Uuid {}", prefab.root_id()),
            };

            if ui.button(&name_imstr, [0.0, 0.0]) {
                ret = Some(Some(prefab.root_id()));
                close_popup = true;
            }
        }

        // None
        if ui.button(im_str!("None"), [0.0, 0.0]) {
            ret = Some(None);
            close_popup = true;
        }

        if close_popup || imgui_utility::pressed_escape(ui) {
            ui.close_current_popup();
        }
    });

    ret
}

pub fn input_usize(ui: &Ui<'_>, label: &ImStr, value: &mut usize) -> bool {
    let mut size_val: i32 = *value as i32;

    if ui.input_int(label, &mut size_val).build() {
        if size_val < 0 {
            size_val = 0;
        }
        *value = size_val as usize;

        true
    } else {
        false
    }
}

pub fn input_isize(ui: &Ui<'_>, label: &str, uid: &str, value: &mut isize) -> bool {
    let mut val = *value as i32;
    if ui.input_int(&im_str!("{}##{}", label, uid), &mut val).build() {
        *value = val as isize;
        true
    } else {
        false
    }
}

pub fn create_window<F>(ui_handler: &mut UiHandler<'_>, flag: ImGuiFlags, mut f: F)
where
    F: FnMut(&mut UiHandler<'_>) -> bool,
{
    if ui_handler.flags.contains(flag) {
        let is_opened = f(ui_handler);
        if is_opened == false {
            ui_handler.flags.remove(flag);
        }
    }
}

pub fn typed_option_selection<T: Default, SomeF, NoneF>(
    label: &str,
    some_label: &str,
    none_label: &str,
    ui: &Ui<'_>,
    uid: &str,
    option: &mut Option<T>,
    mut options_for_some: SomeF,
    mut options_for_none: NoneF,
) -> bool
where
    SomeF: FnMut(&mut T) -> bool,
    NoneF: FnMut(&mut Option<T>) -> bool,
{
    ui.text(label);
    let some_text = im_str!("{}##{}", some_label, uid);
    ui.same_line_with_spacing(0.0, 25.0);
    let clicky_some = ui.radio_button_bool(&some_text, option.is_some());
    ui.same_line_with_spacing(0.0, 70.0);
    let clicky_none = ui.radio_button_bool(&im_str!("{}##{}", none_label, uid), option.is_none());

    let mut ret_val = false;

    if clicky_some {
        if let None = option {
            *option = Some(T::default());
            ret_val = true;
        }
    }

    if let Some(inner) = option {
        if options_for_some(inner) {
            ret_val = true;
        }
    }

    if clicky_none {
        if let Some(_) = option {
            *option = None;
            ret_val = true;
        }
    }

    if let None = option {
        if options_for_none(option) {
            ret_val = true;
        }
    }

    ret_val
}

pub fn left_clicked_item(ui: &Ui<'_>) -> bool {
    ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Left)
}

pub fn right_click_popup<F: FnOnce()>(ui: &Ui<'_>, uid: &str, f: F) {
    let name = im_str!("{}", uid);
    if ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
        ui.open_popup(&name);
    }

    ui.popup(&name, f);
}

pub fn help_menu_item<T: AsRef<str>>(ui: &Ui<'_>, label: &ImStr, tooltip: T) -> bool {
    let do_it = MenuItem::new(label).build(ui);
    imgui_system::help_marker(ui, tooltip);
    do_it
}

pub fn help_marker<T: AsRef<str>>(ui: &Ui<'_>, message: T) {
    ui.same_line(0.0);
    ui.text_disabled("(?)");
    if ui.is_item_hovered() {
        ui.tooltip_text(message);
    }
}

pub fn help_marker_generic<T: AsRef<str>>(ui: &Ui<'_>, item: char, message: T) {
    ui.same_line(0.0);
    ui.text(&im_str!("{}", item));
    if ui.is_item_hovered() {
        ui.tooltip_text(message);
    }
}

pub fn wrap_style_var(ui: &Ui<'_>, style_var: StyleVar, f: impl FnOnce()) {
    let style_var_token = ui.push_style_var(style_var);

    f();

    style_var_token.pop(ui);
}

pub fn wrap_style_color_var(ui: &Ui<'_>, style_color: StyleColor, color: [f32; 4], f: impl FnOnce()) {
    let style_color_token = ui.push_style_color(style_color, color);

    f();

    style_color_token.pop(ui);
}

pub fn imgui_str(message: &str, uid: &str) -> ImString {
    im_str!("{}##{}", message, uid)
}
