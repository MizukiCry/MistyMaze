use bevy::ecs::resource::Resource;
use rand::{Rng, seq::SliceRandom};

#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Open,
    Blocked,
    Safe,
}

#[derive(Default, Resource)]
pub struct Maze {
    pub width: usize,
    pub height: usize,
    pub origin: (usize, usize),
    /// cells size: (width, height)
    pub cells: Vec<Vec<Cell>>,
    pub coins: Vec<(usize, usize)>,

    /// room: (start_x, start_y, size_x, size_y)
    #[allow(dead_code)]
    pub rooms: Vec<(usize, usize, usize, usize)>,
}

#[allow(clippy::needless_range_loop)]
fn connect_rooms(cells: &mut [Vec<Cell>], mut p1: (usize, usize), mut p2: (usize, usize)) {
    // (p1.0, p1.1) -> (p2.0, p1.1)
    // (p2.0, p1.1) -> (p2.0, p2.1)

    if p1.0 > p2.0 {
        (p1, p2) = (p2, p1);
    }

    for i in p1.0..=p2.0 {
        if cells[i][p1.1] == Cell::Blocked {
            cells[i][p1.1] = Cell::Open;
        }
    }

    if p1.1 > p2.1 {
        (p1.1, p2.1) = (p2.1, p1.1);
    }

    for j in p1.1..=p2.1 {
        if cells[p2.0][j] == Cell::Blocked {
            cells[p2.0][j] = Cell::Open;
        }
    }
}

fn room_random_position(room: (usize, usize, usize, usize), rng: &mut impl Rng) -> (usize, usize) {
    (
        rng.random_range(room.0..room.0 + room.2),
        rng.random_range(room.1..room.1 + room.3),
    )
}

impl Maze {
    #[allow(clippy::needless_range_loop)]
    pub fn random(config: MazeConfig) -> Self {
        let mut cells = vec![vec![Cell::Blocked; config.height]; config.width];
        let mut rng = rand::rng();

        let mut is_safe = vec![true; config.safe_room_count];
        is_safe.extend(vec![false; config.room_count - config.safe_room_count]);
        is_safe.shuffle(&mut rng);

        // let mut room_origins: Vec<(usize, usize)> = Vec::new();
        let mut rooms: Vec<(usize, usize, usize, usize)> = Vec::new();

        for room in 0..config.room_count {
            let mut width;
            let mut height;
            let mut x;
            let mut y;

            loop {
                width = rng.random_range(config.room_size_min..=config.room_size_max);
                height = rng.random_range(config.room_size_min..=config.room_size_max);
                x = rng.random_range(1..config.width - width - 1);
                y = rng.random_range(1..config.height - height - 1);

                let mut ok = true;
                for prev in &rooms {
                    let &(prev_x, prev_y, prev_width, prev_height) = prev;
                    if x.max(prev_x) < (x + width).min(prev_x + prev_width)
                        && y.max(prev_y) < (y + height).min(prev_y + prev_height)
                    {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    break;
                }
            }

            let is_safe = is_safe[room];
            for i in x..x + width {
                for j in y..y + height {
                    if is_safe {
                        cells[i][j] = Cell::Safe;
                    } else if cells[i][j] == Cell::Blocked {
                        cells[i][j] = Cell::Open;
                    }
                }
            }

            rooms.push((x, y, width, height));
        }

        for i in 0..config.room_count {
            connect_rooms(
                &mut cells,
                room_random_position(rooms[i], &mut rng),
                room_random_position(rooms[(i + 1) % config.room_count], &mut rng),
            );
        }

        // let mut edges: Vec<(usize, usize)> = Vec::new();
        // for i in 0..config.room_count {
        //     for j in i + 2..config.room_count {
        //         edges.push((i, j));
        //     }
        // }
        // edges.shuffle(&mut rng);
        // edges.shrink_to(config.room_count / 2);

        // for (i, j) in edges {
        //     connect_rooms(
        //         &mut cells,
        //         room_random_position(rooms[i], &mut rng),
        //         room_random_position(rooms[j], &mut rng),
        //     );
        // }

        let mut origin = (0, 0);
        for i in 0..config.room_count {
            if is_safe[i] {
                origin = room_random_position(rooms[i], &mut rng);
                break;
            }
        }

        let mut coins = Vec::new();
        for x in 0..config.width {
            for y in 0..config.height {
                if cells[x][y] != Cell::Blocked
                    && (x, y) != origin
                    && rng.random_range(0.0..1.0) < config.coin_probability
                {
                    coins.push((x, y));
                }
            }
        }

        Maze {
            width: config.width,
            height: config.height,
            origin,
            cells,
            coins,
            rooms,
        }
    }
}

pub struct MazeConfig {
    pub width: usize,
    pub height: usize,
    pub room_size_min: usize,
    pub room_size_max: usize,
    pub room_count: usize,
    pub safe_room_count: usize,
    pub coin_probability: f32,
}

impl MazeConfig {
    /// Creates a new `MazeConfig` with the given width and height.
    /// The width and height are clamped to a minimum of 12.
    pub fn new(width: usize, height: usize) -> Self {
        let width = width.max(12);
        let height = height.max(12);
        let room_size_min: usize = width.min(height).ilog2() as usize;
        let room_size_max: usize = (width.min(height) as f32).powf(0.7).floor() as usize;
        let room_count = ((width - 1) * (height - 1) / (room_size_max * room_size_max)).max(3);
        Self {
            width,
            height,
            room_size_min,
            room_size_max,
            room_count,
            safe_room_count: room_count.ilog2() as usize,
            coin_probability: 0.5,
        }
    }

    #[allow(dead_code)]
    pub fn check(&self) -> bool {
        self.room_size_min <= self.room_size_max
            && self.room_size_min > 0
            && self.width.min(self.height) >= (self.room_size_max + 2).max(10)
            && self.room_count > 0
            && self.safe_room_count <= self.room_count
    }
}
