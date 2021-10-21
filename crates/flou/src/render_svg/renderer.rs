use std::{cmp::Ordering, convert::TryFrom, fmt::Display};

use crate::{
    parse::ast::Direction,
    parts::{Connection, Flou, NodeAttributes, Renderer},
    pos::{impl_pos_from, pos, IndexPos, PixelPos, Position2D},
    svg::{ArrowHead, SVGElement, SVGPath, SVGText},
};

use super::{path::get_path, viewport::Viewport};

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
    pub(crate) const PADDING: isize = 1;

    fn max(self, val: isize) -> Self {
        Self::new(std::cmp::max(self.x, val), std::cmp::max(self.y, val))
    }

    fn normalize(self) -> Self {
        let x = match self.x.cmp(&0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        };

        let y = match self.y.cmp(&0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
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

impl PixelPos {
    fn middle(a: Self, b: Self) -> Self {
        Self::new((a.x + b.x) / 2, (a.y + b.y) / 2)
    }
}

impl Direction {
    pub(crate) fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    pub(crate) fn rotate_clockwise(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            Direction::East => Direction::South,
        }
    }

    pub(crate) fn rotate_counter_clockwise(&self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
            Direction::East => Direction::North,
        }
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

        let connections = SVGElement::new("g")
            .class("connections")
            .children(self.render_connections(flou));

        let background = SVGElement::new("rect")
            .class("background")
            .pos(pos(0, 0))
            .size(size);

        let result = svg.child(background).child(nodes).child(connections);

        Box::new(result)
    }
}

impl SvgRenderer {
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

    fn render_connections<'i>(&self, flou: &'i Flou<'i>) -> Vec<SVGElement<'i>> {
        let mut connections = flou.connections.iter().collect::<Vec<_>>();

        connections.sort_unstable_by(|a, b| a.from.y.cmp(&b.from.y).then(a.from.x.cmp(&b.from.x)));

        connections
            .into_iter()
            .map(|c| self.render_connection(flou, c))
            .collect()
    }

    fn render_connection<'i>(&self, flou: &Flou<'i>, connection: &'i Connection) -> SVGElement<'i> {
        let path = get_path(&flou.grid, (connection.from, None), (connection.to, None));

        // It is assumed that path always has at least 2 points.
        let first_pair: &[PaddedPos] = &[path[1], path[0]];

        let link_points: Vec<_> = std::iter::once(first_pair)
            .chain(path.windows(2))
            .flat_map(<&[_; 2]>::try_from)
            .map(|&[from, to]| {
                let dir = PaddedPos::straight_line(to, from).unwrap();
                let link_point_offset = self.get_link_point_offset(flou, to, dir);
                let point = self.calculate_origin(to) + link_point_offset;
                (point, dir)
            })
            .collect();

        let mut path_svg = SVGPath::new();
        for (point, _) in &link_points {
            path_svg = path_svg.line_to(*point);
        }

        let arrowhead = {
            let (point, dir) = link_points.last().cloned().unwrap();
            let arrowhead_viewport = Viewport::new(point, pos(10, 10));
            ArrowHead::render(arrowhead_viewport, dir.reverse()).class("arrowhead")
        };

        let svg_text = connection.attrs.text.as_ref().map(|text| {
            let text_offset = 20;
            let len = std::cmp::min(3, link_points.len());
            let first_two = &link_points[..len];
            let text_origin = match *first_two {
                [from, to] if link_points.len() == 2 => {
                    PixelPos::middle(from.0, to.0)
                        + PixelPos::from(from.1.rotate_clockwise()) * text_offset
                }
                [_, second, third] => second.0 + PixelPos::from(third.1) * text_offset,
                // Again fine since it is assumed that path always has at least 2 points.
                _ => unreachable!(),
            };

            SVGText::new(text_origin)
                .render(text)
                .class("connection-text")
        });

        let path = path_svg.render().class("path");

        SVGElement::new("g")
            .class("connection")
            .class_opt(connection.attrs.class.as_ref())
            .child(path)
            .child(arrowhead)
            .child_opt(svg_text)
    }

    fn get_link_point_offset<'i>(
        &self,
        flou: &Flou<'i>,
        point: PaddedPos,
        dir: Direction,
    ) -> PixelPos {
        if !point.grid_aligned() {
            let x = if point.grid_x_aligned() {
                self.node.x / 2
            } else {
                self.grid_gap.x / 2
            };
            let y = if point.grid_y_aligned() {
                self.node.y / 2
            } else {
                self.grid_gap.y / 2
            };

            return pos(x, y);
        }

        let origin = self.calculate_node_origin(point.into());
        let viewport = Viewport::new(origin, self.node);
        match flou.node_attributes.get(&IndexPos::from(point)) {
            Some(attrs) => attrs.link_point(viewport, dir),
            None => NodeAttributes::default().link_point(viewport, dir),
        }
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
