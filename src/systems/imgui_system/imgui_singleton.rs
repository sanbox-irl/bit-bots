use super::{imgui_component_utils::InspectorParameters, *};
use std::collections::HashMap;

pub fn singleton_inspector(
    sd: &mut SingletonDatabase,
    cd_n: &ComponentList<Name>,
    entities: &[Entity],
    prefab_hashmap: &PrefabMap,
    ui_handler: &mut UiHandler<'_>,
) -> bool {
    let mut is_open = true;
    let tileset_viewer_window = imgui::Window::new(imgui::im_str!("Singleton Inspector"))
        .size(Vec2::new(290.0, 400.0).into(), imgui::Condition::FirstUseEver)
        .opened(&mut is_open);

    if let Some(window) = tileset_viewer_window.begin(&ui_handler.ui) {
        // @update_singletons
        inspect_this_singleton_component(
            &mut sd.camera,
            &mut sd.associated_entities,
            cd_n,
            entities,
            prefab_hashmap,
            ui_handler,
            is_open,
        );

        // inspect_this_singleton_component(
        //     &mut sd.player,
        //     &mut sd.associated_entities,
        //     cd_n,
        //     entities,
        //     prefab_hashmap,
        //     ui_handler,
        //     is_open,
        //     |serialized, live| serialized.player = live.clone(),
        //     |serialized, live| *live = serialized.player.clone(),
        // );
        window.end(&ui_handler.ui);
    }

    is_open
}

fn inspect_this_singleton_component<T: SingletonBounds>(
    singleton_component: &mut SingletonComponent<T>,
    associated_entities: &mut HashMap<Marker, Entity>,
    name_list: &ComponentList<Name>,
    entities: &[Entity],
    prefabs: &PrefabMap,
    ui_handler: &mut UiHandler<'_>,
    is_open: bool,
) {
    let ui: &mut Ui<'_> = &mut ui_handler.ui;

    let marker_name = imgui::ImString::new(singleton_component.marker().to_string());
    let popup_name = im_str!("Select Associated Entity##Popup {}", marker_name);
    ui.text(&marker_name);

    // Associated Entity
    let two_thirds_size = ui.window_size()[0] * (2.0 / 3.0);
    ui.same_line(two_thirds_size);
    if let Some(assoc_entity) = associated_entities.get(&singleton_component.marker()) {
        if ui.button(&im_str!("Associated Entity##{}", marker_name), [0.0, 0.0]) {
            if ui_handler.stored_ids.contains(&assoc_entity) {
                ui_handler.stored_ids.remove(&assoc_entity);
            } else {
                ui_handler.stored_ids.insert(*assoc_entity);
            }
        }
    } else if ui.button(&im_str!("Select Associated Entity##{}", marker_name), [0.0, 0.0]) {
        ui.open_popup(&popup_name);
    }

    // Inspector
    ui.spacing();

    let uid = &singleton_component.marker().to_string();
    let inspector_parameters = InspectorParameters {
        is_open,
        uid,
        ui,
        entities,
        prefabs,
        entity_names: name_list,
    };
    singleton_component
        .inner_mut()
        .entity_inspector(inspector_parameters);

    // Serde
    ui.spacing();
    if ui.button(&im_str!("Change Associated Entity##{}", marker_name), [0.0, 0.0]) {
        ui.open_popup(&popup_name);
    }

    // Select a new Associated Entity:
    ui.popup_modal(&popup_name)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .build(|| {
            for this_entity in entities {
                let mut close_popup = false;

                let name_imstr = if let Some(name) = name_list.get(this_entity) {
                    imgui::ImString::new(name.inner().name.clone())
                } else {
                    im_str!("Entity ID {}", this_entity.index())
                };

                if ui.button(&name_imstr, [0.0, 0.0]) {
                    associated_entities.insert(singleton_component.marker(), *this_entity);
                    close_popup = true;
                }

                if close_popup || imgui_utility::pressed_escape(ui) {
                    ui.close_current_popup();
                }
            }
        });

    ui.separator();
}
