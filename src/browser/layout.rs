use std::ops::Index;
use std::ops::IndexMut;

use crate::browser::display::CssPx;
use crate::browser::display::DisplayItem;
use crate::browser::display::MARGIN;
use crate::browser::display::Rect;
use crate::browser::display::TextItem;
use crate::browser::display::TextStyle;
use crate::browser::display::VSTEP;
use crate::browser::font;
use crate::html::Document;
use crate::html::DomId;
use crate::html::NodeView;
use crate::html::Tag;

const LIST_INDENT: CssPx = 32.0;
const BULLET_SIZE: CssPx = 7.0;
const BULLET_GAP: CssPx = 8.0;

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

    fn push(&mut self, parent: LayoutId, kind: LayoutBox) -> LayoutId {
        let id = LayoutId(self.nodes.len());
        let previous = self[parent].children.last().copied();

        self.nodes
            .push(LayoutNode::new(kind, Some(parent), previous));
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

#[derive(Debug, Clone, Copy)]
enum LayoutBox {
    Block(DomId),
    Inline(DomId),
    AnonymousBlock,
}

#[derive(Debug)]
struct LayoutNode {
    kind: LayoutBox,

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
        Self::new(LayoutBox::Block(dom), None, None)
    }

    fn new(kind: LayoutBox, parent: Option<LayoutId>, previous: Option<LayoutId>) -> Self {
        Self {
            kind,
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

fn should_create_layout_node(document: &Document, dom: DomId) -> bool {
    match document.view(dom) {
        NodeView::Document { children } => children
            .iter()
            .any(|child| should_create_layout_node(document, *child)),
        NodeView::Tag { tag, .. } => !tag.is_hidden(),
        NodeView::Text(text) => !text.is_empty(),
    }
}

fn is_whitespace_text(document: &Document, dom: DomId) -> bool {
    matches!(document.view(dom), NodeView::Text(text) if text.trim().is_empty())
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

    fn build_children(&mut self, parent: LayoutId) {
        let LayoutBox::Block(dom_parent) = self.tree[parent].kind else {
            return;
        };

        let children = self
            .document
            .children(dom_parent)
            .iter()
            .copied()
            .filter(|child| should_create_layout_node(self.document, *child))
            .collect::<Vec<_>>();

        if children
            .iter()
            .copied()
            .all(|child| !is_block_node(self.document, child))
        {
            for child in children.iter().copied() {
                self.build_inline_node(parent, child);
            }
            return;
        }

        let mut inline_run = Vec::new();

        for child in children {
            if is_block_node(self.document, child) {
                self.flush_anonymous_block(parent, &mut inline_run);

                let layout = self.tree.push(parent, LayoutBox::Block(child));
                self.build_children(layout);
            } else {
                inline_run.push(child);
            }
        }

        self.flush_anonymous_block(parent, &mut inline_run);
    }

    fn flush_anonymous_block(&mut self, parent: LayoutId, inline_run: &mut Vec<DomId>) {
        if inline_run
            .iter()
            .all(|dom| is_whitespace_text(self.document, *dom))
        {
            inline_run.clear();
            return;
        }

        let anonymous = self.tree.push(parent, LayoutBox::AnonymousBlock);
        for child in inline_run.drain(..) {
            self.build_inline_node(anonymous, child);
        }
    }

    fn build_inline_node(&mut self, parent: LayoutId, dom: DomId) {
        if !should_create_layout_node(self.document, dom) {
            return;
        }

        let layout = self.tree.push(parent, LayoutBox::Inline(dom));

        for child in self.document.children(dom) {
            self.build_inline_node(layout, *child);
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

        let is_list_item = self.is_list_item(id);
        let indent = if is_list_item { LIST_INDENT } else { 0.0 };
        let x = self.tree[parent].x + indent;
        let y = self.tree[id]
            .previous
            .map(|previous| self.tree[previous].bottom())
            .unwrap_or(self.tree[parent].y);
        let width = (self.tree[parent].width - indent).max(0.0);

        self.tree[id].set_containing_block(x, y, width);
        self.layout_contents(id);

        if is_list_item {
            self.add_list_marker(id);
        }
    }

    fn is_list_item(&self, id: LayoutId) -> bool {
        matches!(self.tree[id].kind, LayoutBox::Block(dom) if self.document.tag(dom) == Some(Tag::Li))
    }

    fn add_list_marker(&mut self, id: LayoutId) {
        let node = &self.tree[id];
        let marker = Rect {
            x: node.x - BULLET_GAP - BULLET_SIZE,
            y: node.y + (VSTEP - BULLET_SIZE) / 2.0,
            width: BULLET_SIZE,
            height: BULLET_SIZE,
        };

        self.tree[id].height = self.tree[id].height.max(VSTEP);
        self.tree[id].display_list.push(DisplayItem::Rect(marker));
    }

    fn layout_contents(&mut self, id: LayoutId) {
        if self.has_inline_children(id) {
            self.layout_inline_contents(id);
        } else {
            self.layout_block_children(id);
        }

        if matches!(self.tree[id].kind, LayoutBox::Block(dom) if self.document.tag(dom) == Some(Tag::P))
        {
            self.tree[id].height += VSTEP;
        }
    }

    fn has_inline_children(&self, id: LayoutId) -> bool {
        // Construction should guarantee that children are
        // either all inline or all block-level.
        let child = self
            .tree
            .children(id)
            .first()
            .map(|child| self.tree[*child].kind);

        matches!(child, Some(LayoutBox::Inline(_)))
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
        for child in self.tree.children(id) {
            layout.node(self.document, &self.tree, *child);
        }

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

    fn node(&mut self, document: &Document, tree: &LayoutTree, node: LayoutId) {
        let LayoutBox::Inline(dom) = tree[node].kind else {
            return;
        };

        match document.view(dom) {
            NodeView::Document { .. } => {
                for child in tree.children(node) {
                    self.node(document, tree, *child);
                }
            }

            NodeView::Tag {
                tag,
                attributes: _,
                children: _,
            } => {
                self.open_tag(tag);
                for child in tree.children(node) {
                    self.node(document, tree, *child);
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
            self.display_list.push(DisplayItem::Text(TextItem {
                x: self.origin_x + item.x,
                y: self.origin_y + baseline - metrics.ascent,
                text: item.text,
                style: item.style,
            }));
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

    fn text_items(layout: &Layout) -> impl Iterator<Item = &TextItem> {
        layout.display_list.iter().filter_map(|item| match item {
            DisplayItem::Text(item) => Some(item),
            DisplayItem::Rect(_) => None,
        })
    }

    fn text_item<'a>(layout: &'a Layout, text: &str) -> &'a TextItem {
        text_items(layout).find(|item| item.text == text).unwrap()
    }

    fn rect_items(layout: &Layout) -> impl Iterator<Item = &Rect> {
        layout.display_list.iter().filter_map(|item| match item {
            DisplayItem::Text(_) => None,
            DisplayItem::Rect(rect) => Some(rect),
        })
    }

    #[test]
    fn layout_emits_words_not_characters() {
        let layout = layout_html("hello world");
        let items = text_items(&layout).collect::<Vec<_>>();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text, "hello");
        assert_eq!(items[1].text, "world");
    }

    #[test]
    fn layout_applies_bold_tag() {
        let layout = layout_html("<b>hello</b>");

        assert_eq!(text_item(&layout, "hello").style.weight, FontWeight::Bold);
    }

    #[test]
    fn whitespace_between_inline_nodes_is_preserved() {
        let layout = layout_html("hello <em>world</em>");
        let hello = text_item(&layout, "hello");
        let world = text_item(&layout, "world");
        let hello_end = hello.x + font::measure_text(&hello.text, hello.style);
        let space = font::measure_text(" ", hello.style);

        assert!((world.x - hello_end - space).abs() < 0.5);
    }

    #[test]
    fn nested_same_style_tags_restore_previous_style() {
        let layout = layout_html("<b>outer <b>inner</b> outer</b>");
        let outer = text_items(&layout)
            .filter(|item| item.text == "outer")
            .collect::<Vec<_>>();
        let inner = text_item(&layout, "inner");

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

        let font_item = text_item(&layout, "font");
        let comma = text_item(&layout, ",");
        let font_end = font_item.x + font::measure_text(&font_item.text, font_item.style);

        assert_eq!(font_item.style.slant, FontSlant::Italic);
        assert!((comma.x - font_end).abs() < 0.5);
    }

    #[test]
    fn block_children_stack_vertically() {
        let layout = layout_html("<div><p>first</p><p>second</p></div>");
        let first = text_item(&layout, "first");
        let second = text_item(&layout, "second");

        assert!(second.y > first.y);
    }

    #[test]
    fn list_items_are_indented_and_have_markers() {
        let layout = layout_html("<p>plain</p><ul><li>first</li><li>second</li></ul>");
        let plain = text_item(&layout, "plain");
        let first = text_item(&layout, "first");
        let second = text_item(&layout, "second");
        let markers = rect_items(&layout).collect::<Vec<_>>();

        assert_eq!(markers.len(), 2);
        assert!(first.x > plain.x);
        assert_eq!(first.x, second.x);
        assert!(markers[0].x + markers[0].width < first.x);
        assert!(markers[1].x + markers[1].width < second.x);
        assert!(markers[1].y > markers[0].y);
    }

    #[test]
    fn nested_list_items_add_indentation() {
        let layout = layout_html("<ul><li>outer<ul><li>inner</li></ul></li></ul>");
        let outer = text_item(&layout, "outer");
        let inner = text_item(&layout, "inner");

        assert!(inner.x > outer.x);
        assert_eq!(rect_items(&layout).count(), 2);
    }

    #[test]
    fn empty_list_item_reserves_marker_height() {
        let layout = layout_html("<ul><li></li><li>next</li></ul>");
        let markers = rect_items(&layout).collect::<Vec<_>>();

        assert_eq!(markers.len(), 2);
        assert!(markers[1].y > markers[0].y);
    }

    #[test]
    fn mixed_children_create_anonymous_block() {
        let document =
            html::parse("<div><i>Hello, </i><b>world!</b><p>So it began...</p></div>".to_string())
                .unwrap();
        let mut builder = LayoutBuilder::new(&document, 800.0);
        let root = builder.tree.root();
        builder.build_children(root);

        let div = builder.tree.children(root)[0];
        let children = builder.tree.children(div);

        assert!(
            matches!(builder.tree[div].kind, LayoutBox::Block(dom) if document.tag(dom) == Some(Tag::Div))
        );
        assert_eq!(children.len(), 2);
        assert!(matches!(
            builder.tree[children[0]].kind,
            LayoutBox::AnonymousBlock
        ));
        assert!(
            matches!(builder.tree[children[1]].kind, LayoutBox::Block(dom) if document.tag(dom) == Some(Tag::P))
        );

        let inline = builder.tree.children(children[0]);
        assert_eq!(inline.len(), 2);
        assert!(
            matches!(builder.tree[inline[0]].kind, LayoutBox::Inline(dom) if document.tag(dom) == Some(Tag::I))
        );
        assert!(
            matches!(builder.tree[inline[1]].kind, LayoutBox::Inline(dom) if document.tag(dom) == Some(Tag::B))
        );
    }

    #[test]
    fn anonymous_block_lays_out_inline_siblings_on_one_line() {
        let layout = layout_html("<div><i>Hello,</i> <b>world!</b><p>So it began...</p></div>");
        let hello = text_item(&layout, "Hello,");
        let world = text_item(&layout, "world!");
        let paragraph = text_item(&layout, "So");
        let hello_baseline = hello.y + font::font_metrics(hello.style).ascent;
        let world_baseline = world.y + font::font_metrics(world.style).ascent;

        assert!((world_baseline - hello_baseline).abs() < 0.5);
        assert!(world.x > hello.x);
        assert!(paragraph.y > hello.y);
    }

    #[test]
    fn block_child_separates_anonymous_inline_runs() {
        let layout = layout_html("<div>before<p>middle</p>after</div>");
        let before = text_item(&layout, "before");
        let middle = text_item(&layout, "middle");
        let after = text_item(&layout, "after");

        assert!(middle.y > before.y);
        assert!(after.y > middle.y);
    }

    #[test]
    fn head_content_is_hidden() {
        let layout =
            layout_html("<html><head><title>hidden</title></head><body>visible</body></html>");

        assert!(text_items(&layout).any(|item| item.text == "visible"));
        assert!(!text_items(&layout).any(|item| item.text == "hidden"));
    }
}
