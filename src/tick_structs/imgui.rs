use super::{
    game_config::Config, imgui_component_utils::EntityListInformation, ClipboardSupport, Entity,
    EntityAllocator, Window,
};
use anyhow::Error;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::collections::HashMap;
use winit::{event::Event, window::Window as WinitWindow};

#[allow(dead_code)]
pub struct ImGui {
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub meta_data: ImGuiMetaData,
}

#[allow(dead_code)]
impl ImGui {
    pub fn new(entity_allocator: &EntityAllocator, window: &Window, config: &Config) -> Self {
        let mut imgui = Context::create();
        if let Some(clipboard_context) = ClipboardSupport::new() {
            imgui.set_clipboard_backend(Box::new(clipboard_context));
        }

        // Ini Save Location:
        use std::path::Path;
        let ini_save_path = Path::new("dev/imgui/imgui_ini.ini").to_path_buf();
        imgui.set_ini_filename(Some(ini_save_path));

        // Logging Location
        let log_location = Path::new("dev/imgui/imgui_log.txt").to_path_buf();
        imgui.set_log_filename(log_location);

        // Set Style
        Self::set_style(imgui.style_mut());

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window.window, HiDpiMode::Locked(1.0));

        // Reconfigure the font -- this is a hack to deal with HDPi.
        let roboto_font_data = include_bytes!("../../assets/fonts/imgui_font.ttf");
        let icon_font = include_bytes!("../../assets/fonts/imgui_font_icons.otf");

        imgui.fonts().add_font(&[
            FontSource::TtfData {
                config: Some(FontConfig {
                    size_pixels: (13.0 * window.window.scale_factor()) as f32,
                    ..FontConfig::default()
                }),
                data: roboto_font_data,
                size_pixels: config.imgui_pixel_size,
            },
            FontSource::TtfData {
                config: Some(FontConfig {
                    size_pixels: (13.0 * window.window.scale_factor()) as f32,
                    pixel_snap_h: true,
                    glyph_ranges: FontGlyphRanges::from_slice(&[0xf000, 0xf941, 0]),
                    ..FontConfig::default()
                }),
                data: icon_font,
                size_pixels: config.imgui_pixel_size,
            },
        ]);

        // Load Our own MetaData:
        let data = include_str!("../../dev/imgui/meta_data.yaml");
        let mut meta_data: ImGuiMetaData = serde_yaml::from_str(data).unwrap_or_default();

        // Fix the meta_data!
        meta_data.stored_ids.retain(|&id| entity_allocator.is_live(id));

        // Mac Setup
        #[cfg(target_os = "macos")]
        {
            imgui.io_mut().config_mac_os_behaviors = true;
        }

        Self {
            imgui,
            platform,
            meta_data,
        }
    }

    pub fn save_meta_data(&mut self) -> Result<(), Error> {
        let success = serde_yaml::to_string(&self.meta_data)?;
        std::fs::write("dev/imgui/meta_data.yaml", success)?;

        Ok(())
    }

    pub fn take_input(&mut self, window: &WinitWindow, event: &Event<'_, ()>) {
        self.platform.handle_event(self.imgui.io_mut(), window, event);
    }

    pub fn begin_frame<'a>(
        &'a mut self,
        window: &Window,
        pressed_save: bool,
        delta_time: f32,
    ) -> Result<UiHandler<'a>, Error> {
        self.platform
            .prepare_frame(self.imgui.io_mut(), &window.window)
            .map_err(|e| format_err!("{}", e))?;

        self.imgui.io_mut().delta_time = delta_time;
        let ui = self.imgui.frame();

        Ok(UiHandler {
            platform: &self.platform,
            flags: &mut self.meta_data.flags,
            stored_ids: &mut self.meta_data.stored_ids,
            scene_graph_entities: &mut self.meta_data.entity_vec,
            entity_list_information: &mut self.meta_data.entity_list_information,
            scene_changing_info: &mut self.meta_data.scene_changing_info,
            request_save: pressed_save && (ui.io().key_ctrl || ui.io().key_super),
            ui,
        })
    }

    fn set_style(style: &mut imgui::Style) {
        style.item_spacing = [7.0, 2.0];
        style.item_inner_spacing = [6.0, 4.0];
        style.indent_spacing = 12.0;

        style.scrollbar_size = 18.0;

        style.window_rounding = 5.3;
        style.grab_rounding = 2.3;
        style.frame_rounding = 2.3;
        style.frame_border_size = 1.0;

        style.use_dark_colors();
        style.colors[imgui::StyleColor::Text as usize] = [0.95, 0.96, 0.98, 1.00];
        style.colors[imgui::StyleColor::TextDisabled as usize] = [0.36, 0.42, 0.47, 1.00];
        style.colors[imgui::StyleColor::WindowBg as usize] = [0.11, 0.15, 0.17, 1.00];
        style.colors[imgui::StyleColor::ChildBg as usize] = [27.0 / 255.0, 32.0 / 255.0, 46.0 / 255.0, 1.00];
        style.colors[imgui::StyleColor::PopupBg as usize] = [0.08, 0.08, 0.08, 1.0];
        style.colors[imgui::StyleColor::Border as usize] = [0.08, 0.10, 0.12, 1.00];
        style.colors[imgui::StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.00];
        style.colors[imgui::StyleColor::FrameBg as usize] = [0.20, 0.25, 0.29, 1.00];
        style.colors[imgui::StyleColor::FrameBgHovered as usize] = [0.12, 0.20, 0.28, 1.00];
        style.colors[imgui::StyleColor::FrameBgActive as usize] = [0.09, 0.12, 0.14, 1.00];
        style.colors[imgui::StyleColor::TitleBg as usize] =
            style.colors[imgui::StyleColor::WindowBg as usize];
        style.colors[imgui::StyleColor::TitleBgActive as usize] =
            style.colors[imgui::StyleColor::WindowBg as usize];
        style.colors[imgui::StyleColor::TitleBgCollapsed as usize] =
            style.colors[imgui::StyleColor::WindowBg as usize];
        style.colors[imgui::StyleColor::MenuBarBg as usize] = [0.204, 0.211, 0.251, 1.000];
        style.colors[imgui::StyleColor::ScrollbarBg as usize] = [0.02, 0.02, 0.02, 0.39];
        style.colors[imgui::StyleColor::ScrollbarGrab as usize] = [0.20, 0.25, 0.29, 1.00];
        style.colors[imgui::StyleColor::ScrollbarGrabHovered as usize] = [0.18, 0.22, 0.25, 1.00];
        style.colors[imgui::StyleColor::ScrollbarGrabActive as usize] = [0.09, 0.21, 0.31, 1.00];
        style.colors[imgui::StyleColor::CheckMark as usize] = [0.28, 0.56, 1.00, 1.00];
        style.colors[imgui::StyleColor::SliderGrab as usize] = [0.28, 0.56, 1.00, 1.00];
        style.colors[imgui::StyleColor::SliderGrabActive as usize] = [0.37, 0.61, 1.00, 1.00];
        style.colors[imgui::StyleColor::Button as usize] = [0.20, 0.25, 0.29, 1.00];
        style.colors[imgui::StyleColor::ButtonHovered as usize] = [0.28, 0.56, 1.00, 1.00];
        style.colors[imgui::StyleColor::ButtonActive as usize] = [0.06, 0.53, 0.98, 1.00];
        style.colors[imgui::StyleColor::Header as usize] = [0.20, 0.25, 0.29, 0.55];
        style.colors[imgui::StyleColor::HeaderHovered as usize] = [0.26, 0.59, 0.98, 0.80];
        style.colors[imgui::StyleColor::HeaderActive as usize] = [0.26, 0.59, 0.98, 1.00];
        style.colors[imgui::StyleColor::Separator as usize] = [0.20, 0.25, 0.29, 1.00];
        style.colors[imgui::StyleColor::SeparatorHovered as usize] = [0.10, 0.40, 0.75, 0.78];
        style.colors[imgui::StyleColor::SeparatorActive as usize] = [0.10, 0.40, 0.75, 1.00];
        style.colors[imgui::StyleColor::ResizeGrip as usize] = [0.26, 0.59, 0.98, 0.25];
        style.colors[imgui::StyleColor::ResizeGripHovered as usize] = [0.26, 0.59, 0.98, 0.67];
        style.colors[imgui::StyleColor::ResizeGripActive as usize] = [0.26, 0.59, 0.98, 0.95];
        style.colors[imgui::StyleColor::Tab as usize] = [0.11, 0.15, 0.17, 1.00];
        style.colors[imgui::StyleColor::TabHovered as usize] = [0.26, 0.59, 0.98, 0.80];
        style.colors[imgui::StyleColor::TabActive as usize] = [0.20, 0.25, 0.29, 1.00];
        style.colors[imgui::StyleColor::TabUnfocused as usize] = [0.11, 0.15, 0.17, 1.00];
        style.colors[imgui::StyleColor::TabUnfocusedActive as usize] = [0.11, 0.15, 0.17, 1.00];
        style.colors[imgui::StyleColor::PlotLines as usize] = [0.61, 0.61, 0.61, 1.00];
        style.colors[imgui::StyleColor::PlotLinesHovered as usize] = [1.00, 0.43, 0.35, 1.00];
        style.colors[imgui::StyleColor::PlotHistogram as usize] = [0.90, 0.70, 0.00, 1.00];
        style.colors[imgui::StyleColor::PlotHistogramHovered as usize] = [1.00, 0.60, 0.00, 1.00];
        style.colors[imgui::StyleColor::TextSelectedBg as usize] = [0.26, 0.59, 0.98, 0.35];
        style.colors[imgui::StyleColor::DragDropTarget as usize] = [0.26, 0.59, 0.98, 1.00];
        style.colors[imgui::StyleColor::NavHighlight as usize] = [0.26, 0.59, 0.98, 1.00];
        style.colors[imgui::StyleColor::NavWindowingHighlight as usize] = [1.00, 1.00, 1.00, 0.70];
        style.colors[imgui::StyleColor::NavWindowingDimBg as usize] = [0.80, 0.80, 0.80, 0.20];
        style.colors[imgui::StyleColor::ModalWindowDimBg as usize] = [0.80, 0.80, 0.80, 0.35];
    }
}

use std::collections::HashSet;
pub struct UiHandler<'a> {
    pub ui: Ui<'a>,
    pub platform: &'a WinitPlatform,
    pub flags: &'a mut ImGuiFlags,
    pub stored_ids: &'a mut HashSet<Entity>,
    pub scene_graph_entities: &'a mut Vec<Entity>,
    pub entity_list_information: &'a mut HashMap<String, EntityListInformation>,
    pub scene_changing_info: &'a mut SceneImGuiManager,
    request_save: bool,
}

impl<'a> UiHandler<'a> {
    pub fn can_save_scene(&self) -> bool {
        if super::scene_system::current_scene_mode() == super::SceneMode::Draft {
            self.request_save
        } else {
            false
        }
    }

    pub fn prepare_render(&self, window: &Window) {
        self.platform.prepare_render(&self.ui, &window.window);
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ImGuiMetaData {
    pub flags: ImGuiFlags,
    pub stored_ids: HashSet<Entity>,
    #[serde(skip)]
    pub entity_vec: Vec<Entity>,
    #[serde(skip)]
    pub entity_list_information: HashMap<String, EntityListInformation>,
    #[serde(skip)]
    pub scene_changing_info: SceneImGuiManager,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SceneImGuiManager {
    pub create_scene: String,
    pub switch_scene_name: String,
    pub delete_scene_name: String,
}

use bitflags::bitflags;
bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct ImGuiFlags: u64 {
        const TIME_KEEPER           =   0b0000_0000_0000_0001;
        const SPRITE_RESOURCE       =   0b0000_0000_0000_0010;
        const TILEMAP_RESOURCE      =   0b0000_0000_0000_0100;
        const ENTITY_VIEWER         =   0b0000_0000_0000_1000;
        const SINGLETONS            =   0b0000_0000_0001_0000;
        const GAME_CONFIG           =   0b0000_0000_0010_0000;
        const PREFAB_INSPECTOR      =   0b0000_0000_0100_0000;
        const MAIN_MENU_BAR         =   0b0000_0000_1000_0000;
        const IMGUI_EXAMPLE         =   0b0000_0001_0000_0000;
    }
}
