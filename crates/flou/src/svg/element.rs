use std::{
    borrow::Cow,
    fmt::{self},
};

use crate::pos::PixelPos;

fn escape(input: &str) -> Cow<str> {
    fn should_escape(c: char) -> bool {
        c == '<' || c == '>' || c == '&' || c == '"' || c == '\''
    }

    if input.contains(should_escape) {
        let mut output = String::with_capacity(input.len());
        for c in input.chars() {
            match c {
                '\'' => output.push_str("&apos;"),
                '"' => output.push_str("&quot;"),
                '<' => output.push_str("&lt;"),
                '>' => output.push_str("&gt;"),
                '&' => output.push_str("&amp;"),
                _ => output.push(c),
            }
        }
        Cow::Owned(output)
    } else {
        Cow::Borrowed(input)
    }
}

// This is a hacky workaround for lifetime issues in SVGElement::text().
// There's probably a better way of resolving them without duplicating code.
fn escape_cow(input: Cow<str>) -> Cow<str> {
    fn should_escape(c: char) -> bool {
        c == '<' || c == '>' || c == '&' || c == '"' || c == '\''
    }

    if input.contains(should_escape) {
        let mut output = String::with_capacity(input.len());
        for c in input.chars() {
            match c {
                '\'' => output.push_str("&apos;"),
                '"' => output.push_str("&quot;"),
                '<' => output.push_str("&lt;"),
                '>' => output.push_str("&gt;"),
                '&' => output.push_str("&amp;"),
                _ => output.push(c),
            }
        }
        Cow::Owned(output)
    } else {
        input
    }
}

fn indent(depth: usize) -> String {
    const SIZE: usize = 2;
    " ".repeat(SIZE * depth)
}

#[derive(Debug)]
enum Node<'a> {
    Text(Cow<'a, str>),
    Element(SVGElement<'a>),
}

impl Node<'_> {
    fn print(&self, depth: usize, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Text(text) => {
                for (i, line) in text.lines().enumerate() {
                    if i != 0 {
                        writeln!(f)?;
                    }
                    f.write_str(&indent(depth))?;
                    f.write_str(line)?;
                }
            }
            Node::Element(el) => el.print(depth, f)?,
        };

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct SVGElement<'a> {
    tag: Cow<'a, str>,
    attributes: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    classes: Vec<Cow<'a, str>>,
    children: Vec<Node<'a>>,
}

impl<'a> SVGElement<'a> {
    pub(crate) fn new<I: Into<Cow<'a, str>>>(tag: I) -> Self {
        Self {
            tag: tag.into(),
            attributes: Vec::new(),
            classes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub(crate) fn pos(self, pos: PixelPos) -> Self {
        self.attr("x", pos.x.to_string())
            .attr("y", pos.y.to_string())
    }

    pub(crate) fn cpos(self, pos: PixelPos) -> Self {
        self.attr("cx", pos.x.to_string())
            .attr("cy", pos.y.to_string())
    }

    pub(crate) fn size(self, size: PixelPos) -> Self {
        self.attr("width", size.x.to_string())
            .attr("height", size.y.to_string())
    }

    pub(crate) fn class<I: Into<Cow<'a, str>>>(mut self, s: I) -> Self {
        self.classes.push(s.into());
        self
    }

    pub(crate) fn class_opt<I: Into<Cow<'a, str>>>(self, s: Option<I>) -> Self {
        match s {
            Some(s) => self.class(s),
            None => self,
        }
    }

    pub(crate) fn attr<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        let key = key.into();
        if key == "class" {
            panic!("Use .class() instead.");
        }

        self.attributes.push((key, value.into()));
        self
    }

    pub(crate) fn child(mut self, child: SVGElement<'a>) -> Self {
        self.children.push(Node::Element(child));
        self
    }

    pub(crate) fn child_opt(self, child: Option<SVGElement<'a>>) -> Self {
        match child {
            Some(child) => self.child(child),
            None => self,
        }
    }

    pub(crate) fn text<I: Into<Cow<'a, str>>>(mut self, text: I) -> Self {
        let text = text.into();
        let text = escape_cow(text);
        self.children.push(Node::Text(text));
        self
    }

    pub(crate) fn children<T>(mut self, children: T) -> Self
    where
        T: IntoIterator<Item = SVGElement<'a>>,
    {
        self.children
            .extend(children.into_iter().map(Node::Element));
        self
    }

    fn print(&self, depth: usize, f: &mut fmt::Formatter) -> fmt::Result {
        let attributes = self
            .attributes
            .iter()
            .map(|(key, value)| (key, escape(value)))
            .collect::<Vec<_>>();

        let classes = self.classes.iter().map(|x| escape(x)).collect::<Vec<_>>();

        f.write_str(&indent(depth))?;
        f.write_str("<")?;
        f.write_str(&self.tag)?;

        if !classes.is_empty() {
            write!(f, " class=\"{}\"", classes.join(" "))?;
        }

        let attributes = attributes
            .iter()
            .map(|(k, v)| format!(" {}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join("");
        f.write_str(&attributes)?;

        if self.children.is_empty() {
            f.write_str(" />")?;
            return Ok(());
        }

        f.write_str(">")?;

        match self.children.first() {
            Some(child @ Node::Text(_)) if self.children.len() == 1 => {
                child.print(0, f)?;
            }
            _ => {
                for child in &self.children {
                    writeln!(f)?;
                    child.print(depth + 1, f)?;
                }
                writeln!(f)?;
                f.write_str(&indent(depth))?;
            }
        }

        write!(f, "</{}>", self.tag)?;
        Ok(())
    }
}

impl fmt::Display for SVGElement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print(0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::{escape, SVGElement};

    use crate::test::assert_eq;

    #[test]
    fn tag_only() {
        assert_eq!(SVGElement::new("a").to_string(), "<a />");
    }

    #[test]
    fn with_attributes() {
        assert_eq!(
            SVGElement::new("a").attr("foo", "bar").to_string(),
            r#"<a foo="bar" />"#,
        );

        assert_eq!(
            SVGElement::new("a")
                .attr("foo", "bar")
                .attr("bar", "baz")
                .to_string(),
            r#"<a foo="bar" bar="baz" />"#,
        );
    }

    #[test]
    fn with_child() {
        assert_eq!(
            SVGElement::new("div")
                .child(SVGElement::new("foo"))
                .to_string(),
            r#"
<div>
  <foo />
</div>
            "#
            .trim(),
        );
    }

    #[test]
    fn escape_attributes() {
        assert_eq!(escape("\""), "&quot;");
        assert_eq!(escape("'"), "&apos;");
        assert_eq!(escape("<"), "&lt;");
        assert_eq!(escape(">"), "&gt;");
        assert_eq!(escape("&"), "&amp;");
    }

    #[test]
    fn with_escaped_attribute() {
        assert_eq!(
            SVGElement::new("div").class("'Hi'").to_string(),
            r#"<div class="&apos;Hi&apos;" />"#,
        )
    }

    #[test]
    fn complex_example() {
        assert_eq!(
            SVGElement::new("p")
                .class("block")
                .child(
                    SVGElement::new("a")
                        .attr("href", "example.com")
                        .child(SVGElement::new("span").text("Hi"))
                        .text("there")
                )
                .child(SVGElement::new("button").text("Press me"))
                .to_string(),
            r#"
<p class="block">
  <a href="example.com">
    <span>Hi</span>
    there
  </a>
  <button>Press me</button>
</p>
            "#
            .trim(),
        );
    }
}
