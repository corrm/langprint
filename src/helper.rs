use crate::text::IndentStyle;

// #[inline(always)]
// ponytail: indent_level stays i32 to avoid cascading &mut i32 → usize across 7 pub traits and 3 backends. Migrate at 1.0.
pub(crate) fn indent(indent_level: i32, indent_size: i32, indent_style: IndentStyle) -> String {
    if indent_level < 0 {
        panic!("Indent level out of range");
    } else if indent_level == 0 {
        return String::new();
    }

    match indent_style {
        IndentStyle::Tabs => "\t".repeat(indent_level as usize),
        IndentStyle::Spaces => " ".repeat((indent_level * indent_size) as usize),
    }
}
