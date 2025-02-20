use floem::{
    cosmic_text::{
        Attrs, AttrsList, FamilyOwned, LineHeightValue, TextLayout, Weight,
    },
    id::Id,
    peniko::{
        kurbo::{Point, Rect},
        Color,
    },
    reactive::create_effect,
    style::{ComputedStyle, Style},
    taffy::prelude::Node,
    view::{ChangeFlags, View},
    Renderer, ViewContext,
};

enum FocusTextState {
    Text(String),
    FocusColor(Color),
    FocusIndices(Vec<usize>),
}

pub fn focus_text(
    text: impl Fn() -> String + 'static,
    focus_indices: impl Fn() -> Vec<usize> + 'static,
    focus_color: impl Fn() -> Color + 'static,
) -> FocusText {
    let cx = ViewContext::get_current();

    let id = cx.new_id();

    create_effect(cx.scope, move |_| {
        let new_text = text();
        id.update_state(FocusTextState::Text(new_text), false);
    });

    create_effect(cx.scope, move |_| {
        let focus_color = focus_color();
        id.update_state(FocusTextState::FocusColor(focus_color), false);
    });

    create_effect(cx.scope, move |_| {
        let focus_indices = focus_indices();
        id.update_state(FocusTextState::FocusIndices(focus_indices), false);
    });

    FocusText {
        id,
        text: "".to_string(),
        text_layout: None,
        color: None,
        focus_color: Color::default(),
        focus_indices: Vec::new(),
        text_node: None,
        font_size: None,
        font_family: None,
        line_height: None,
        available_text: None,
        available_width: None,
        available_text_layout: None,
    }
}

pub struct FocusText {
    id: Id,
    text: String,
    text_layout: Option<TextLayout>,
    color: Option<Color>,
    focus_color: Color,
    focus_indices: Vec<usize>,
    font_size: Option<f32>,
    font_family: Option<String>,
    line_height: Option<LineHeightValue>,
    text_node: Option<Node>,
    available_text: Option<String>,
    available_width: Option<f32>,
    available_text_layout: Option<TextLayout>,
}

impl FocusText {
    fn set_text_layout(&mut self) {
        let mut attrs = Attrs::new().color(self.color.unwrap_or_default());
        if let Some(font_size) = self.font_size {
            attrs = attrs.font_size(font_size);
        }
        let font_family = self.font_family.as_ref().map(|font_family| {
            let family: Vec<FamilyOwned> =
                FamilyOwned::parse_list(font_family).collect();
            family
        });
        if let Some(font_family) = font_family.as_ref() {
            attrs = attrs.family(font_family);
        }
        if let Some(line_height) = self.line_height {
            attrs = attrs.line_height(line_height);
        }

        let mut attrs_list = AttrsList::new(attrs);

        for &i_start in &self.focus_indices {
            let i_end = self
                .text
                .char_indices()
                .find(|(i, _)| *i == i_start)
                .map(|(_, c)| c.len_utf8() + i_start);
            let i_end = if let Some(i_end) = i_end {
                i_end
            } else {
                continue;
            };
            attrs_list.add_span(
                i_start..i_end,
                attrs.color(self.focus_color).weight(Weight::BOLD),
            );
        }
        let mut text_layout = TextLayout::new();
        text_layout.set_text(&self.text, attrs_list);
        self.text_layout = Some(text_layout);

        if let Some(new_text) = self.available_text.as_ref() {
            let new_text_len = new_text.len();

            let mut attrs = Attrs::new().color(self.color.unwrap_or_default());
            if let Some(font_size) = self.font_size {
                attrs = attrs.font_size(font_size);
            }
            let font_family = self.font_family.as_ref().map(|font_family| {
                let family: Vec<FamilyOwned> =
                    FamilyOwned::parse_list(font_family).collect();
                family
            });
            if let Some(font_family) = font_family.as_ref() {
                attrs = attrs.family(font_family);
            }

            let mut attrs_list = AttrsList::new(attrs);

            for &i_start in &self.focus_indices {
                if i_start + 3 > new_text_len {
                    break;
                }
                let i_end = self
                    .text
                    .char_indices()
                    .find(|(i, _)| *i == i_start)
                    .map(|(_, c)| c.len_utf8() + i_start);
                let i_end = if let Some(i_end) = i_end {
                    i_end
                } else {
                    continue;
                };
                attrs_list.add_span(
                    i_start..i_end,
                    attrs.color(self.focus_color).weight(Weight::BOLD),
                );
            }
            let mut text_layout = TextLayout::new();
            text_layout.set_text(new_text, attrs_list);
            self.available_text_layout = Some(text_layout);
        }
    }
}

impl View for FocusText {
    fn id(&self) -> floem::id::Id {
        self.id
    }

    fn child(&self, _id: floem::id::Id) -> Option<&dyn View> {
        None
    }

    fn child_mut(&mut self, _id: floem::id::Id) -> Option<&mut dyn View> {
        None
    }

    fn children(&self) -> Vec<&dyn View> {
        Vec::new()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn View> {
        Vec::new()
    }

    fn update(
        &mut self,
        cx: &mut floem::context::UpdateCx,
        state: Box<dyn std::any::Any>,
    ) -> floem::view::ChangeFlags {
        if let Ok(state) = state.downcast() {
            match *state {
                FocusTextState::Text(text) => {
                    self.text = text;
                }
                FocusTextState::FocusColor(color) => {
                    self.focus_color = color;
                }
                FocusTextState::FocusIndices(indices) => {
                    self.focus_indices = indices;
                }
            }
            self.set_text_layout();
            cx.request_layout(self.id());
            ChangeFlags::LAYOUT
        } else {
            ChangeFlags::empty()
        }
    }

    fn layout(
        &mut self,
        cx: &mut floem::context::LayoutCx,
    ) -> floem::taffy::prelude::Node {
        cx.layout_node(self.id, true, |cx| {
            if self.font_size != cx.current_font_size()
                || self.font_family.as_deref() != cx.current_font_family()
                || self.line_height != cx.current_line_height()
            {
                self.font_size = cx.current_font_size();
                self.font_family = cx.current_font_family().map(|s| s.to_string());
                self.line_height = cx.current_line_height();
                self.set_text_layout();
            }
            if self.text_layout.is_none() {
                self.set_text_layout();
            }
            let text_layout = self.text_layout.as_ref().unwrap();
            let size = text_layout.size();
            let width = size.width.ceil() as f32;
            let height = size.height as f32;

            if self.text_node.is_none() {
                self.text_node = Some(cx.new_node());
            }
            let text_node = self.text_node.unwrap();

            let style = Style::BASE
                .size_px(width, height)
                .compute(&ComputedStyle::default())
                .to_taffy_style();
            cx.set_style(text_node, style);
            vec![text_node]
        })
    }

    fn compute_layout(&mut self, cx: &mut floem::context::LayoutCx) -> Option<Rect> {
        let text_node = self.text_node.unwrap();
        let layout = cx.layout(text_node).unwrap();
        let text_layout = self.text_layout.as_ref().unwrap();
        let width = text_layout.size().width as f32;
        if width > layout.size.width {
            if self.available_width != Some(layout.size.width) {
                let mut dots_text = TextLayout::new();
                let mut attrs = Attrs::new().color(
                    self.color.unwrap_or_else(|| Color::rgb8(0xf0, 0xf0, 0xea)),
                );
                if let Some(font_size) = self.font_size {
                    attrs = attrs.font_size(font_size);
                }
                let font_family = self.font_family.as_ref().map(|font_family| {
                    let family: Vec<FamilyOwned> =
                        FamilyOwned::parse_list(font_family).collect();
                    family
                });
                if let Some(font_family) = font_family.as_ref() {
                    attrs = attrs.family(font_family);
                }
                dots_text.set_text("...", AttrsList::new(attrs));

                let dots_width = dots_text.size().width as f32;
                let width_left = layout.size.width - dots_width;
                let hit_point =
                    text_layout.hit_point(Point::new(width_left as f64, 0.0));
                let index = hit_point.index;

                let new_text = if index > 0 {
                    format!("{}...", &self.text[..index])
                } else {
                    "".to_string()
                };
                self.available_text = Some(new_text);
                self.available_width = Some(layout.size.width);
                self.set_text_layout();
            }
        } else {
            self.available_text = None;
            self.available_width = None;
            self.available_text_layout = None;
        }

        None
    }

    fn event(
        &mut self,
        _cx: &mut floem::context::EventCx,
        _id_path: Option<&[floem::id::Id]>,
        _event: floem::event::Event,
    ) -> bool {
        false
    }

    fn paint(&mut self, cx: &mut floem::context::PaintCx) {
        if self.color != cx.current_color()
            || self.font_size != cx.current_font_size()
            || self.font_family.as_deref() != cx.current_font_family()
        {
            self.color = cx.current_color();
            self.font_size = cx.current_font_size();
            self.font_family = cx.current_font_family().map(|s| s.to_string());
            self.set_text_layout();
        }
        let text_node = self.text_node.unwrap();
        let location = cx.layout(text_node).unwrap().location;
        let point = Point::new(location.x as f64, location.y as f64);
        if let Some(text_layout) = self.available_text_layout.as_ref() {
            cx.draw_text(text_layout, point);
        } else {
            cx.draw_text(self.text_layout.as_ref().unwrap(), point);
        }
    }
}
