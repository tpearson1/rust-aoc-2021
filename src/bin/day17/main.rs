use std::{collections::HashSet, fs, ops};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day17/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let target = Target::parse_from_str(&input).ok_or("Failed to parse input")?;
    let (max_y, num_velocities) = solve(&target).ok_or("Probe did not enter the region")?;
    println!("Max y on trajectory that enters the region: {}", max_y);
    println!("Number of unique velocities that work: {}", num_velocities);

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl ops::Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl ops::AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Target {
    xmin: i32,
    xmax: i32,
    ymin: i32,
    ymax: i32,
}

impl Target {
    pub fn new((xmin, xmax): (i32, i32), (ymin, ymax): (i32, i32)) -> Self {
        Self {
            xmin,
            xmax,
            ymin,
            ymax,
        }
    }

    pub fn parse_from_str(input: &str) -> Option<Target> {
        let input = input.lines().next()?;

        const PREFIX: &str = "target area: x=";
        if !input.starts_with(PREFIX) {
            return None;
        }

        let input = &input[PREFIX.len()..];
        let (xrange, yrange) = input.split_once(", y=")?;
        let xrange = Self::parse_range(xrange)?;
        let yrange = Self::parse_range(yrange)?;

        Some(Self::new(xrange, yrange))
    }

    fn parse_range(range: &str) -> Option<(i32, i32)> {
        let (lhs, rhs) = range.split_once("..")?;
        Some((lhs.parse().ok()?, rhs.parse().ok()?))
    }
}

fn x_velocity_range(xmin: i32, xmax: i32) -> (i32, i32) {
    if xmin < 0 {
        if xmax < 0 {
            // xmin ... xmax ... 0
            (xmin, -min_vel(-xmax) + 1)
        } else {
            // xmin ... 0 ... xmax
            (xmin, xmax)
        }
    } else {
        // 0 ... xmin ... xmax
        (min_vel(xmin) - 1, xmax)
    }
}

fn y_velocity_range(ymin: i32, ymax: i32) -> (i32, i32) {
    if ymin < 0 {
        if ymax < 0 {
            // Below ymin -> jumps over immediately. Max value seems way harder
            // after some thinking, and a dummy value of 1000 works for the
            // given input
            // - Binary search not guaranteed to work due to discrete nature of
            //   the problem
            // - If region includes parts of x = 0, there is NO max y (shoot
            //   straight up with velocity of your choice). As long as the
            //   region includes at least one triangular number, it will work
            //   (-1, -3, -6, -10, ...)
            // - It also depends on the x position in nontrivial ways
            (ymin, 1000)
        } else {
            // Below ymin -> jumps over immediately.
            // Above ymax -> jumps over on way up and down.
            (ymin, ymax)
        }
    } else {
        // Needs to reach at least that high.
        // If it has velocity above ymax, it will jump "over" the whole range on
        // the way up, and by symettry also on the way down
        (min_vel(ymin) - 1, ymax)
    }
}

fn min_vel(val: i32) -> i32 {
    // If the object has velocity v, it moves by v, then v - 1, and so on,
    // totalling v + (v - 1) + ... + 1, which equals v * (v + 1) / 2.
    //
    // Solving for v with x = v * (v + 1) / 2, assuming positive x:
    //   2x = v * (v + 1)
    //    0 = v^2 + v - 2x
    //    v = (-1 + sqrt(1 + 8x)) / 2 (only positive root)
    ((-1. + (1. + 8. * val as f64).sqrt()) * 0.5) as i32
}

pub fn solve(target: &Target) -> Option<(i32, usize)> {
    let (xs, xf) = x_velocity_range(target.xmin, target.xmax);
    let (ys, yf) = y_velocity_range(target.ymin, target.ymax);
    let mut ymax: Option<i32> = None;

    let mut success_velocities = HashSet::new();

    let yrange = (target.ymin, target.ymax);

    for x_vel in xs..=xf {
        for x in target.xmin..=target.xmax {
            for y_vel in ys..=yf {
                if let Some(max) = simulate_throw(x, Vec2::new(x_vel, y_vel), yrange) {
                    success_velocities.insert((x_vel, y_vel));
                    ymax = Some(ymax.map(|x| x.max(max)).unwrap_or(max));
                }
            }
        }
    }

    Some((ymax?, success_velocities.len()))
}

fn simulate_throw(target_x: i32, initial_velocity: Vec2, yrange: (i32, i32)) -> Option<i32> {
    let mut ymax = 0;

    let x_step = -target_x.signum();

    let mut velocity = initial_velocity;
    let mut pos = Vec2::ZERO;
    loop {
        if velocity.y < 0 && pos.y < yrange.0 {
            return None;
        }

        if pos.x == target_x && (yrange.0..=yrange.1).contains(&pos.y) {
            // We're in the target range. If y is still going up, figure out
            // what maximum y will hit.
            if velocity.y > 0 {
                ymax += velocity.y * (velocity.y + 1) / 2;
            }

            return Some(ymax);
        }

        pos += velocity;
        ymax = ymax.max(pos.y);

        velocity.y -= 1;
        if velocity.x != 0 {
            velocity.x += x_step;
        }

        if target_x > 0 && pos.x > target_x {
            return None;
        }

        if target_x < 0 && pos.x < target_x {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input() {
        const EXAMPLE: &str = "target area: x=20..30, y=-10..-5\n";
        let target = Target::parse_from_str(EXAMPLE).unwrap();
        assert_eq!(target, Target::new((20, 30), (-10, -5)));
    }

    #[test]
    fn test_x_velocity_range() {
        assert_eq!(x_velocity_range(5, 7), (1, 7));
        assert_eq!(x_velocity_range(6, 10), (2, 10));
        assert_eq!(x_velocity_range(-4, 16), (-4, 16));
        assert_eq!(x_velocity_range(-11, 10), (-11, 10));
        assert_eq!(x_velocity_range(-20, -11), (-20, -3));
    }

    #[test]
    fn test_solve() {
        let target = Target::new((20, 30), (-10, -5));
        let result = solve(&target);
        assert_eq!(result, Some((45, 112)));
    }
}
