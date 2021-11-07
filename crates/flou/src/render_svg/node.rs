use crate::{
    parse::ast::{Direction, NodeShape},
    parts::NodeAttributes,
    pos::{pos, PixelPos},
    svg::{SVGElement, SVGPath, SVGText},
};

use super::viewport::{Midpoints, Viewport};

impl Default for NodeShape {
    fn default() -> Self {
        Self::Rectangle
    }
}

impl NodeShape {
    pub(crate) fn render(&self, viewport: Viewport) -> SVGElement<'static> {
        match &self {
            Self::Rectangle => SVGElement::new("rect")
                .class("rect")
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
                    .class("circle")
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

    pub(crate) fn link_point(&self, viewport: Viewport, dir: Direction) -> PixelPos {
        match &self.shape.unwrap_or_default() {
            NodeShape::Circle | NodeShape::Square | NodeShape::AngledSquare => {
                let radius = std::cmp::min(viewport.size.x, viewport.size.y) / 2;
                let center = viewport.center();

                // Calculate the midpoints as offsets from the center of the
                // viewport because it's simpler, but then subtract the origin
                // (top-left corner) because the offsets need to be relative to *it*.
                let midpoints = Midpoints {
                    top: center + pos(0, -radius) - viewport.origin,
                    bottom: center + pos(0, radius) - viewport.origin,
                    left: center + pos(-radius, 0) - viewport.origin,
                    right: center + pos(radius, 0) - viewport.origin,
                };

                midpoints.get_from_direction(dir)
            }
            _ => viewport.midpoints_relative().get_from_direction(dir),
        }
    }
}
