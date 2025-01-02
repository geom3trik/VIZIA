use crate::{events::ViewHandler, prelude::*};
use hashbrown::HashMap;
use vizia_storage::{LayoutParentIterator, TreeBreadthIterator};
use vizia_style::{
    matches_selector,
    selectors::{
        attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint},
        context::{MatchingForInvalidation, NeedsSelectorFlags, SelectorCaches},
        matching::ElementSelectorFlags,
        parser::{Component, NthType},
        OpaqueElement, SelectorImpl,
    },
    Element, MatchingContext, MatchingMode, PseudoClass, QuirksMode, SelectorIdent, Selectors,
};

/// A node used for style matching.
#[derive(Clone)]
pub(crate) struct Node<'s, 't, 'v> {
    entity: Entity,
    store: &'s Style,
    tree: &'t Tree<Entity>,
    views: &'v HashMap<Entity, Box<dyn ViewHandler>>,
}

impl std::fmt::Debug for Node<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.entity)
    }
}

/// Used for selector matching.
impl Element for Node<'_, '_, '_> {
    type Impl = Selectors;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self)
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn parent_element(&self) -> Option<Self> {
        self.tree.get_layout_parent(self.entity).map(|parent| Node {
            entity: parent,
            store: self.store,
            tree: self.tree,
            views: self.views,
        })
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.tree.get_prev_layout_sibling(self.entity).map(|parent| Node {
            entity: parent,
            store: self.store,
            tree: self.tree,
            views: self.views,
        })
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.tree.get_next_layout_sibling(self.entity).map(|parent| Node {
            entity: parent,
            store: self.store,
            tree: self.tree,
            views: self.views,
        })
    }

    fn is_empty(&self) -> bool {
        !self.tree.has_children(self.entity)
    }

    fn is_root(&self) -> bool {
        self.entity == Entity::root()
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &SelectorIdent) -> bool {
        if let Some(element) = self.views.get(&self.entity).and_then(|view| view.element()) {
            return element == local_name.0;
        }

        false
    }

    fn has_namespace(&self, _ns: &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        false
    }

    fn is_part(&self, _name: &<Self::Impl as SelectorImpl>::Identifier) -> bool {
        false
    }

    fn imported_part(
        &self,
        _name: &<Self::Impl as SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as SelectorImpl>::Identifier> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn is_same_type(&self, other: &Self) -> bool {
        if let Some(element) = self.views.get(&self.entity).and_then(|view| view.element()) {
            if let Some(other_element) =
                self.views.get(&other.entity).and_then(|view| view.element())
            {
                return element == other_element;
            }
        }

        false
    }

    fn is_link(&self) -> bool {
        false
    }

    fn has_id(
        &self,
        name: &<Self::Impl as SelectorImpl>::Identifier,
        _case_sensitivity: CaseSensitivity,
    ) -> bool {
        if let Some(id) = self.store.ids.get(self.entity) {
            *id == name.0
        } else {
            false
        }
    }

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::Identifier,
        _case_sensitivity: CaseSensitivity,
    ) -> bool {
        if let Some(classes) = self.store.classes.get(self.entity) {
            return classes.contains(&name.0);
        }

        false
    }

    fn attr_matches(
        &self,
        _ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        _local_name: &<Self::Impl as SelectorImpl>::LocalName,
        _operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        _context: &mut MatchingContext<'_, Self::Impl>,
    ) -> bool {
        false
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        _context: &mut MatchingContext<'_, Self::Impl>,
    ) -> bool {
        if let Some(psudeo_class_flag) = self.store.pseudo_classes.get(self.entity) {
            match pc {
                PseudoClass::Hover => psudeo_class_flag.contains(PseudoClassFlags::HOVER),
                PseudoClass::Active => psudeo_class_flag.contains(PseudoClassFlags::ACTIVE),
                PseudoClass::Over => psudeo_class_flag.contains(PseudoClassFlags::OVER),
                PseudoClass::Focus => psudeo_class_flag.contains(PseudoClassFlags::FOCUS),
                PseudoClass::FocusVisible => {
                    psudeo_class_flag.contains(PseudoClassFlags::FOCUS_VISIBLE)
                }
                PseudoClass::FocusWithin => {
                    psudeo_class_flag.contains(PseudoClassFlags::FOCUS_WITHIN)
                }
                PseudoClass::Enabled => {
                    self.store.disabled.get(self.entity).map(|disabled| !*disabled).unwrap_or(true)
                }
                PseudoClass::Disabled => {
                    self.store.disabled.get(self.entity).copied().unwrap_or_default()
                }
                PseudoClass::ReadOnly => psudeo_class_flag.contains(PseudoClassFlags::READ_ONLY),
                PseudoClass::ReadWrite => psudeo_class_flag.contains(PseudoClassFlags::READ_WRITE),
                PseudoClass::PlaceHolderShown => {
                    psudeo_class_flag.contains(PseudoClassFlags::PLACEHOLDER_SHOWN)
                }
                PseudoClass::Default => psudeo_class_flag.contains(PseudoClassFlags::DEFAULT),
                PseudoClass::Checked => psudeo_class_flag.contains(PseudoClassFlags::CHECKED),
                PseudoClass::Indeterminate => {
                    psudeo_class_flag.contains(PseudoClassFlags::INDETERMINATE)
                }
                PseudoClass::Blank => psudeo_class_flag.contains(PseudoClassFlags::BLANK),
                PseudoClass::Valid => psudeo_class_flag.contains(PseudoClassFlags::VALID),
                PseudoClass::Invalid => psudeo_class_flag.contains(PseudoClassFlags::INVALID),
                PseudoClass::InRange => psudeo_class_flag.contains(PseudoClassFlags::IN_RANGE),
                PseudoClass::OutOfRange => {
                    psudeo_class_flag.contains(PseudoClassFlags::OUT_OF_RANGE)
                }
                PseudoClass::Required => psudeo_class_flag.contains(PseudoClassFlags::REQUIRED),
                PseudoClass::Optional => psudeo_class_flag.contains(PseudoClassFlags::OPTIONAL),
                PseudoClass::UserValid => psudeo_class_flag.contains(PseudoClassFlags::USER_VALID),
                PseudoClass::UserInvalid => {
                    psudeo_class_flag.contains(PseudoClassFlags::USER_INVALID)
                }
                PseudoClass::Lang(_) => todo!(),
                PseudoClass::Dir(_) => todo!(),
                PseudoClass::Custom(name) => {
                    println!("custom: {}", name);
                    todo!()
                }
            }
        } else {
            false
        }
    }

    fn first_element_child(&self) -> Option<Self> {
        None
    }

    fn apply_selector_flags(&self, _flags: ElementSelectorFlags) {}

    fn has_custom_state(&self, _name: &<Self::Impl as SelectorImpl>::Identifier) -> bool {
        false
    }

    fn add_element_unique_hashes(
        &self,
        _filter: &mut vizia_style::selectors::bloom::BloomFilter,
    ) -> bool {
        false
    }
}

/// Link inheritable inline properties to their parent.
pub(crate) fn inline_inheritance_system(cx: &mut Context, redraw_entities: &mut Vec<Entity>) {
    for entity in cx.tree.into_iter() {
        if let Some(parent) = cx.tree.get_layout_parent(entity) {
            if cx.style.disabled.inherit_inline(entity, parent)
                | cx.style.caret_color.inherit_inline(entity, parent)
                | cx.style.selection_color.inherit_inline(entity, parent)
            {
                redraw_entities.push(entity);
            }

            if cx.style.font_color.inherit_inline(entity, parent)
                | cx.style.font_size.inherit_inline(entity, parent)
                | cx.style.font_family.inherit_inline(entity, parent)
                | cx.style.font_weight.inherit_inline(entity, parent)
                | cx.style.font_slant.inherit_inline(entity, parent)
                | cx.style.font_width.inherit_inline(entity, parent)
                | cx.style.text_decoration_line.inherit_inline(entity, parent)
                | cx.style.text_stroke_width.inherit_inline(entity, parent)
                | cx.style.text_stroke_style.inherit_inline(entity, parent)
                | cx.style.font_variation_settings.inherit_inline(entity, parent)
            {
                cx.style.needs_text_update(entity);
            }
        }
    }
}

/// Link inheritable shared properties to their parent.
pub(crate) fn shared_inheritance_system(cx: &mut Context, redraw_entities: &mut Vec<Entity>) {
    for entity in cx.tree.into_iter() {
        if let Some(parent) = cx.tree.get_layout_parent(entity) {
            if cx.style.font_color.inherit_shared(entity, parent)
                | cx.style.font_size.inherit_shared(entity, parent)
                | cx.style.font_family.inherit_shared(entity, parent)
                | cx.style.font_weight.inherit_shared(entity, parent)
                | cx.style.font_slant.inherit_shared(entity, parent)
                | cx.style.font_width.inherit_shared(entity, parent)
                | cx.style.text_decoration_line.inherit_shared(entity, parent)
                | cx.style.text_stroke_width.inherit_shared(entity, parent)
                | cx.style.text_stroke_style.inherit_shared(entity, parent)
                | cx.style.font_variation_settings.inherit_shared(entity, parent)
            {
                cx.style.needs_text_update(entity);
            }

            if cx.style.caret_color.inherit_shared(entity, parent)
                | cx.style.selection_color.inherit_shared(entity, parent)
            {
                redraw_entities.push(entity);
            }
        }
    }
}

fn link_style_data(
    style: &mut Style,
    tree: &Tree<Entity>,
    entity: Entity,
    redraw_entities: &mut Vec<Entity>,
    matched_rules: &[Rule],
) {
    let mut should_relayout = false;
    let mut should_redraw = false;
    let mut should_reflow = false;

    // Display
    if style.display.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.visibility.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.z_index.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.overflowx.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.overflowy.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.clip_path.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.backdrop_filter.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.blend_mode.link(entity, matched_rules) {
        should_redraw = true;
    }

    // Opacity
    if style.opacity.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.left.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.right.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.top.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.bottom.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.min_left.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.min_right.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.min_top.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.min_bottom.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.max_left.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.max_right.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.max_top.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.max_bottom.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    // Size
    if style.width.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.height.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    // Size Constraints
    if style.max_width.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.min_width.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.max_height.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.min_height.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    // Border
    if style.border_width.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.border_color.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.border_style.link(entity, matched_rules) {
        should_redraw = true;
    }

    // Corner

    if style.corner_top_left_shape.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.corner_top_right_shape.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.corner_bottom_left_shape.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.corner_bottom_right_shape.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.corner_top_left_radius.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.corner_top_right_radius.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.corner_bottom_left_radius.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.corner_bottom_right_radius.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.outline_width.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.outline_color.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.outline_offset.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.layout_type.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.position_type.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    // Background
    if style.background_color.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.background_image.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.background_size.link(entity, matched_rules) {
        should_redraw = true;
    }

    // Font
    if style.font_color.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.font_size.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
        should_reflow = true;
    }

    if style.font_family.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
        should_reflow = true;
    }

    if style.font_weight.link(entity, matched_rules) {
        should_redraw = true;
        should_relayout = true;
        should_reflow = true;
    }

    if style.font_slant.link(entity, matched_rules) {
        should_redraw = true;
        should_relayout = true;
        should_reflow = true;
    }

    if style.font_width.link(entity, matched_rules) {
        should_redraw = true;
        should_relayout = true;
        should_reflow = true;
    }

    if style.font_variation_settings.link(entity, matched_rules) {
        should_redraw = true;
        should_relayout = true;
        should_reflow = true;
    }

    if style.text_wrap.link(entity, matched_rules) {
        should_redraw = true;
        should_relayout = true;
        should_reflow = true;
    }

    if style.text_align.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.text_overflow.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.line_clamp.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.selection_color.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.caret_color.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.text_decoration_line.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.text_stroke_width.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.text_stroke_style.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.underline_style.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.underline_color.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.overline_style.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.overline_color.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.strikethrough_style.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.strikethrough_color.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.letter_spacing.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.word_spacing.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    if style.line_height.link(entity, matched_rules) {
        should_redraw = true;
        should_reflow = true;
    }

    // Outer Shadow
    if style.shadow.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.child_left.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.child_right.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.child_top.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.child_bottom.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.row_between.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.col_between.link(entity, matched_rules) {
        should_relayout = true;
        should_redraw = true;
    }

    if style.cursor.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.pointer_events.link(entity, matched_rules) {
        should_redraw = true;
    }

    // Transform
    if style.transform.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.transform_origin.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.translate.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.rotate.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.scale.link(entity, matched_rules) {
        should_redraw = true;
    }

    if style.fill.link(entity, matched_rules) {
        should_redraw = true;
    }

    //
    if should_relayout {
        style.system_flags.set(SystemFlags::RELAYOUT, true);
    }

    if should_redraw {
        redraw_entities.push(entity);
    }

    if should_reflow {
        let iter = LayoutParentIterator::new(tree, entity);
        for parent in iter {
            if style.display.get(parent).copied().unwrap_or_default() != Display::None {
                style.needs_text_update(parent);
                break;
            }
        }
    }
}

/// Compute a list of matching style rules for a given entity.
pub(crate) fn compute_matched_rules(
    cx: &Context,
    entity: Entity,
    matched_rules: &mut Vec<(Rule, u32)>,
) {
    for (rule, selector_list) in cx.style.rules.iter() {
        let mut cache = SelectorCaches::default();
        let mut context = MatchingContext::new(
            MatchingMode::Normal,
            None,
            &mut cache,
            QuirksMode::NoQuirks,
            NeedsSelectorFlags::No,
            MatchingForInvalidation::No,
        );

        for selector in selector_list.slice() {
            let matches = matches_selector(
                selector,
                0,
                None,
                &Node { entity, store: &cx.style, tree: &cx.tree, views: &cx.views },
                &mut context,
            );
            if matches {
                matched_rules.push((*rule, selector.specificity()));
                break;
            }
        }
    }

    matched_rules.sort_by_cached_key(|(_, s)| *s);
    matched_rules.reverse();
}

fn has_same_selector(cx: &Context, entity1: Entity, entity2: Entity) -> bool {
    let element1 = cx.views.get(&entity1).and_then(|view| view.element()).unwrap_or_default();

    let element2 = cx.views.get(&entity2).and_then(|view| view.element()).unwrap_or_default();

    if element1 != element2 {
        return false;
    };

    let id1 = if let Some(id) = cx.style.ids.get(entity1) { id } else { "" };
    let id2 = if let Some(id) = cx.style.ids.get(entity2) { id } else { "" };

    if id1 != id2 {
        return false;
    }

    if let Some(classes1) = cx.style.classes.get(entity1) {
        if let Some(classes2) = cx.style.classes.get(entity2) {
            if !classes2.is_subset(classes1) || !classes1.is_subset(classes2) {
                return false;
            }
        }
    }

    if let Some(psudeo_class_flag1) = cx.style.pseudo_classes.get(entity1) {
        if let Some(psudeo_class_flag2) = cx.style.pseudo_classes.get(entity2) {
            if psudeo_class_flag2.bits() != psudeo_class_flag1.bits() {
                return false;
            }
        }
    }

    true
}

pub(crate) struct MatchedRulesCache {
    pub entity: Entity,
    pub rules: Vec<(Rule, u32)>,
}

// Iterates the tree and determines the matching style rules for each entity, then links the entity to the corresponding style rule data.
pub(crate) fn style_system(cx: &mut Context) {
    let mut redraw_entities = Vec::new();

    inline_inheritance_system(cx, &mut redraw_entities);

    if !cx.style.restyle.is_empty() {
        let iterator = TreeBreadthIterator::full(&cx.tree);

        let mut parent = None;
        let mut cache: Vec<MatchedRulesCache> = Vec::with_capacity(50);

        // Restyle the entire application.
        for entity in iterator {
            if !cx.style.restyle.contains(entity) {
                continue;
            }

            let mut matched_rules = Vec::with_capacity(50);

            let current_parent = cx.tree.get_layout_parent(entity);

            let mut compute_match = true;

            if current_parent == parent
                && !cx.tree.is_first_child(entity)
                && !cx.tree.is_last_child(entity)
            {
                // if has same selector look up rules
                'cache: for entry in &cache {
                    if has_same_selector(cx, entry.entity, entity) {
                        matched_rules.clone_from(&entry.rules);
                        compute_match = false;

                        for rule in entry.rules.iter() {
                            if let Some(selectors) = cx.style.rules.get(&rule.0) {
                                for selector in selectors.slice() {
                                    for component in selector.iter() {
                                        match *component {
                                            Component::Nth(n)
                                                if n.ty == NthType::Child
                                                    || n.ty == NthType::LastChild
                                                    || n.ty == NthType::OnlyChild =>
                                            {
                                                matched_rules.clear();
                                                compute_match = true;
                                                continue 'cache;
                                            }

                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }

                        break 'cache;
                    }
                }
            } else {
                parent = current_parent;
                cache.clear();
            }

            if compute_match {
                compute_matched_rules(cx, entity, &mut matched_rules);
                cache.push(MatchedRulesCache { entity, rules: matched_rules.clone() });
            }

            if !matched_rules.is_empty() {
                link_style_data(
                    &mut cx.style,
                    &cx.tree,
                    entity,
                    &mut redraw_entities,
                    &matched_rules.iter().map(|(rule, _)| *rule).collect::<Vec<_>>(),
                );
            }
        }
        cx.style.restyle.clear();

        shared_inheritance_system(cx, &mut redraw_entities);

        for entity in redraw_entities {
            cx.needs_redraw(entity);
        }
    }
}
