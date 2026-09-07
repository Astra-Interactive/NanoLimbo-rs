use crate::chat::{
    ClickEvent, Component, ComponentContent, HoverEvent, NamedColor, RgbColor, Style, TextColor,
};
use crate::mini_message_node::MiniMessageNode;

/// Tags that insert content and then end, rather than styling what follows them.
const INLINE_TAGS: [&str; 7] = ["newline", "br", "key", "lang", "translate", "tr", "reset"];

fn is_inline_tag(name: &str) -> bool {
    INLINE_TAGS.contains(&name)
}

fn named_color(name: &str) -> Option<NamedColor> {
    match name {
        "grey" => Some(NamedColor::Gray),
        "dark_grey" => Some(NamedColor::DarkGray),
        other => NamedColor::from_key(other),
    }
}

fn hex_color(text: &str) -> Option<RgbColor> {
    let digits = text.strip_prefix('#')?;
    if digits.len() != 6 || !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(digits, 16)
        .ok()
        .map(RgbColor::from_packed)
}

/// Resolves a colour written either by name or as `#rrggbb`.
fn parse_color(text: &str) -> Option<TextColor> {
    named_color(text)
        .map(TextColor::Named)
        .or_else(|| hex_color(text).map(TextColor::Rgb))
}

fn decoration_of(name: &str) -> Option<&'static str> {
    match name {
        "b" | "bold" => Some("bold"),
        "i" | "italic" | "em" => Some("italic"),
        "u" | "underlined" => Some("underlined"),
        "st" | "strikethrough" => Some("strikethrough"),
        "obf" | "obfuscated" => Some("obfuscated"),
        _ => None,
    }
}

/// Splits the inside of a tag on `:`, honouring single and double quotes so a URL or a
/// command containing a colon survives intact.
fn split_arguments(body: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for character in body.chars() {
        match quote {
            Some(active) if character == active => quote = None,
            Some(_) => current.push(character),
            None if character == '\'' || character == '"' => quote = Some(character),
            None if character == ':' => parts.push(std::mem::take(&mut current)),
            None => current.push(character),
        }
    }
    parts.push(current);
    parts
}

/// Reads the body of a tag starting at `open`, returning it and the index just past `>`.
///
/// Returns `None` when the tag is unterminated, in which case the `<` is literal text.
fn read_tag_body(characters: &[char], open: usize) -> Option<(String, usize)> {
    let mut body = String::new();
    let mut quote: Option<char> = None;
    let mut index = open + 1;

    while index < characters.len() {
        let character = *characters.get(index)?;
        match quote {
            Some(active) if character == active => {
                quote = None;
                body.push(character);
            }
            Some(_) => body.push(character),
            None if character == '\'' || character == '"' => {
                quote = Some(character);
                body.push(character);
            }
            None if character == '>' => return Some((body, index + 1)),
            None if character == '<' => return None,
            None => body.push(character),
        }
        index += 1;
    }
    None
}

fn push_text(children: &mut Vec<MiniMessageNode>, buffer: &mut String) {
    if !buffer.is_empty() {
        children.push(MiniMessageNode::Text {
            text: std::mem::take(buffer),
        });
    }
}

/// Parses MiniMessage markup into a node tree.
///
/// Unknown tags are kept as literal text, matching the reference implementation's
/// non-strict mode: a message is never rejected for a typo, it just shows the typo.
fn parse_nodes(input: &str) -> Vec<MiniMessageNode> {
    let characters: Vec<char> = input.chars().collect();
    let mut stack: Vec<MiniMessageNode> = vec![MiniMessageNode::Tag {
        name: String::new(),
        arguments: Vec::new(),
        raw: String::new(),
        children: Vec::new(),
    }];
    let mut buffer = String::new();
    let mut index = 0;

    while index < characters.len() {
        let Some(&character) = characters.get(index) else {
            break;
        };

        if character == '\\'
            && let Some(&next) = characters.get(index + 1)
            && (next == '<' || next == '>')
        {
            buffer.push(next);
            index += 2;
            continue;
        }

        if character != '<' {
            buffer.push(character);
            index += 1;
            continue;
        }

        let Some((body, after)) = read_tag_body(&characters, index) else {
            buffer.push(character);
            index += 1;
            continue;
        };

        let closing = body.starts_with('/');
        let arguments = split_arguments(body.trim_start_matches('/'));
        let Some(name) = arguments.first().map(|first| first.to_ascii_lowercase()) else {
            buffer.push(character);
            index += 1;
            continue;
        };

        if closing {
            close_tag(&mut stack, &mut buffer, &name);
            index = after;
            continue;
        }

        if name == "reset" {
            push_text(current_children(&mut stack), &mut buffer);
            collapse_to_root(&mut stack);
            index = after;
            continue;
        }

        push_text(current_children(&mut stack), &mut buffer);
        let node = MiniMessageNode::Tag {
            name: name.clone(),
            arguments: arguments.get(1..).unwrap_or_default().to_vec(),
            raw: format!("<{body}>"),
            children: Vec::new(),
        };

        if is_inline_tag(&name) {
            current_children(&mut stack).push(node);
        } else {
            stack.push(node);
        }
        index = after;
    }

    push_text(current_children(&mut stack), &mut buffer);
    collapse_to_root(&mut stack);

    match stack.into_iter().next() {
        Some(MiniMessageNode::Tag { children, .. }) => children,
        _ => Vec::new(),
    }
}

fn current_children(stack: &mut [MiniMessageNode]) -> &mut Vec<MiniMessageNode> {
    for node in stack.iter_mut().rev() {
        if let MiniMessageNode::Tag { children, .. } = node {
            return children;
        }
    }
    unreachable!("the stack always holds at least the synthetic root tag")
}

/// Pops one level of the stack, attaching it to its parent.
fn pop_one(stack: &mut Vec<MiniMessageNode>) {
    if stack.len() <= 1 {
        return;
    }
    if let Some(finished) = stack.pop() {
        current_children(stack).push(finished);
    }
}

fn collapse_to_root(stack: &mut Vec<MiniMessageNode>) {
    while stack.len() > 1 {
        pop_one(stack);
    }
}

/// Closes the innermost tag with this name, or the innermost tag of any name when the
/// closing tag names nothing that is open.
fn close_tag(stack: &mut Vec<MiniMessageNode>, buffer: &mut String, name: &str) {
    push_text(current_children(stack), buffer);

    let target = stack.iter().rposition(|node| match node {
        MiniMessageNode::Tag { name: open, .. } => {
            open == name
                || decoration_of(open) == decoration_of(name) && decoration_of(name).is_some()
        }
        MiniMessageNode::Text { .. } => false,
    });

    match target {
        Some(position) if position > 0 => {
            while stack.len() > position {
                pop_one(stack);
            }
        }
        _ => pop_one(stack),
    }
}

fn gradient_color(stops: &[TextColor], position: usize, total: usize) -> TextColor {
    let Some(&first) = stops.first() else {
        return TextColor::Named(NamedColor::White);
    };
    if stops.len() == 1 || total <= 1 {
        return first;
    }

    let progress = position as f64 / (total - 1) as f64;
    let segments = (stops.len() - 1) as f64;
    let scaled = progress * segments;
    let index = (scaled.floor() as usize).min(stops.len() - 2);
    let local = scaled - index as f64;

    let (Some(&start), Some(&end)) = (stops.get(index), stops.get(index + 1)) else {
        return first;
    };
    let blend = |from: u8, to: u8| -> u8 {
        (f64::from(from) + (f64::from(to) - f64::from(from)) * local + 0.5).floor() as u8
    };
    TextColor::Rgb(RgbColor::new(
        blend(start.rgb().red, end.rgb().red),
        blend(start.rgb().green, end.rgb().green),
        blend(start.rgb().blue, end.rgb().blue),
    ))
}

/// Full-saturation, full-value colour at the given hue, truncating each channel the way
/// the reference implementation does.
fn rainbow_color(position: usize, total: usize) -> TextColor {
    let hue = if total == 0 {
        0.0
    } else {
        position as f64 / total as f64
    };
    let sector = (hue * 6.0).floor();
    let offset = hue * 6.0 - sector;
    let rising = (offset * 255.0).floor() as u8;
    // Not `255 - rising`: each channel is floored independently, and at the midpoint of a
    // sector that is 127 on both sides rather than 127 and 128.
    let falling = ((1.0 - offset) * 255.0).floor() as u8;

    let rgb = match sector as u32 % 6 {
        0 => RgbColor::new(255, rising, 0),
        1 => RgbColor::new(falling, 255, 0),
        2 => RgbColor::new(0, 255, rising),
        3 => RgbColor::new(0, falling, 255),
        4 => RgbColor::new(rising, 0, 255),
        _ => RgbColor::new(255, 0, falling),
    };
    TextColor::Rgb(rgb)
}

/// A colour ramp being spread across the characters enclosed by a tag.
struct ColorSpread {
    stops: Vec<TextColor>,
    rainbow: bool,
    total: usize,
    position: usize,
}

impl ColorSpread {
    fn next_color(&mut self) -> TextColor {
        let color = if self.rainbow {
            rainbow_color(self.position, self.total)
        } else {
            gradient_color(&self.stops, self.position, self.total)
        };
        self.position += 1;
        color
    }
}

/// Builds a component for a tag, folding a lone plain-text child into it.
///
/// Without the fold, `<red>text` would produce an empty component wrapping a text one.
/// The reference implementation collapses that, and so does this — but only when the
/// child carries nothing of its own to lose.
fn assemble(style: Style, mut children: Vec<Component>) -> Component {
    let absorbable = children.first().is_some_and(|first| {
        matches!(first.content, ComponentContent::Text { .. })
            && first.style.is_empty()
            && first.children.is_empty()
    });

    if absorbable && !children.is_empty() {
        let first = children.remove(0);
        return Component::new(first.content, style, children);
    }

    Component::new(
        ComponentContent::Text {
            text: String::new(),
        },
        style,
        children,
    )
}

fn spread_text(text: &str, spread: &mut ColorSpread) -> Vec<Component> {
    text.chars()
        .map(|character| {
            let mut style = Style::empty();
            style.color = Some(spread.next_color());
            Component::new(
                ComponentContent::Text {
                    text: character.to_string(),
                },
                style,
                Vec::new(),
            )
        })
        .collect()
}

fn style_for_tag(name: &str, arguments: &[String]) -> Option<Style> {
    let mut style = Style::empty();

    if let Some(color) = parse_color(name) {
        style.color = Some(color);
        return Some(style);
    }
    if (name == "color" || name == "colour" || name == "c")
        && let Some(color) = arguments.first().and_then(|value| parse_color(value))
    {
        style.color = Some(color);
        return Some(style);
    }

    match decoration_of(name) {
        Some("bold") => style.bold = Some(true),
        Some("italic") => style.italic = Some(true),
        Some("underlined") => style.underlined = Some(true),
        Some("strikethrough") => style.strikethrough = Some(true),
        Some("obfuscated") => style.obfuscated = Some(true),
        _ => return click_or_hover_style(name, arguments),
    }
    Some(style)
}

fn click_or_hover_style(name: &str, arguments: &[String]) -> Option<Style> {
    let mut style = Style::empty();

    match name {
        "click" => {
            let action = arguments.first()?.as_str();
            let value = arguments.get(1).cloned().unwrap_or_default();
            style.click_event = Some(match action {
                "open_url" => ClickEvent::OpenUrl { url: value },
                "run_command" => ClickEvent::RunCommand { command: value },
                "suggest_command" => ClickEvent::SuggestCommand { command: value },
                "copy_to_clipboard" => ClickEvent::CopyToClipboard { text: value },
                _ => return None,
            });
        }
        "hover" => {
            if arguments.first()?.as_str() != "show_text" {
                return None;
            }
            let value = arguments.get(1).cloned().unwrap_or_default();
            style.hover_event = Some(HoverEvent::ShowText {
                text: Box::new(parse_mini_message(&value)),
            });
        }
        "font" => style.font = Some(arguments.first()?.clone()),
        "insertion" => style.insertion = Some(arguments.first()?.clone()),
        _ => return None,
    }
    Some(style)
}

fn inline_component(name: &str, arguments: &[String]) -> Option<Component> {
    match name {
        "newline" | "br" => Some(Component::text("\n")),
        "key" => Some(Component::keybind(arguments.first()?.clone())),
        "lang" | "translate" | "tr" => Some(Component::translatable(
            arguments.first()?.clone(),
            Vec::new(),
        )),
        _ => None,
    }
}

fn build_nodes(nodes: &[MiniMessageNode], spread: &mut Option<ColorSpread>) -> Vec<Component> {
    let mut built = Vec::new();

    for node in nodes {
        match node {
            MiniMessageNode::Text { text } => match spread {
                Some(active) => built.extend(spread_text(text, active)),
                None => built.push(Component::text(text.clone())),
            },
            MiniMessageNode::Tag {
                name,
                arguments,
                raw,
                children,
            } => built.extend(build_tag(name, arguments, raw, children, spread)),
        }
    }

    built
}

fn build_tag(
    name: &str,
    arguments: &[String],
    raw: &str,
    children: &[MiniMessageNode],
    spread: &mut Option<ColorSpread>,
) -> Vec<Component> {
    if let Some(component) = inline_component(name, arguments) {
        return vec![component];
    }

    if name == "gradient" || name == "rainbow" {
        let total = children.iter().map(MiniMessageNode::text_length).sum();
        let stops: Vec<TextColor> = arguments
            .iter()
            .filter_map(|argument| parse_color(argument))
            .collect();
        let mut nested = Some(ColorSpread {
            stops: if stops.is_empty() {
                vec![TextColor::Named(NamedColor::White)]
            } else {
                stops
            },
            rainbow: name == "rainbow",
            total,
            position: 0,
        });
        return vec![assemble(Style::empty(), build_nodes(children, &mut nested))];
    }

    match style_for_tag(name, arguments) {
        Some(style) => vec![assemble(style, build_nodes(children, spread))],
        // Unrecognised, so it was never markup. Show it, the way a lenient parser should.
        None => {
            let mut built = vec![Component::text(raw.to_owned())];
            built.extend(build_nodes(children, spread));
            built
        }
    }
}

/// Parses MiniMessage markup into a component.
pub fn parse_mini_message(input: &str) -> Component {
    let nodes = parse_nodes(input);
    let mut built = build_nodes(&nodes, &mut None);

    match built.len() {
        0 => Component::empty(),
        1 => built.remove(0),
        _ => assemble(Style::empty(), built),
    }
}
