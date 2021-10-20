use crate::{pos::PixelPos, svg::SVGElement};

pub(crate) enum PathD {
    MoveTo(PixelPos),
    LineTo(PixelPos),
    End,
}

impl ToString for PathD {
    fn to_string(&self) -> String {
        match self {
            PathD::MoveTo(pos) => format!("M {} {}", pos.x, pos.y),
            PathD::LineTo(pos) => format!("L {} {}", pos.x, pos.y),
            PathD::End => "Z".into(),
        }
    }
}

pub(crate) struct SVGPath {
    d: Vec<PathD>,
}

impl SVGPath {
    pub(crate) fn new() -> Self {
        Self { d: Vec::new() }
    }

    pub(crate) fn line_to(mut self, pos: PixelPos) -> Self {
        let cmd = if self.d.is_empty() {
            PathD::MoveTo(pos)
        } else {
            PathD::LineTo(pos)
        };
        self.d.push(cmd);
        self
    }

    pub(crate) fn end(mut self) -> Self {
        self.d.push(PathD::End);
        self
    }

    pub(crate) fn render(self) -> SVGElement<'static> {
        SVGElement::new("path").attr("d", self.get_d())
    }

    fn get_d(&self) -> String {
        self.d
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use crate::{pos::pos, svg::SVGPath};

    #[test]
    fn create_path() {
        let mut path = SVGPath::new().line_to(pos(10, 20));
        assert_eq!(path.get_d(), "M 10 20");

        path = path.line_to(pos(30, 40));
        assert_eq!(path.get_d(), "M 10 20 L 30 40");

        path = path.end();
        assert_eq!(path.get_d(), "M 10 20 L 30 40 Z");
    }
}
