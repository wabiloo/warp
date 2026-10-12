use std::path::PathBuf;

use warp_core::ui::appearance::Appearance;
use warp_core::ui::theme::Fill;
use warpui::elements::{
    ChildView, ClippedScrollStateHandle, ClippedScrollable, ConstrainedBox, Container,
    CornerRadius, CrossAxisAlignment, Dismiss, DropShadow, Flex, ParentElement, Radius,
    ScrollbarWidth, Text,
};
use warpui::fonts::Properties;
use warpui::keymap::FixedBinding;
use warpui::{
    AppContext, Element, Entity, TypedActionView, View, ViewContext, ViewHandle, SingletonEntity,
};

use crate::menu::{self, Event as MenuEvent, Menu, MenuItemFields};
use crate::ui_components::{blended_colors, icons};

const MENU_WIDTH: f32 = 300.0;
const MENU_MAX_HEIGHT: f32 = 260.0;
const TENANTS_FILE_NAME: &str = "tenants";

pub struct TenantPopupView {
    tenants: Vec<String>,
    tenants_menu: ViewHandle<Menu<TenantPopupAction>>,
    scroll_state: ClippedScrollStateHandle,
}

#[derive(Debug, Clone)]
pub enum TenantPopupAction {
    ClosePopup,
    SelectTenant { label: String },
}

#[derive(Debug, Clone)]
pub enum TenantPopupEvent {
    Close,
    SelectTenant { label: String },
}

pub fn init(app: &mut AppContext) {
    use warpui::keymap::macros::*;

    app.register_fixed_bindings([FixedBinding::new(
        "escape",
        TenantPopupAction::ClosePopup,
        id!(TenantPopupView::ui_name()),
    )]);
}

impl TenantPopupView {
    pub fn new(current_tenant: Option<String>, ctx: &mut ViewContext<Self>) -> Self {
        let tenants = list_tenants();
        let tenants_menu = ctx.add_typed_action_view(|ctx| {
            let mut menu = Menu::new().with_width(MENU_WIDTH);
            menu.set_items(Self::menu_items(&tenants, current_tenant.as_deref()), ctx);
            let selected_index = tenants
                .iter()
                .position(|tenant| Some(tenant.as_str()) == current_tenant.as_deref())
                .unwrap_or(0);
            menu.set_selected_by_index(selected_index, ctx);
            menu
        });
        ctx.subscribe_to_view(&tenants_menu, |_, _, event, ctx| {
            if let MenuEvent::Close { .. } = event {
                ctx.emit(TenantPopupEvent::Close);
            }
        });

        Self {
            tenants,
            tenants_menu,
            scroll_state: Default::default(),
        }
    }

    pub fn focus_content(&self, ctx: &mut ViewContext<Self>) {
        ctx.focus(&self.tenants_menu);
    }

    fn menu_items(
        tenants: &[String],
        current_tenant: Option<&str>,
    ) -> Vec<menu::MenuItem<TenantPopupAction>> {
        tenants
            .iter()
            .map(|label| {
                let mut fields = MenuItemFields::new(label).with_on_select_action(
                    TenantPopupAction::SelectTenant {
                        label: label.clone(),
                    },
                );
                fields = if Some(label.as_str()) == current_tenant {
                    fields.with_icon(icons::Icon::Check)
                } else {
                    fields.with_indent()
                };
                menu::MenuItem::Item(fields)
            })
            .collect()
    }

    fn render_empty_state(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        let theme = appearance.theme();
        let color = blended_colors::text_sub(theme, theme.surface_2());

        Container::new(
            Text::new(
                "No tenants found in your tenants file",
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(color)
            .soft_wrap(true)
            .finish(),
        )
        .with_uniform_padding(12.)
        .finish()
    }
}

impl View for TenantPopupView {
    fn ui_name() -> &'static str {
        "TenantPopup"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);

        let content = if self.tenants.is_empty() {
            self.render_empty_state(app)
        } else {
            Flex::column()
                .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
                .with_child(ChildView::new(&self.tenants_menu).finish())
                .finish()
        };

        let scrollable = ClippedScrollable::vertical(
            self.scroll_state.clone(),
            content,
            ScrollbarWidth::Auto,
            appearance.theme().nonactive_ui_detail().into(),
            appearance.theme().active_ui_detail().into(),
            warpui::elements::Fill::None,
        )
        .with_overlayed_scrollbar()
        .finish();

        let background: Fill = appearance.theme().surface_2();
        Dismiss::new(
            ConstrainedBox::new(
                Container::new(scrollable)
                    .with_background(background)
                    .with_corner_radius(CornerRadius::with_all(Radius::Pixels(8.)))
                    .with_drop_shadow(DropShadow::default())
                    .finish(),
            )
            .with_width(MENU_WIDTH)
            .with_max_height(MENU_MAX_HEIGHT)
            .finish(),
        )
        .prevent_interaction_with_other_elements()
        .on_dismiss(|ctx, _app| ctx.dispatch_typed_action(TenantPopupAction::ClosePopup))
        .finish()
    }
}

impl Entity for TenantPopupView {
    type Event = TenantPopupEvent;
}

impl TypedActionView for TenantPopupView {
    type Action = TenantPopupAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            TenantPopupAction::ClosePopup => ctx.emit(TenantPopupEvent::Close),
            TenantPopupAction::SelectTenant { label } => {
                ctx.emit(TenantPopupEvent::SelectTenant {
                    label: label.clone(),
                });
                ctx.emit(TenantPopupEvent::Close);
            }
        }
    }
}

fn tenants_file_path() -> Option<PathBuf> {
    let home = match std::env::var_os("BPKIO_HOME").filter(|home| !home.is_empty()) {
        Some(home) => PathBuf::from(home),
        None => dirs::home_dir()?.join(".bpkio"),
    };
    Some(home.join(TENANTS_FILE_NAME))
}

fn list_tenants() -> Vec<String> {
    tenants_file_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|contents| parse_tenant_labels(&contents))
        .unwrap_or_default()
}

/// Returns the `[section]` names of an INI file, in order.
///
/// Only section headers are read: the file also holds credentials, which must never be
/// retained or logged.
fn parse_tenant_labels(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let header = line.strip_prefix('[')?;
            let label = header[..header.rfind(']')?].trim();
            (!label.is_empty()).then(|| label.to_string())
        })
        .collect()
}

#[cfg(test)]
#[path = "tenant_popup_tests.rs"]
mod tests;
