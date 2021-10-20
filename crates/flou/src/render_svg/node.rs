use crate::{
    parse::ast::NodeShape,
    parts::NodeAttributes,
    svg::{SVGElement, SVGPath, SVGText},
};

use super::viewport::Viewport;

impl Default for NodeShape {
    fn default() -> Self {
        Self::Rectangle
    }
}

impl NodeShape {
    pub(crate) fn render(&self, viewport: Viewport) -> SVGElement<'static> {
        match &self {
            Self::Rectangle => SVGElement::new("rect")
                .class("rectangle")
                .pos(viewport.origin)
                .size(viewport.size),

            Self::Square => {
                let size = std::cmp::min(viewport.size.x, viewport.size.y);
                let origin = viewport.origin + (viewport.size - size) / 2;

                SVGElement::new("rect")
                    .class("square")
                    .pos(origin)
                    .size(size.into())
            }

            Self::Diamond => {
                let midpoints = viewport.midpoints();

                SVGPath::new()
                    .line_to(midpoints.top)
                    .line_to(midpoints.left)
                    .line_to(midpoints.bottom)
                    .line_to(midpoints.right)
                    .end()
                    .render()
                    .class("diamond")
            }

            Self::AngledSquare => {
                let size = std::cmp::min(viewport.size.x, viewport.size.y);
                let origin = viewport.origin + (viewport.size - size) / 2;
                let viewport = Viewport::new(origin, size.into());
                let midpoints = viewport.midpoints();

                SVGPath::new()
                    .line_to(midpoints.top)
                    .line_to(midpoints.left)
                    .line_to(midpoints.bottom)
                    .line_to(midpoints.right)
                    .end()
                    .render()
                    .class("angled_square")
            }

            Self::Ellipse => {
                let size = viewport.size / 2;

                SVGElement::new("ellipse")
                    .class("ellipse")
                    .cpos(viewport.center())
                    .attr("rx", size.x.to_string())
                    .attr("ry", size.y.to_string())
            }

            Self::Circle => {
                let diameter = std::cmp::min(viewport.size.x, viewport.size.y);
                let radius = diameter / 2;

                SVGElement::new("circle")
                    .class("circe")
                    .cpos(viewport.center())
                    .attr("r", radius.to_string())
            }
        }
    }
}

impl NodeAttributes {
    fn wrapper() -> SVGElement<'static> {
        SVGElement::new("g").class("node-wrapper")
    }

    pub(crate) fn render_default(viewport: Viewport) -> SVGElement<'static> {
        let shape = NodeShape::default().render(viewport);
        Self::wrapper().child(shape.class("node"))
    }

    pub(crate) fn render(&self, viewport: Viewport) -> SVGElement {
        let shape = self.shape.unwrap_or_default().render(viewport);

        let text = self
            .text
            .as_ref()
            .map(|text| SVGText::new(viewport.center()).render(text));

        Self::wrapper()
            .class_opt(self.class.as_ref())
            .child(shape.class("node"))
            .child_opt(text)
    }
}
