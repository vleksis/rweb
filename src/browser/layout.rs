use std::ops::Index;
use std::ops::IndexMut;

use crate::browser::display::CssPx;
use crate::browser::display::DisplayItem;
use crate::browser::display::MARGIN;
use crate::browser::display::TextStyle;
use crate::browser::display::VSTEP;
use crate::browser::font;
use crate::html::Document;
use crate::html::DomId;
use crate::html::NodeView;
use crate::html::Tag;

#[derive(Debug)]
pub(super) struct Layout {
    width: CssPx,
    height: CssPx,
    display_list: Vec<DisplayItem>,
}

impl Layout {
    pub(super) fn empty() -> Self {
        Self {
            width: 0.0,
            height: VSTEP,
            display_list: Vec::new(),
        }
    }

    pub(super) fn build(document: &Document, viewport_width: CssPx) -> Self {
        LayoutBuilder::new(document, viewport_width).build()
    }

    pub(super) fn width(&self) -> CssPx {
        self.width
    }

    pub(super) fn display_list(&self) -> &[DisplayItem] {
        &self.display_list
    }

    pub(super) fn height(&self) -> CssPx {
        self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LayoutId(usize);

#[derive(Debug)]
struct LayoutTree {
    nodes: Vec<LayoutNode>,
}

impl LayoutTree {
    fn new(root: DomId) -> Self {
        Self {
            nodes: vec![LayoutNode::root(root)],
        }
    }

    fn root(&self) -> LayoutId {
        LayoutId(0)
    }

    fn push(&mut self, parent: LayoutId, dom: DomId) -> LayoutId {
        let id = LayoutId(self.nodes.len());
        let previous = self[parent].children.last().copied();

        self.nodes
            .push(LayoutNode::new(dom, Some(parent), previous));
        self[parent].children.push(id);

        id
    }

    fn children(&self, parent: LayoutId) -> &[LayoutId] {
        &self[parent].children
    }
}

impl Index<LayoutId> for LayoutTree {
    type Output = LayoutNode;

    fn index(&self, index: LayoutId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl IndexMut<LayoutId> for LayoutTree {
    fn index_mut(&mut self, index: LayoutId) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

#[derive(Debug)]
struct LayoutNode {
    dom: DomId,

    parent: Option<LayoutId>,
    previous: Option<LayoutId>,
    children: Vec<LayoutId>,

    x: CssPx,
    y: CssPx,
    width: CssPx,
    height: CssPx,

    display_list: Vec<DisplayItem>,
}

impl LayoutNode {
    fn root(dom: DomId) -> Self {
        Self::new(dom, None, None)
    }

    fn new(dom: DomId, parent: Option<LayoutId>, previous: Option<LayoutId>) -> Self {
        Self {
            dom,
            parent,
            previous,
            children: Vec::new(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            display_list: Vec::new(),
        }
    }

    fn set_containing_block(&mut self, x: CssPx, y: CssPx, width: CssPx) {
        self.x = x;
        self.y = y;
        self.width = width;
    }

    fn bottom(&self) -> CssPx {
        self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutMode {
    Block,
    Inline,
}

fn layout_mode(document: &Document, dom: DomId) -> LayoutMode {
    match document.view(dom) {
        NodeView::Text(_) => LayoutMode::Inline,
        NodeView::Document { children } | NodeView::Tag { children, .. } => {
            if has_block_child(document, children) {
                LayoutMode::Block
            } else {
                LayoutMode::Inline
            }
        }
    }
}

fn has_block_child(document: &Document, children: &[DomId]) -> bool {
    children
        .iter()
        .copied()
        .filter(|child| should_create_layout_node(document, *child))
        .any(|child| is_block_node(document, child))
}

fn should_create_layout_node(document: &Document, dom: DomId) -> bool {
    match document.view(dom) {
        NodeView::Document { children } => children
            .iter()
            .any(|child| should_create_layout_node(document, *child)),
        NodeView::Tag { tag, .. } => !tag.is_hidden(),
        NodeView::Text(text) => !text.trim().is_empty(),
    }
}

fn is_block_node(document: &Document, dom: DomId) -> bool {
    if let NodeView::Tag { tag, .. } = document.view(dom) {
        tag.is_block()
    } else {
        false
    }
}

struct LayoutBuilder<'a> {
    document: &'a Document,
    viewport_width: CssPx,
    tree: LayoutTree,
}

impl<'a> LayoutBuilder<'a> {
    fn new(document: &'a Document, viewport_width: CssPx) -> Self {
        let root = document.root();

        Self {
            document,
            viewport_width,
            tree: LayoutTree::new(root),
        }
    }

    fn build(mut self) -> Layout {
        self.build_children(self.tree.root());
        self.layout_tree();

        let width = self.viewport_width;
        let display_list = self.paint();
        let height = self.height();

        Layout {
            width,
            height,
            display_list,
        }
    }

    fn build_node(&mut self, parent: LayoutId, dom: DomId) {
        if !should_create_layout_node(self.document, dom) {
            return;
        }

        let layout = self.tree.push(parent, dom);
        self.build_children(layout);
    }

    fn build_children(&mut self, parent: LayoutId) {
        let dom = self.tree[parent].dom;

        if layout_mode(self.document, dom) != LayoutMode::Block {
            return;
        }

        let children = self.document.children(dom).to_vec();
        for child in children {
            self.build_node(parent, child);
        }
    }

    fn layout_tree(&mut self) {
        let root = self.tree.root();
        let width = (self.viewport_width - 2.0 * MARGIN).max(0.0);

        self.tree[root].set_containing_block(MARGIN, MARGIN, width);
        self.layout_contents(root);
    }

    fn layout_node(&mut self, id: LayoutId) {
        let Some(parent) = self.tree[id].parent else {
            return;
        };

        let x = self.tree[parent].x;
        let y = self.tree[id]
            .previous
            .map(|previous| self.tree[previous].bottom())
            .unwrap_or(self.tree[parent].y);
        let width = self.tree[parent].width;

        self.tree[id].set_containing_block(x, y, width);
        self.layout_contents(id);
    }

    fn layout_contents(&mut self, id: LayoutId) {
        let dom = self.tree[id].dom;

        match layout_mode(self.document, dom) {
            LayoutMode::Block => self.layout_block_children(id),
            LayoutMode::Inline => self.layout_inline_contents(id),
        }
    }

    fn layout_block_children(&mut self, id: LayoutId) {
        let children = self.tree.children(id).to_vec();

        for child in &children {
            self.layout_node(*child);
        }

        self.tree[id].height = self.children_height(id, &children);
    }

    fn layout_inline_contents(&mut self, id: LayoutId) {
        let mut layout = InlineLayout::new(self.tree[id].x, self.tree[id].y, self.tree[id].width);
        let dom = self.tree[id].dom;
        layout.node(self.document, dom);

        let (display_list, height) = layout.finish();
        self.tree[id].display_list = display_list;
        self.tree[id].height = height;
    }

    fn children_height(&self, parent: LayoutId, children: &[LayoutId]) -> CssPx {
        children
            .last()
            .map(|last| self.tree[*last].bottom() - self.tree[parent].y)
            .unwrap_or(0.0)
    }

    fn paint(&self) -> Vec<DisplayItem> {
        let mut display_list = Vec::new();
        self.paint_node(self.tree.root(), &mut display_list);
        display_list
    }

    fn paint_node(&self, id: LayoutId, display_list: &mut Vec<DisplayItem>) {
        display_list.extend(self.tree[id].display_list.iter().cloned());

        for child in &self.tree[id].children {
            self.paint_node(*child, display_list);
        }
    }

    fn height(&self) -> CssPx {
        let root = &self.tree[self.tree.root()];

        (root.bottom() + MARGIN).max(VSTEP)
    }
}

#[derive(Debug)]
struct InlineLayout {
    display_list: Vec<DisplayItem>,
    line: Vec<LineItem>,
    styles: Vec<TextStyle>,
    origin_x: CssPx,
    origin_y: CssPx,
    width: CssPx,
    x: CssPx,
    y: CssPx,
    pending_space: bool,
}

impl InlineLayout {
    fn new(origin_x: CssPx, origin_y: CssPx, width: CssPx) -> Self {
        Self {
            display_list: Vec::new(),
            line: Vec::new(),
            styles: vec![TextStyle::default()],
            origin_x,
            origin_y,
            width,
            x: 0.0,
            y: 0.0,
            pending_space: false,
        }
    }

    fn node(&mut self, document: &Document, node: DomId) {
        match document.view(node) {
            NodeView::Document { children } => {
                for child in children {
                    self.node(document, *child);
                }
            }

            NodeView::Tag {
                tag,
                attributes: _,
                children,
            } => {
                if tag.is_hidden() {
                    return;
                }

                self.open_tag(tag);
                for child in children {
                    self.node(document, *child);
                }
                self.close_tag(tag);
            }

            NodeView::Text(text) => {
                self.text(text);
            }
        }
    }

    fn text(&mut self, text: &str) {
        let mut word = String::new();

        for c in text.chars() {
            if c.is_whitespace() {
                self.flush_word(&mut word);
                self.pending_space = true;
            } else {
                word.push(c);
            }
        }

        self.flush_word(&mut word);
    }

    fn flush_word(&mut self, word: &mut String) {
        if !word.is_empty() {
            self.word(std::mem::take(word));
        }
    }

    fn word(&mut self, word: String) {
        let style = self.style();
        let space_width = font::measure_text(" ", style);
        let word_width = font::measure_text(&word, style);
        let word_x = self.next_word_x(space_width);

        if word_x + word_width > self.width {
            self.flush();
        }

        let word_x = self.next_word_x(space_width);

        self.line.push(LineItem {
            x: word_x,
            text: word,
            style,
        });
        self.x = word_x + word_width;
        self.pending_space = false;
    }

    fn next_word_x(&self, space_width: CssPx) -> CssPx {
        if self.pending_space && !self.line.is_empty() {
            self.x + space_width
        } else {
            self.x
        }
    }

    fn open_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Br => self.flush(),
            Tag::B | Tag::Strong => self.push_style(TextStyle::bold),
            Tag::I | Tag::Em => self.push_style(TextStyle::italic),
            Tag::Small => self.push_style(TextStyle::smaller),
            Tag::Big => self.push_style(TextStyle::larger),
            _ => {}
        }
    }

    fn close_tag(&mut self, tag: Tag) {
        match tag {
            Tag::P => {
                self.flush();
                self.y += VSTEP;
            }

            Tag::B | Tag::Strong | Tag::I | Tag::Em | Tag::Small | Tag::Big => self.pop_style(),
            _ => {}
        }
    }

    fn style(&self) -> TextStyle {
        self.styles.last().copied().unwrap_or_default()
    }

    fn push_style(&mut self, update: fn(&mut TextStyle)) {
        let mut style = self.style();
        update(&mut style);
        self.styles.push(style);
    }

    fn pop_style(&mut self) {
        if self.styles.len() > 1 {
            self.styles.pop();
        }
    }

    fn flush(&mut self) {
        if self.line.is_empty() {
            return;
        }

        let max_ascent = self
            .line
            .iter()
            .map(|item| font::font_metrics(item.style).ascent)
            .fold(0.0, CssPx::max);
        let baseline = self.y + 1.25 * max_ascent;
        let max_descent = self
            .line
            .iter()
            .map(|item| font::font_metrics(item.style).descent)
            .fold(0.0, CssPx::max);

        for item in self.line.drain(..) {
            let metrics = font::font_metrics(item.style);
            self.display_list.push(DisplayItem {
                x: self.origin_x + item.x,
                y: self.origin_y + baseline - metrics.ascent,
                text: item.text,
                style: item.style,
            });
        }

        self.x = 0.0;
        self.y = baseline + 1.25 * max_descent;
        self.pending_space = false;
    }

    fn finish(mut self) -> (Vec<DisplayItem>, CssPx) {
        self.flush();

        (self.display_list, self.y)
    }
}

#[derive(Debug)]
struct LineItem {
    x: CssPx,
    text: String,
    style: TextStyle,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::FontSlant;
    use crate::browser::FontWeight;
    use crate::html;

    fn layout_html(source: &str) -> Layout {
        let document = html::parse(source.to_string()).unwrap();

        Layout::build(&document, 800.0)
    }

    #[test]
    fn layout_emits_words_not_characters() {
        let layout = layout_html("hello world");

        assert_eq!(layout.display_list.len(), 2);
        assert_eq!(layout.display_list[0].text, "hello");
        assert_eq!(layout.display_list[1].text, "world");
    }

    #[test]
    fn layout_applies_bold_tag() {
        let layout = layout_html("<b>hello</b>");

        assert_eq!(layout.display_list[0].style.weight, FontWeight::Bold);
    }

    #[test]
    fn whitespace_between_inline_nodes_is_preserved() {
        let layout = layout_html("hello <em>world</em>");
        let hello = layout
            .display_list
            .iter()
            .find(|item| item.text == "hello")
            .unwrap();
        let world = layout
            .display_list
            .iter()
            .find(|item| item.text == "world")
            .unwrap();
        let hello_end = hello.x + font::measure_text(&hello.text, hello.style);
        let space = font::measure_text(" ", hello.style);

        assert!((world.x - hello_end - space).abs() < 0.5);
    }

    #[test]
    fn nested_same_style_tags_restore_previous_style() {
        let layout = layout_html("<b>outer <b>inner</b> outer</b>");
        let outer = layout
            .display_list
            .iter()
            .filter(|item| item.text == "outer")
            .collect::<Vec<_>>();
        let inner = layout
            .display_list
            .iter()
            .find(|item| item.text == "inner")
            .unwrap();

        assert_eq!(outer.len(), 2);
        assert!(
            outer
                .iter()
                .all(|item| item.style.weight == FontWeight::Bold)
        );
        assert_eq!(inner.style.weight, FontWeight::Bold);
    }

    #[test]
    fn punctuation_after_inline_tag_does_not_get_extra_space() {
        let layout = layout_html("What is a <em>font</em>, exactly?");

        let font_item = layout
            .display_list
            .iter()
            .find(|item| item.text == "font")
            .unwrap();
        let comma = layout
            .display_list
            .iter()
            .find(|item| item.text == ",")
            .unwrap();
        let font_end = font_item.x + font::measure_text(&font_item.text, font_item.style);

        assert_eq!(font_item.style.slant, FontSlant::Italic);
        assert!((comma.x - font_end).abs() < 0.5);
    }

    #[test]
    fn block_children_stack_vertically() {
        let layout = layout_html("<div><p>first</p><p>second</p></div>");
        let first = layout
            .display_list
            .iter()
            .find(|item| item.text == "first")
            .unwrap();
        let second = layout
            .display_list
            .iter()
            .find(|item| item.text == "second")
            .unwrap();

        assert!(second.y > first.y);
    }

    #[test]
    fn head_content_is_hidden() {
        let layout =
            layout_html("<html><head><title>hidden</title></head><body>visible</body></html>");

        assert!(
            layout
                .display_list
                .iter()
                .any(|item| item.text == "visible")
        );
        assert!(!layout.display_list.iter().any(|item| item.text == "hidden"));
    }
}
