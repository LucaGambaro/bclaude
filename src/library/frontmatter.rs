/// Subset of YAML frontmatter we care about.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
}

fn unquote(value: &str) -> String {
    let v = value.trim();
    for q in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return v[1..v.len() - 1].to_string();
        }
    }
    v.to_string()
}

/// Parses `---` delimited frontmatter; supports plain, quoted and `>`/`|` block values.
pub fn parse(text: &str) -> Frontmatter {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Frontmatter::default();
    }
    let body: Vec<&str> = lines.take_while(|l| l.trim() != "---").collect();
    let mut fm = Frontmatter::default();
    let mut i = 0;
    while i < body.len() {
        let line = body[i];
        i += 1;
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        let value = value.trim();
        let resolved = if value.starts_with('>') || value.starts_with('|') {
            let mut parts = Vec::new();
            while i < body.len()
                && (body[i].starts_with(char::is_whitespace) || body[i].trim().is_empty())
            {
                if !body[i].trim().is_empty() {
                    parts.push(body[i].trim());
                }
                i += 1;
            }
            parts.join(" ")
        } else {
            unquote(value)
        };
        match key.trim() {
            "name" => fm.name = Some(resolved),
            "description" => fm.description = Some(resolved),
            _ => {}
        }
    }
    fm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_is_empty() {
        assert_eq!(parse("# title\nname: x"), Frontmatter::default());
        assert_eq!(parse(""), Frontmatter::default());
    }

    #[test]
    fn plain_and_quoted_values() {
        let fm = parse("---\nname: foo\ndescription: \"Does: things\"\nother: 1\n---\nbody");
        assert_eq!(fm.name.as_deref(), Some("foo"));
        assert_eq!(fm.description.as_deref(), Some("Does: things"));
    }

    #[test]
    fn single_quotes_are_stripped() {
        assert_eq!(
            parse("---\ndescription: 'hi'\n---").description.as_deref(),
            Some("hi")
        );
    }

    #[test]
    fn block_scalars_are_joined() {
        let fm = parse("---\ndescription: >\n  first line\n  second line\nname: n\n---\n");
        assert_eq!(fm.description.as_deref(), Some("first line second line"));
        assert_eq!(fm.name.as_deref(), Some("n"));
    }

    #[test]
    fn indented_keys_are_ignored() {
        let fm = parse("---\nmetadata:\n  name: nested\n---\n");
        assert_eq!(fm.name, None);
    }

    #[test]
    fn unterminated_frontmatter_reads_to_end() {
        assert_eq!(parse("---\nname: a\n").name.as_deref(), Some("a"));
    }
}
