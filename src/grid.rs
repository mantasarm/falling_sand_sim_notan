use notan::{graphics::Texture, draw::{Draw, DrawImages}, prelude::{Graphics, App}, math::Vec2};

use crate::{element::*, movement::{downward, downward_sides, apply_velocity, apply_gravity, upward, sideways_gas}};

pub const COLS: usize = (1280. / 2.) as usize;
pub const ROWS: usize = (720. / 2.) as usize;

pub struct Grid {
	pos: (f32, f32),
	grid: Box<[[Cell; ROWS]; COLS]>,
	future_grid: Box<[[Cell; ROWS]; COLS]>,
	texture: Texture,
	bytes: Vec<u8>
}

impl Grid {
	pub fn new(x: f32, y: f32, gfx: &mut Graphics) -> Self {
		let bytes = vec![0; COLS * ROWS * 4];

		let texture = gfx
			.create_texture()
			.from_bytes(&bytes, COLS as i32, ROWS as i32)
			.build()
			.unwrap();

		let mut grid = create_cells_array();

		for i in 0..COLS {
			grid[i][0] = solid_element();
			grid[i][ROWS - 1] = solid_element();
		}
		for j in 0..ROWS {
			grid[0][j] = solid_element();
			grid[COLS - 1][j] = solid_element();
		}

		let future_grid = grid.clone();
		
		Self {
			pos: (x, y),
			grid,
			future_grid,
			texture,
			bytes
		}
	}

	pub fn update(&mut self) {
		self.future_grid = self.grid.clone();

		let flip_y = fastrand::bool();
		for mut i in 0..COLS {
			let flip_x = fastrand::bool();
			for mut j in 0..ROWS {
				if flip_x {
					j = ROWS - j - 1;
				}
				if flip_y {
					i = COLS - i - 1;
				}
				if self.grid[i][j].element == self.future_grid[i][j].element {
					match self.grid[i][j].element {
						Element::Sand => {
							apply_gravity(&mut self.future_grid, i, j);
							
							if !apply_velocity(&mut self.future_grid, i, j) {
								if !downward(&mut self.future_grid, i, j) {
									if !downward_sides(&mut self.future_grid, i, j) {
										self.future_grid[i][j].velocity = Vec2::ZERO;
									}
								}
							}
						},
						Element::SawDust => {
							apply_gravity(&mut self.future_grid, i, j);
							
							if !apply_velocity(&mut self.future_grid, i, j) {
								if !downward(&mut self.future_grid, i, j) {
									if !downward_sides(&mut self.future_grid, i, j) {
										self.future_grid[i][j].velocity = Vec2::ZERO;
									}
								}
							}
						},

						Element::Water => {
							apply_gravity(&mut self.future_grid, i, j);
							
							if !apply_velocity(&mut self.future_grid, i, j) {
								if !downward(&mut self.future_grid, i, j) {
									self.future_grid[i][j].velocity = Vec2::ZERO;

									let mut dir = 0.;

									if self.future_grid[i - 1][j].density < self.future_grid[i][j].density {
										dir = -1.;
									} else if self.future_grid[i + 1][j].density < self.future_grid[i][j].density {
										dir = 1.;
									}

									
									if dir != 0. {	
										self.future_grid[i][j].velocity.x = 5. * dir;
										self.future_grid[i][j].velocity.y = 1.;
									}
								}
							}
						},
						Element::Smoke => {
							if !upward(&mut self.future_grid, i, j) {
								sideways_gas(&mut self.future_grid, i, j, 10);
							}
						}
						_ => ()
					}
				}
			}
		}
		self.grid = self.future_grid.clone();
	}

	pub fn render(&mut self, gfx: &mut Graphics, draw: &mut Draw) {
		self.update_bytes();

		gfx.update_texture(&mut self.texture)
        	.with_data(&self.bytes)
        	.update()
        	.unwrap();
		
		draw.image(&self.texture).size(gfx.device.size().0 as f32, gfx.device.size().1 as f32).position(self.pos.0, self.pos.1);
	}

	fn update_bytes(&mut self) {
		for i in 0..self.bytes.len() / 4 {
			self.bytes[i * 4..i * 4 + 4].copy_from_slice(&self.grid[i % COLS][i / COLS].color);
		}
	}

	pub fn modify_elements(&mut self, i: usize, j: usize, brush_size: i32, cell: &Cell) {
		for x in -brush_size / 2..=brush_size / 2 {
			for y in -brush_size / 2..brush_size / 2 {
				if ((i as i32 - (i as i32 - x)).pow(2) + (j as i32 - (j as i32 - y)).pow(2)) <= (brush_size / 2).pow(2)  {
					self.modify_element((i as i32 - x) as usize, (j as i32 - y) as usize, cell);
				}
			}
		}
	}

	pub fn modify_element(&mut self, i: usize, j: usize, cell: &Cell) {
		if in_bound(i, j) {
			let mut c_cell = cell.to_owned();
			let amount = 40;
			let mut c = fastrand::u8(0..=amount);

			if c_cell.color[0] < c || c_cell.color[1] < c || c_cell.color[2] < c {
				c = 0;
			}
			
			c_cell.color = [cell.color[0] - c, cell.color[1] - c, cell.color[2] - c, 255];
			self.grid[i][j] = c_cell;
		}
	}

	pub fn explode(&mut self, i: usize, j: usize, radius: i32, force: f32) {
		for x in -radius / 2..=radius / 2 {
			for y in -radius / 2..radius / 2 {
				if ((i as i32 - (i as i32 - x)).pow(2) + (j as i32 - (j as i32 - y)).pow(2)) <= (radius / 2).pow(2)  {
					if in_bound((i as i32 - x) as usize, (j as i32 - y) as usize) {
						let mut angle = Vec2::new(x as f32, y as f32);
						angle = angle.normalize_or_zero() * force * -1.;

						self.grid[(i as i32 - x) as usize][(j as i32 - y) as usize].velocity += angle;
					}
				} 
			}
		}
	}

	pub fn get_cell(&self, i: usize, j: usize) -> &Cell {
		&self.grid[i][j]
	}

	pub fn mouse_in_sim(&self, app: &mut App) -> (usize, usize) {
		(((app.mouse.x - self.pos.0) / (app.window().width() as f32 / COLS as f32)) as usize, (app.mouse.y / (app.window().height() as f32 / ROWS as f32)) as usize)
	}
}

fn in_bound(i: usize, j: usize) -> bool {
	return i > 0 && j > 0 && i < COLS - 1 && j < ROWS - 1
}

pub fn create_cells_array() -> Box<[[Cell; ROWS]; COLS]> {
    let mut data = std::mem::ManuallyDrop::new(vec![air_element(); ROWS * COLS]);
    
    unsafe {
        Box::from_raw(data.as_mut_ptr() as *mut [[Cell; ROWS]; COLS])
    }
}