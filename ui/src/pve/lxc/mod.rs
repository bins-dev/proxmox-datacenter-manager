mod overview;
use overview::LxcOverviewPanel;

use std::rc::Rc;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Fa, Row, TabBarItem, TabPanel};

use pdm_api_types::resource::PveLxcResource;

use crate::pve::utils::render_lxc_name;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct LxcPanel {
    remote: String,
    node: String,
    info: PveLxcResource,

    #[prop_or(60_000)]
    /// The interval for refreshing the rrd data
    pub rrd_interval: u32,

    #[prop_or(10_000)]
    /// The interval for refreshing the status data
    pub status_interval: u32,
}

impl LxcPanel {
    pub fn new(remote: String, node: String, info: PveLxcResource) -> Self {
        yew::props!(Self { remote, node, info })
    }
}

pub struct LxcPanelComp {}

impl yew::Component for LxcPanelComp {
    type Message = ();
    type Properties = LxcPanel;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();

        let title: Html = Row::new()
            .gap(2)
            .class(pwt::css::AlignItems::Baseline)
            .with_child(Fa::new("cube"))
            .with_child(tr! {"CT {0}", render_lxc_name(&props.info, true)})
            .into();

        TabPanel::new()
            .class(pwt::css::FlexFit)
            .title(title)
            .with_item_builder(
                TabBarItem::new()
                    .key("status_view")
                    .label(tr!("Overview"))
                    .icon_class("fa fa-tachometer"),
                {
                    let remote = props.remote.clone();
                    let node = props.node.clone();
                    let info = props.info.clone();
                    move |_| {
                        LxcOverviewPanel::new(remote.clone(), node.clone(), info.clone()).into()
                    }
                },
            )
            .into()
    }
}

impl Into<VNode> for LxcPanel {
    fn into(self) -> VNode {
        VComp::new::<LxcPanelComp>(Rc::new(self), None).into()
    }
}
