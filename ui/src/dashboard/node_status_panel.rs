use std::rc::Rc;

use pdm_api_types::remotes::RemoteType;
use pdm_api_types::resource::NodeStatusCount;
use pdm_search::{Search, SearchTerm};
use proxmox_yew_comp::Status;
use pwt::{
    css::{AlignItems, FlexFit, JustifyContent},
    prelude::*,
    widget::{Column, Fa},
};
use yew::{
    virtual_dom::{VComp, VNode},
    Properties,
};

use crate::search_provider::get_search_provider;

use super::loading_column;

#[derive(PartialEq, Clone, Properties)]
pub struct NodeStatusPanel {
    remote_type: RemoteType,
    status: Option<NodeStatusCount>,
    failed_remotes: usize,
}

impl NodeStatusPanel {
    pub fn new(
        remote_type: RemoteType,
        status: Option<NodeStatusCount>,
        failed_remotes: usize,
    ) -> Self {
        yew::props!(Self {
            remote_type,
            status,
            failed_remotes,
        })
    }
}

impl From<NodeStatusPanel> for VNode {
    fn from(value: NodeStatusPanel) -> Self {
        let comp = VComp::new::<NodeStatusPanelComponent>(Rc::new(value), None);
        VNode::from(comp)
    }
}

pub struct NodeStatusPanelComponent {}

impl yew::Component for NodeStatusPanelComponent {
    type Message = Search;
    type Properties = NodeStatusPanel;

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

        let (icon, status_msg, search_terms) = match &props.status {
            Some(status) => map_status(status, props.remote_type, props.failed_remotes),
            None => return loading_column().into(),
        };

        let column = Column::new()
            .padding(4)
            .class("pwt-pointer")
            .class(FlexFit)
            .class(AlignItems::Center)
            .class(JustifyContent::Center)
            .gap(2)
            .onclick(ctx.link().callback({
                let search_terms = search_terms.clone();
                move |_| Search::with_terms(search_terms.clone())
            }))
            .onkeydown(ctx.link().batch_callback({
                let search_terms = search_terms.clone();
                move |event: KeyboardEvent| match event.key().as_str() {
                    "Enter" | " " => Some(Search::with_terms(search_terms.clone())),
                    _ => None,
                }
            }))
            .with_child(icon.large_4x())
            .with_child(status_msg);
        column.into()
    }
}

fn map_status(
    status: &NodeStatusCount,
    remote_type: RemoteType,
    failed_remotes: usize,
) -> (Fa, String, Vec<SearchTerm>) {
    let mut search_terms = vec![
        SearchTerm::new("node").category(Some("type")),
        SearchTerm::new(remote_type.to_string()).category(Some("remote-type")),
    ];
    let (icon, status_msg) = match status {
        NodeStatusCount {
            online,
            offline,
            unknown,
        } if *offline > 0 => {
            search_terms.push(SearchTerm::new("offline").category(Some("status")));
            (
                Status::Error.into(),
                tr!(
                    "{0} of {1} nodes are offline",
                    offline,
                    online + offline + unknown,
                ),
            )
        }
        NodeStatusCount { unknown, .. } if *unknown > 0 => {
            search_terms.push(SearchTerm::new("unknown").category(Some("status")));
            (
                Status::Warning.into(),
                tr!("{0} nodes have an unknown status", unknown),
            )
        }
        NodeStatusCount { online, .. } if failed_remotes > 0 => match remote_type {
            RemoteType::Pve => (
                Status::Unknown.into(),
                tr!("{0} of an unknown number of nodes online", online),
            ),
            RemoteType::Pbs => (
                Status::Error.into(),
                tr!("{0} remotes failed", failed_remotes),
            ),
        },
        NodeStatusCount { online, .. } => (Status::Success.into(), tr!("{0} nodes online", online)),
    };

    (icon, status_msg, search_terms)
}
