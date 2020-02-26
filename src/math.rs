use crate::raw::RawPoint;
use crate::simd::*;
use alloc::vec::*;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Curve {
    a: Point,
    b: Point,
    c: Point,
}

impl Curve {
    fn new(a: Point, b: Point, c: Point) -> Curve {
        Curve {
            a,
            b,
            c,
        }
    }

    fn at(&self, t: f32) -> Point {
        let x = (1.0 - t).powi(2) * self.a.x + 2.0 * (1.0 - t) * t * self.b.x + t.powi(2) * self.c.x;
        let y = (1.0 - t).powi(2) * self.a.y + 2.0 * (1.0 - t) * t * self.b.y + t.powi(2) * self.c.y;
        Point::new(x, y)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    /// Absolute X coordinate.
    pub x: f32,
    /// Absolute Y coordinate.
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point {
            x,
            y,
        }
    }

    pub fn raw(p: &RawPoint) -> Point {
        Point {
            x: p.x,
            y: p.y,
        }
    }

    pub fn midpoint_raw(a: &RawPoint, b: &RawPoint) -> Point {
        Point {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Line {
    /// X0, Y0, X1, Y1.
    pub coords: f32x4,
    pub x_mod: f32,
    pub y_mod: f32,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Line {
        let x_mod = if end.x >= start.x {
            1.0
        } else {
            0.0
        };
        let y_mod = if end.y >= start.y {
            1.0
        } else {
            0.0
        };
        Line {
            coords: f32x4::new(start.x, start.y, end.x, end.y),
            x_mod,
            y_mod,
        }
    }
}

pub struct Geometry {
    pub lines: Vec<Line>,
}

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            lines: Vec::new(),
        }
    }

    pub fn push(&mut self, start: Point, end: Point) {
        if start.y != end.y {
            self.lines.push(Line::new(start, end));
        }
    }
}

const SUBDIVISIONS: u32 = 3;

fn populate_lines(geometry: &mut Geometry, previous: &RawPoint, current: &RawPoint, next: &RawPoint) {
    if !current.on_curve() {
        // Curve. We're off the curve, find the on-curve positions for the previous and next points
        // then make a curve out of that.
        let previous = if previous.on_curve() {
            Point::raw(&previous)
        } else {
            Point::midpoint_raw(&previous, current)
        };
        let next = if next.on_curve() {
            Point::raw(&next)
        } else {
            Point::midpoint_raw(current, &next)
        };
        let current = Point::raw(current);
        let curve = Curve::new(previous, current, next);

        if SUBDIVISIONS <= 1 {
            geometry.push(previous, current);
            geometry.push(current, next);
        } else {
            let increment = 1.0 / (SUBDIVISIONS as f32);
            for x in 0..SUBDIVISIONS {
                let t0 = increment * (x as f32);
                let t1 = increment * ((x + 1) as f32);
                let p0 = curve.at(t0);
                let p1 = curve.at(t1);
                geometry.push(p0, p1);
            }
        }
    } else if next.on_curve() {
        // Line. Both the current and the next point are on the curve, it's a line.
        geometry.push(Point::raw(current), Point::raw(next));
    } else {
        // Do nothing. The current point is on the curve but the next one isn't, so the next point
        // will end up drawing the curve that the current point is on.
    }
}

pub fn compile(points: &[RawPoint]) -> Geometry {
    let mut geometry = Geometry::new();
    let mut first = RawPoint::default();
    let mut second = RawPoint::default();
    let mut previous = RawPoint::default();
    let mut current = RawPoint::default();
    let mut index = 0;
    for next in points {
        match index {
            0 => {
                first = *next;
                previous = *next;
            }
            1 => {
                second = *next;
                current = *next;
            }
            _ => {
                populate_lines(&mut geometry, &previous, &current, next);
                if next.end_point {
                    populate_lines(&mut geometry, &current, next, &first);
                    populate_lines(&mut geometry, next, &first, &second);
                    index = -1;
                } else {
                    previous = current;
                    current = *next;
                }
            }
        }
        index += 1;
    }
    geometry
}
