use std::rc::Rc;

use pdm_api_types::resource::{GuestStatusCount, ResourceType};
use pdm_search::{Search, SearchTerm};
use proxmox_yew_comp::GuestState;
use pwt::{
    css::{self, TextAlign},
    prelude::*,
    widget::{Container, Fa, List, ListTile},
};
use yew::{
    virtual_dom::{VComp, VNode},
    Properties,
};

use crate::{pve::GuestType, search_provider::get_search_provider};

use super::loading_column;

#[derive(PartialEq, Clone, Properties)]
pub struct GuestPanel {
    guest_type: GuestType,
    status: Option<GuestStatusCount>,
}

impl GuestPanel {
    pub fn new(guest_type: GuestType, status: Option<GuestStatusCount>) -> Self {
        yew::props!(Self { guest_type, status })
    }
}

impl From<GuestPanel> for VNode {
    fn from(value: GuestPanel) -> Self {
        let comp = VComp::new::<PdmGuestPanel>(Rc::new(value), None);
        VNode::from(comp)
    }
}

#[derive(PartialEq, Clone)]
pub enum StatusRow {
    State(GuestState, u64),
    All(u64),
}

pub struct PdmGuestPanel {}

impl yew::Component for PdmGuestPanel {
    type Message = Search;
    type Properties = GuestPanel;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        if let Some(provider) = get_search_provider(ctx) {
            provider.search(msg);
        }
        false
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();
        let guest_type = props.guest_type;
        let status = match &props.status {
            Some(status) => status,
            None => return loading_column().into(),
        };

        let data = vec![
            StatusRow::State(GuestState::Running, status.running),
            StatusRow::State(GuestState::Stopped, status.stopped),
            StatusRow::State(GuestState::Template, status.template),
            StatusRow::State(GuestState::Unknown, status.unknown),
            StatusRow::All(status.running + status.stopped + status.template + status.unknown),
        ];

        let tiles: Vec<_> = data
            .into_iter()
            .filter_map(|row| create_list_tile(ctx.link(), guest_type, row))
            .collect();

        let list = List::new(tiles.len() as u64, move |idx: u64| {
            tiles[idx as usize].clone()
        })
        .padding(4)
        .class(css::Flex::Fill)
        .grid_template_columns("auto auto 1fr auto");

        list.into()
    }
}

fn create_list_tile(
    link: &html::Scope<PdmGuestPanel>,
    guest_type: GuestType,
    status_row: StatusRow,
) -> Option<ListTile> {
    let (icon, text, count, status, template) = match status_row {
        StatusRow::State(guest_state, count) => match guest_state {
            GuestState::Template | GuestState::Unknown if count == 0 => return None,
            GuestState::Paused => return None,
            GuestState::Running => (
                Fa::from(guest_state),
                tr!("running"),
                count,
                Some("running"),
                Some(false),
            ),
            GuestState::Stopped => (
                Fa::from(guest_state),
                tr!("stopped"),
                count,
                Some("stopped"),
                Some(false),
            ),
            GuestState::Template => (
                Fa::from(guest_state),
                tr!("Template"),
                count,
                None,
                Some(true),
            ),
            GuestState::Unknown => (
                Fa::from(guest_state),
                tr!("Unknown"),
                count,
                Some("unknown"),
                None,
            ),
        },
        StatusRow::All(count) => (Fa::from(guest_type), tr!("All"), count, None, None),
    };

    Some(
        ListTile::new()
            .tabindex(0)
            .interactive(true)
            .with_child(icon)
            .with_child(Container::new().padding_x(2).with_child(text))
            .with_child(
                Container::new()
                    .class(TextAlign::Right)
                    // FIXME: replace with `column_gap` to `List` when implemented
                    .padding_end(2)
                    .with_child(count),
            )
            .with_child(Fa::new("search"))
            // FIXME: repalce with on_activate for `ListTile` when implemented
            .onclick(link.callback(move |_| create_guest_search_term(guest_type, status, template)))
            .onkeydown(link.batch_callback(
                move |event: KeyboardEvent| match event.key().as_str() {
                    "Enter" | " " => Some(create_guest_search_term(guest_type, status, template)),
                    _ => None,
                },
            )),
    )
}

fn create_guest_search_term(
    guest_type: GuestType,
    status: Option<&'static str>,
    template: Option<bool>,
) -> Search {
    let resource_type: ResourceType = guest_type.into();
    if status.is_none() && template.is_none() {
        return Search::with_terms(vec![
            SearchTerm::new(resource_type.as_str()).category(Some("type"))
        ]);
    }

    let mut terms = vec![SearchTerm::new(resource_type.as_str()).category(Some("type"))];

    if let Some(template) = template {
        terms.push(SearchTerm::new(template.to_string()).category(Some("template")));
    }
    if let Some(status) = status {
        terms.push(SearchTerm::new(status).category(Some("status")));
    }
    Search::with_terms(terms)
}
