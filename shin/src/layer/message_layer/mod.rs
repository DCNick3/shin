mod font_atlas;
mod message;
mod messagebox;

use std::sync::Arc;

use glam::{vec2, Mat4};
use message::{Message, MessageStatus};
use shin_core::{
    time::Ticks,
    vm::command::types::{MessageboxStyle, MessageboxType},
};
use shin_render::{render_pass::RenderPass, PassKind};

use crate::{
    adv::assets::AdvFonts,
    layer::{
        message_layer::{font_atlas::FontAtlas, messagebox::Messagebox},
        properties::LayerProperties,
        render_params::TransformParams,
        DrawableLayer, Layer,
    },
    render::overlay::{OverlayCollector, OverlayVisitable},
    update::{AdvUpdatable, AdvUpdateContext, Updatable, UpdateContext},
};

#[derive(Clone)]
pub struct MessageLayer {
    props: LayerProperties,
    style: MessageboxStyle,
    // font_atlas: Arc<FontAtlas>,
    // message: Option<Message>,
    // messagebox: Messagebox,
}

impl MessageLayer {
    pub fn new() -> Self {
        Self {
            props: LayerProperties::new(),
            style: MessageboxStyle::default(),
            // font_atlas: Arc::new(FontAtlas::new(device, queue, fonts.medium_font)),
            // message: None,
            // messagebox: Messagebox::new(textures),
        }
    }

    pub fn set_style(&mut self, style: MessageboxStyle) {
        self.style = style;

        // self.messagebox.set_messagebox_type(style.messagebox_type);
    }

    pub fn set_message(&mut self, context: &UpdateContext, text: &str) {
        todo!()

        // self.messagebox.set_visible(true);
        //
        // // TODO: devise a better positioning scheme maybe?
        // let (base_position, show_character_name) = match self.style.messagebox_type {
        //     MessageboxType::Neutral
        //     | MessageboxType::WitchSpace
        //     | MessageboxType::Ushiromiya
        //     | MessageboxType::Transparent => (vec2(-740.0 - 10.0, 300.0 - 156.0), true),
        //     MessageboxType::Novel => (vec2(-740.0 - 10.0, 300.0 - 156.0 - 450.0), false),
        //     MessageboxType::NoText => {
        //         todo!()
        //     }
        // };
        //
        // let message = Message::new(
        //     context,
        //     self.font_atlas.clone(),
        //     base_position,
        //     show_character_name,
        //     text,
        // );
        //
        // self.messagebox.set_metrics(message.metrics());
        // self.message = Some(message);
    }

    pub fn close(&mut self) {
        // self.message = None;
        // self.messagebox.set_visible(false);
    }

    pub fn is_finished(&self) -> bool {
        todo!()
        // self.message
        //     .as_ref()
        //     .map(|m| m.is_complete())
        //     .unwrap_or(true)
    }

    pub fn is_section_finished(&self, section_num: u32) -> bool {
        todo!()
        // self.message
        //     .as_ref()
        //     .map(|m| m.sent_signals() > section_num)
        //     .expect("MessageLayer::is_section_finished called when no message is set")
    }

    pub fn signal(&mut self) {
        todo!()
        // if let Some(message) = self.message.as_mut() {
        //     message.signal();
        // }
    }

    pub fn advance(&mut self) {
        todo!()
        // if let Some(m) = self.message.as_mut() {
        //     m.advance()
        // }
    }

    pub fn fast_forward(&mut self) {
        todo!()
        // if let Some(m) = self.message.as_mut() {
        //     m.fast_forward()
        // }
    }
}

impl AdvUpdatable for MessageLayer {
    fn update(&mut self, context: &AdvUpdateContext) {
        self.props.update(context)
        // self.messagebox.update(ctx);
        // if let Some(message) = &mut self.message {
        //     message.update(ctx);
        // }
    }
}

impl Layer for MessageLayer {
    fn render(
        &self,
        _pass: &mut RenderPass,
        _transform: &TransformParams,
        _stencil_ref: u8,
        _pass_kind: PassKind,
    ) {
        // TODO
    }
}

impl DrawableLayer for MessageLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}

// impl OverlayVisitable for MessageLayer {
//     fn visit_overlay(&self, collector: &mut OverlayCollector) {
//         collector.subgroup(
//             "Message Layer",
//             |collector| {
//                 collector.overlay(
//                     "Status",
//                     |_ctx, top_left| {
//                         let status = match self.message {
//                             None => "N",
//                             Some(ref m) => match m.status() {
//                                 MessageStatus::Printing => "P",
//                                 MessageStatus::ClickWaiting => "K",
//                                 MessageStatus::SignalWaiting => "Y",
//                                 MessageStatus::Complete => "C",
//                             },
//                         };
//                         let blocks = self
//                             .message
//                             .as_ref()
//                             .map(|m| m.completed_blocks())
//                             .unwrap_or(0);
//                         let signalled_out =
//                             self.message.as_ref().map(|m| m.sent_signals()).unwrap_or(0);
//                         let signalled_in = self
//                             .message
//                             .as_ref()
//                             .map(|m| m.received_signals())
//                             .unwrap_or(0);
//                         let time = self
//                             .message
//                             .as_ref()
//                             .map(|v| v.time())
//                             .unwrap_or(Ticks::ZERO);
//
//                         top_left.label(format!(
//                             "MessageLayer: {} B={} So={} Si={} T={:06.1} AF={:04.1}%",
//                             status,
//                             blocks,
//                             signalled_out,
//                             signalled_in,
//                             time,
//                             self.font_atlas.free_space() * 100.0,
//                         ));
//                     },
//                     true,
//                 );
//                 self.font_atlas.visit_overlay(collector);
//             },
//             true,
//         );
//     }
// }
