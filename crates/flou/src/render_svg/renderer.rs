use std::fmt::Display;

use crate::{
    parts::{Flou, NodeAttributes, Renderer},
    pos::{impl_pos_from, pos, IndexPos, PixelPos, Position2D},
    svg::SVGElement,
};

use super::viewport::Viewport;

impl Default for SvgRenderer {
    fn default() -> Self {
        Self {
            node: pos(200, 100),
            grid_gap: pos(50, 50),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct PaddedSpace;
pub(crate) type PaddedPos = Position2D<isize, PaddedSpace>;

impl_pos_from!(PaddedPos, PixelPos, i32);

impl PaddedPos {
    const PADDING: isize = 1;

    fn snap_to_grid(self) -> Self {
        let res: IndexPos = self.into();
        res.into()
    }

    fn max(self, val: isize) -> Self {
        Self::new(std::cmp::max(self.x, val), std::cmp::max(self.y, val))
    }

    fn normalize(self) -> Self {
        let x = match self.x {
            _ if self.x > 0 => 1,
            _ if self.x < 0 => -1,
            _ => 0,
        };

        let y = match self.y {
            _ if self.y > 0 => 1,
            _ if self.y < 0 => -1,
            _ => 0,
        };

        Self::new(x, y)
    }
}

impl From<IndexPos> for PaddedPos {
    fn from(other: IndexPos) -> Self {
        let res = other * (PaddedPos::PADDING + 1) + PaddedPos::PADDING;
        Self::new(res.x, res.y)
    }
}

impl From<PaddedPos> for IndexPos {
    fn from(other: PaddedPos) -> Self {
        let res = (other - PaddedPos::PADDING) / (PaddedPos::PADDING + 1);
        Self::new(res.x, res.y)
    }
}

pub struct SvgRenderer {
    node: PixelPos,
    grid_gap: PixelPos,
}

impl Renderer for SvgRenderer {
    fn render<'i>(&self, flou: &'i Flou<'i>) -> Box<dyn Display + 'i> {
        let style = SVGElement::new("style").text(include_str!("../css/default.css"));
        let size = self.calculate_svg_size(flou.grid.size);

        let svg = SVGElement::new("svg")
            .attr("xmlns", "http://www.w3.org/2000/svg")
            .size(size)
            .child(style);

        let nodes = SVGElement::new("g")
            .class("nodes")
            .children(self.render_nodes(flou));

        let background = SVGElement::new("rect")
            .class("background")
            .pos(pos(0, 0))
            .size(size);

        let result = svg.child(background).child(nodes);

        Box::new(result)
    }
}

impl SvgRenderer {
    fn render_nodes<'i>(&self, flou: &'i Flou<'i>) -> Vec<SVGElement<'i>> {
        let mut positions = flou
            .grid
            .position_to_id
            .iter()
            .map(|(&pos, _)| pos)
            .collect::<Vec<_>>();

        positions.sort_unstable_by(|a, b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));

        positions
            .into_iter()
            .map(|pos| {
                let origin = self.calculate_node_origin(pos);
                let viewport = Viewport::new(origin, self.node);

                match flou.node_attributes.get(&pos) {
                    Some(node_attrs) => node_attrs.render(viewport),
                    None => NodeAttributes::render_default(viewport),
                }
            })
            .collect()
    }

    fn calculate_node_origin(&self, pos: IndexPos) -> PixelPos {
        let node_offset: PixelPos = pos.into();
        let num_grid_gaps = (node_offset + 1) * PaddedPos::PADDING as i32;

        node_offset * self.node + num_grid_gaps * self.grid_gap
    }

    fn calculate_origin(&self, pos: PaddedPos) -> PixelPos {
        let aligned_pos = pos.snap_to_grid();
        let grid_distance = pos - aligned_pos;

        // Pos of the origin of the nearest node, which is grid-aligned.
        let node_offset = self.calculate_node_origin(aligned_pos.into());

        // If the connection point isn't grid-aligned, then it's past the nearest node.
        let norm_distance = PixelPos::from(grid_distance.normalize()) * self.node;

        // Include the distance to the nearest node if the position isn't grid-aligned.
        let grid_distance = (grid_distance - 1).max(0);
        let grid_offset = PixelPos::from(grid_distance) * self.grid_gap;

        node_offset + norm_distance + grid_offset
    }

    fn calculate_svg_size(&self, grid_size: IndexPos) -> PixelPos {
        self.calculate_origin(grid_size.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{pos::pos, test::assert_eq};

    use super::SvgRenderer;

    #[test]
    fn calculates_origin_without_grid_gap() {
        let renderer = SvgRenderer {
            node: pos(50, 100),
            grid_gap: pos(0, 0),
        };

        let actual = renderer.calculate_node_origin(pos(0, 0));
        assert_eq!(actual, pos(0, 0));

        let actual = renderer.calculate_node_origin(pos(2, 0));
        assert_eq!(actual, pos(100, 0));

        let actual = renderer.calculate_node_origin(pos(1, 3));
        assert_eq!(actual, pos(50, 300));
    }

    #[test]
    fn calculates_origin_with_grid_gap() {
        let renderer = SvgRenderer {
            node: pos(50, 100),
            grid_gap: pos(10, 20),
        };

        let actual = renderer.calculate_node_origin(pos(0, 0));
        assert_eq!(actual, pos(10, 20));

        let actual = renderer.calculate_node_origin(pos(2, 0));
        assert_eq!(actual, pos(130, 20));

        let actual = renderer.calculate_node_origin(pos(1, 3));
        assert_eq!(actual, pos(70, 380));
    }
}
