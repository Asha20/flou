use crate::{
    parse::ast::Direction,
    pos::{pos, PixelPos},
    render_svg::Viewport,
    svg::{SVGElement, SVGPath},
};

#[derive(Debug, PartialEq)]
struct ArrowheadPoints {
    tip: PixelPos,
    center: PixelPos,
    left_corner: PixelPos,
    right_corner: PixelPos,
}

impl ArrowheadPoints {
    pub(crate) fn render(self) -> SVGElement<'static> {
        SVGPath::new()
            .line_to(self.tip)
            .line_to(self.left_corner)
            .line_to(self.center)
            .line_to(self.right_corner)
            .line_to(self.tip)
            .render()
    }
}

pub(crate) struct ArrowHead;

impl ArrowHead {
    /// `viewport.origin` is the tip of the arrowhead.
    /// `viewport.size.x` is the wingspan of the arrowhead.
    /// `viewport.size.y` is the length of the arrowhead.
    /// `dir` is the direction the arrowhead is facing.
    pub(crate) fn render(viewport: Viewport, dir: Direction) -> SVGElement<'static> {
        Self::get_points(viewport, dir).render()
    }

    fn get_points(viewport: Viewport, dir: Direction) -> ArrowheadPoints {
        let dir = dir.reverse();

        let size = match dir {
            Direction::East | Direction::West => pos(viewport.size.y, viewport.size.x),
            _ => viewport.size,
        };

        let center = viewport.origin + PixelPos::from(dir) * size / 2;

        let left_corner = PixelPos::from(dir) + PixelPos::from(dir.rotate_clockwise());
        let left_corner = center + left_corner * size / 2;

        let right_corner = PixelPos::from(dir) + PixelPos::from(dir.rotate_counter_clockwise());
        let right_corner = center + right_corner * size / 2;

        ArrowheadPoints {
            tip: viewport.origin,
            center,
            left_corner,
            right_corner,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse::ast::Direction, pos::pos, render_svg::Viewport, test::assert_eq};

    use super::{ArrowHead, ArrowheadPoints};

    #[test]
    fn points_are_correct() {
        let viewport = Viewport::new(pos(100, 100), pos(20, 40));
        let actual = ArrowHead::get_points(viewport, Direction::North);

        assert_eq!(
            actual,
            ArrowheadPoints {
                tip: pos(100, 100),
                center: pos(100, 120),
                left_corner: pos(90, 140),
                right_corner: pos(110, 140)
            }
        );

        let viewport = Viewport::new(pos(200, 200), pos(20, 40));
        let actual = ArrowHead::get_points(viewport, Direction::East);

        assert_eq!(
            actual,
            ArrowheadPoints {
                tip: pos(200, 200),
                center: pos(180, 200),
                left_corner: pos(160, 190),
                right_corner: pos(160, 210),
            }
        )
    }
}
