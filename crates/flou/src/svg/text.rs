use crate::{pos::PixelPos, svg::SVGElement};

pub(crate) struct SVGText {
    pos: PixelPos,
}

impl SVGText {
    pub(crate) fn new(pos: PixelPos) -> Self {
        Self { pos }
    }

    pub(crate) fn render(self, s: &str) -> SVGElement {
        let text = SVGElement::new("text").pos(self.pos);
        let line_count = s.lines().count();

        if line_count == 1 {
            return text.text(s);
        }

        let children = s.lines().enumerate().map(|(i, line)| {
            let offset = Self::calculate_offset(i, line_count);

            SVGElement::new("tspan")
                .attr("x", self.pos.x.to_string())
                .attr("dy", format!("{}em", offset))
                .text(line)
        });

        text.children(children)
    }

    fn calculate_offset(line_number: usize, line_count: usize) -> f32 {
        if line_number == 0 {
            -((line_count - 1) as f32) / 2.0
        } else {
            1.0
        }
    }
}
