use crate::{accessibility::IntoNode, prelude::*};
use accesskit::{
    NodeBuilder, NodeId, Rect, TextDirection, TextPosition, TextSelection, TreeUpdate,
};
use cosmic_text::Edit;
use unicode_segmentation::UnicodeSegmentation;
use vizia_storage::LayoutTreeIterator;

// Updates node properties from view properties
// Should be run after layout so that things like bounding box are correct
// This system doesn't change the structure of the accessibility tree as this is done when views are built/removed
// TODO: Change this to incrementally update nodes when required instead of updating all nodes every frame
pub fn accessibility_system(cx: &mut Context, tree: &Tree<Entity>) {
    let iterator = LayoutTreeIterator::full(tree);

    for entity in iterator {
        let node_id = entity.accesskit_id();
        if let Some(node_builder) = cx.style.accesskit_node_builders.get_mut(entity) {
            let navigable = cx
                .style
                .abilities
                .get(entity)
                .copied()
                .unwrap_or_default()
                .contains(Abilities::NAVIGABLE);

            if node_builder.role() == Role::Unknown && !navigable {
                continue;
            }

            let bounds = cx.cache.get_bounds(entity);

            node_builder.set_bounds(Rect {
                x0: bounds.x as f64,
                y0: bounds.y as f64,
                x1: (bounds.x + bounds.w) as f64,
                y1: (bounds.y + bounds.h) as f64,
            });

            if let Some(disabled) = cx.style.disabled.get(entity).copied() {
                if disabled {
                    node_builder.set_disabled();
                } else {
                    node_builder.clear_disabled();
                }
            }

            let focusable = cx
                .style
                .abilities
                .get(entity)
                .map(|flags| flags.contains(Abilities::NAVIGABLE))
                .unwrap_or(false);

            if focusable {
                node_builder.set_selected_from_focus();
            } else {
                node_builder.clear_selected_from_focus();
            }

            let mut child_nodes = Vec::new();
            // Here we need to construct the correct text edit nodes for each wrapped line of text
            let role = node_builder.role();
            // println!("{} {:?}", entity, role);

            if role == Role::TextField {
                // This is a dirty hack because we need the bounds of the inner inner text content
                // which we know is going to be 3 more than the id of the textbox
                let text_content_id = Entity::new(entity.index() as u32 + 3, 0);
                let bounds = cx.cache.get_bounds(text_content_id);

                // We need a child node per line
                let mut children = Vec::new();
                cx.text_context.with_editor(text_content_id, |editor| {
                    let cursor = editor.cursor();
                    let selection = editor.select_opt().unwrap_or(cursor);

                    let mut selection_active_line = node_id;
                    let mut selection_anchor_line = node_id;
                    let mut selection_active_cursor = 0;
                    let mut selection_anchor_cursor = 0;

                    let mut current_cursor = 0;
                    let mut prev_line_index = std::usize::MAX;

                    for (index, line) in editor.buffer().layout_runs().enumerate() {
                        // Concatenate the parent id with the index of the text line to form a unique node id
                        let mut line_id = (entity.index() as u64 + 1) << 32;
                        line_id |= index as u64;
                        let line_id: NodeId = std::num::NonZeroU64::new(line_id).unwrap().into();

                        children.push(line_id);

                        let text = line.text;

                        let mut line_node = NodeBuilder::new(Role::InlineTextBox);

                        let line_height = editor.buffer().metrics().line_height as f64;
                        line_node.set_bounds(Rect {
                            x0: bounds.x as f64,
                            y0: bounds.y as f64 + line.line_y as f64
                                - editor.buffer().metrics().font_size as f64,
                            x1: bounds.x as f64 + line.line_w as f64,
                            y1: bounds.y as f64 + line.line_y as f64
                                - editor.buffer().metrics().font_size as f64
                                + line_height,
                        });
                        line_node.set_text_direction(if line.rtl {
                            TextDirection::RightToLeft
                        } else {
                            TextDirection::LeftToRight
                        });

                        let mut character_lengths = Vec::with_capacity(line.glyphs.len());
                        let mut character_positions = Vec::with_capacity(line.glyphs.len());
                        let mut character_widths = Vec::with_capacity(line.glyphs.len());

                        // Get the actual text in the line
                        let first_glyph_pos =
                            line.glyphs.first().map(|glyph| glyph.start).unwrap_or_default();
                        let last_glyph_pos =
                            line.glyphs.last().map(|glyph| glyph.end).unwrap_or_default();

                        let mut line_text = text[first_glyph_pos..last_glyph_pos].to_owned();

                        let word_lengths = line_text
                            .unicode_words()
                            .map(|word| word.len() as u8)
                            .collect::<Vec<_>>();

                        let mut line_length = 0;

                        for glyph in line.glyphs.iter() {
                            let length = (glyph.end - glyph.start) as u8;

                            line_length += length as usize;

                            let position = glyph.x;
                            let width = glyph.w;

                            character_lengths.push(length);
                            character_positions.push(position);
                            character_widths.push(width);
                        }

                        // Cosmic strips the newlines but accesskit needs them so we append them back in if line ended originally ended with a newline
                        // If the last glyph position is equal to the end of the buffer line then this layout run is the last one and ends in a newline.
                        if last_glyph_pos == line.text.len() {
                            line_text += "\n";
                            character_lengths.push(1);
                            character_positions.push(line.line_w);
                            character_widths.push(0.0);
                        }

                        // FIXME: The screen reader gets out of sync with the cursor due to affinity

                        // TODO: Might need to append any spaces that were stripped during layout. This can be done by
                        // figuring out if the start of the next line is greater than the end of the current line as long
                        // as the lines have the same `line_i`. This will require a peekable iterator loop.

                        // if prev_line_index == line.line_i {
                        //     println!("{} {} {}", line_text, first_glyph_pos, current_cursor);
                        // }

                        line_node.set_value(line_text.into_boxed_str());
                        line_node.set_character_lengths(character_lengths.into_boxed_slice());
                        line_node.set_character_positions(character_positions.into_boxed_slice());
                        line_node.set_character_widths(character_widths.into_boxed_slice());
                        line_node.set_word_lengths(word_lengths.into_boxed_slice());
                        child_nodes
                            .push((line_id, line_node.build(&mut cx.style.accesskit_node_classes)));

                        if line.line_i != prev_line_index {
                            current_cursor = 0;
                        }

                        if line.line_i == cursor.line {
                            if prev_line_index != line.line_i {
                                if cursor.index <= line_length {
                                    selection_active_line = line_id;
                                    selection_active_cursor = cursor.index;
                                }
                            } else {
                                if cursor.index > current_cursor {
                                    selection_active_line = line_id;
                                    selection_active_cursor = cursor.index - current_cursor;
                                }
                            }
                        }

                        // Check if the current line contains the cursor or selection
                        // This is a mess because a line happens due to soft and hard breaks but
                        // the cursor and selected indices are relative to the lines caused by hard breaks only.
                        if line.line_i == selection.line {
                            // A previous line index different to the current means that the current line follows a hard break
                            if prev_line_index != line.line_i {
                                if selection.index <= line_length {
                                    selection_anchor_line = line_id;
                                    selection_anchor_cursor = selection.index;
                                }
                            } else {
                                if selection.index > current_cursor {
                                    selection_anchor_line = line_id;
                                    selection_anchor_cursor = selection.index - current_cursor;
                                }
                            }
                        }

                        current_cursor += line_length;
                        prev_line_index = line.line_i;
                    }

                    // println!(
                    //     "{:?} {} {:?} {}",
                    //     selection_anchor_line,
                    //     selection_anchor_cursor,
                    //     selection_active_line,
                    //     selection_active_cursor
                    // );

                    node_builder.set_text_selection(TextSelection {
                        anchor: TextPosition {
                            node: selection_anchor_line,
                            character_index: selection_anchor_cursor,
                        },
                        focus: TextPosition {
                            node: selection_active_line,
                            character_index: selection_active_cursor,
                        },
                    });

                    // println!("children: {} {:?}", entity, children);

                    node_builder.set_children(children);
                });
            }

            // println!("{:?} {:?}", node_id, node_builder.labelled_by());

            let mut nodes =
                vec![(node_id, node_builder.clone().build(&mut cx.style.accesskit_node_classes))];

            // println!("{} {:?}", entity, child_nodes);
            // If child nodes were generated then append them to the nodes list
            if !child_nodes.is_empty() {
                nodes.extend(child_nodes.into_iter());
            }

            cx.tree_updates.push(TreeUpdate {
                nodes,
                tree: None,
                focus: cx.window_has_focus.then_some(cx.focused.accesskit_id()),
            });
        }
    }
}