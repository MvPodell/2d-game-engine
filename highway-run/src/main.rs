// TODO: use SPRITE instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine;
use engine::wgpu;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::{distributions::Uniform, Rng};
const W: f32 = 768.0;
const H: f32 = 1280.0;
const GUY_SPEED: f32 = 4.0;
const PAVEMENT_SPEED: f32 = -1.0;
const SPRITE_MAX: usize = 1000;
const COLLISION_DISTANCE: f32 = 22.0;
const DROP_OFF_DIST: f32 = 75.0;
const COP_DISTANCE: f32 = 42.0;
const COLLISION_STEPS: usize = 3;
const GUY_Y_POS: f32 = 24.0;
struct Guy {
    pos: Vec2,
    is_visible: bool,
}

struct Sprite {
    pos: Vec2,
    vel: Vec2,
}

// struct AudioState {
//     coin_collect_sound: StaticSoundData,
//     audio_manager: AudioManager<DefaultBackend>,
// }

enum GameState {
    TitleScreen,
    InGame,
    //GameOver, //eventually add the GameOver
}

struct Game {
    camera: engine::Camera,
    walls: Vec<SPRITE>,
    guy: Guy,
    cop: Guy,
    cars: Vec<Sprite>,
    car_timer: u32,
    car_speed_multiplier: f32,
    coin_speed_multiplier: f32,
    coins: Vec<Sprite>,
    coin_timer: u32,
    pavements: Vec<Sprite>,
    pavement_timer: u32,
    score: u32,
    font: engine::BitFont,
    curr_frame: usize,
    frame_counter: usize,
    frame_direction: isize,
    game_over: bool,
    game_state: GameState,
}

impl engine::Game for Game {
    // create new game instance
    fn new(engine: &mut Engine) -> Self {
        let camera = Camera {
            screen_pos: [0.0, 0.0],
            screen_size: [W, H],
        };
        #[cfg(target_arch = "wasm32")]
        let sprite_img = {
            let img_bytes = include_bytes!("../content/spritesheet.png");
            let img_bytes = include_bytes!("../content/spritesheet2.png");
            let img_bytes = include_bytes!("../content/spritesheet.png");
            image::load_from_memory_with_format(&img_bytes, image::ImageFormat::Png)
                .map_err(|e| e.to_string())
                .unwrap()
                .into_rgba8()
        };
        #[cfg(not(target_arch = "wasm32"))]
        let sprite_img = image::open("../content/spritesheet.png")
            .unwrap()
            .into_rgba8();
        let sprite_tex = engine.renderer.gpu.create_texture(
            &sprite_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            sprite_img.dimensions(),
            Some("spr-demo.png"),
        );

        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &sprite_tex,
            // &title_screen_tex,
            vec![Transform::zeroed(); SPRITE_MAX], //bg, three walls, guy, a few cars
            vec![SheetRegion::zeroed(); SPRITE_MAX],
            camera,
        );
        let guy = Guy {
            pos: Vec2 {
                x: 378.66,
                y: GUY_Y_POS,
            },
            is_visible: true,
        };
        let cop = Guy {
            pos: Vec2 {
                x: 378.66,
                y: -50.0,
            },
            is_visible: false,
        };

        let floor = SPRITE {
            center: Vec2 { x: W / 2.0, y: 8.0 },
            size: Vec2 { x: W, y: 16.0 },
        };

        let left_wall = SPRITE {
            center: Vec2 { x: 8.0, y: H / 2.0 },
            size: Vec2 { x: 288.0, y: H },
        };

        let right_wall = SPRITE {
            center: Vec2 {
                x: W - 8.0,
                y: H / 2.0,
            },
            size: Vec2 { x: 288.0, y: H },
        };

        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 0, 512, 0, 80, 8),
            10,
        );

        let mut pavements = Vec::with_capacity(34);
        // right pavement
        pavements.push(Sprite {
            pos: Vec2 { x: W - 2.0, y: 0.0 },
            vel: Vec2 { x: 0.0, y: -1.0 },
        });
        // create a left pavement
        pavements.push(Sprite {
            pos: Vec2 { x: 2.0, y: 0.0 },
            vel: Vec2 { x: 0.0, y: -1.0 },
        });
        let car_speed_multiplier = 1.0;
        let coin_speed_multiplier = 1.0;

        // let mut audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
        // let coin_collect_sound = StaticSoundData::from_file("/Users/rachelyang/game-engine-2d/content/coin.mp3", StaticSoundSettings::default())?;

        // let audio_state = AudioState {
        //     coin_collect_sound,
        //     audio_manager,
        // };

        Game {
            camera,
            guy,
            cop,
            walls: vec![left_wall, right_wall, floor],
            cars: Vec::with_capacity(8),
            car_timer: 0,
            coins: Vec::with_capacity(33),
            coin_timer: 0,
            car_speed_multiplier,
            coin_speed_multiplier,
            pavements,
            pavement_timer: 0,
            score: 0,
            font,
            curr_frame: 0,
            frame_counter: 0,
            frame_direction: 1,
            game_over: false,
            // audio_state: AudioState,
            game_state: GameState::TitleScreen,
        }
    }

    fn is_game_over(&self) -> bool {
        self.game_over
    }

    fn update(&mut self, engine: &mut Engine, acc: f32) {
        match self.game_state {
            GameState::TitleScreen => {
                // Check if the space bar is pressed
                if engine.input.is_key_pressed(engine::Key::Space) {
                    // Transition to the in-game state
                    self.game_state = GameState::InGame;
                }
            }

            GameState::InGame => {
                let mut now = std::time::Instant::now();
                // set the speed of animation for guy. Adjust number after modulo.
                self.frame_counter = (self.frame_counter + 1) % 5;
                if self.frame_counter == 0 {
                    // Update the current frame based on the direction
                    self.curr_frame = (self.curr_frame as isize + self.frame_direction) as usize;

                    // Change the direction if reaching the boundaries
                    if self.curr_frame == 2 || self.curr_frame == 0 {
                        self.frame_direction *= -1;
                    }
                }
                // column values
                let possible_values = [261.33, 378.66, 496.0];
                let side_values = [100.0, W-100.0];

                // for continuous left or right movement
                let dir = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
                self.guy.pos.x += dir * GUY_SPEED;
                self.cop.pos.x += dir * GUY_SPEED;

                // for continuous up or down movement
                let dir = engine.input.key_axis(engine::Key::Down, engine::Key::Up);
                self.guy.pos.y += dir * GUY_SPEED;
                self.cop.pos.y += dir * GUY_SPEED;

                let mut contacts = Vec::with_capacity(self.walls.len());

                for _iter in 0..COLLISION_STEPS {
                    let guy_sprite = SPRITE {
                        center: self.guy.pos,
                        size: Vec2 { x: 38.4, y: 65.33 },
                    };
                    contacts.clear();

                    contacts.extend(
                        self.walls
                            .iter()
                            .enumerate()
                            .filter_map(|(ri, w)| w.displacement(guy_sprite).map(|d| (ri, d))),
                    );
                    if contacts.is_empty() {
                        break;
                    }

                    contacts.sort_by(|(_r1i, d1), (_r2i, d2)| {
                        d2.length_squared()
                            .partial_cmp(&d1.length_squared())
                            .unwrap()
                    });
                    for (wall_idx, _disp) in contacts.iter() {
                        let guy_aabb = SPRITE {
                            center: self.guy.pos,
                            size: Vec2 { x: 38.4, y: 65.33 },
                        };
                        let wall = self.walls[*wall_idx];
                        let mut disp = wall.displacement(guy_sprite).unwrap_or(Vec2::ZERO);
                        // We got to a basically zero collision amount
                        if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                            break;
                        }
                        // Guy is left of wall, push left
                        if self.guy.pos.x < wall.center.x {
                            disp.x *= -1.0;
                        }
                        // Guy is below wall, push down
                        if self.guy.pos.y < wall.center.y {
                            disp.y *= -1.0;
                        }
                        if disp.x.abs() <= disp.y.abs() {
                            self.guy.pos.x += disp.x;
                            // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                        } else if disp.y.abs() <= disp.x.abs() {
                            self.guy.pos.y += disp.y;
                            // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                        }
                    }
                }
                let mut rng = rand::thread_rng();

                // create columns for cars
                let uniform = Uniform::new(0, possible_values.len());
                let random_index = rng.sample(uniform);
                let random_value = possible_values[random_index];

                // create columns for coins
                let uniform_coins = Uniform::new(0, side_values.len());
                let random_index_coins = rng.sample(uniform_coins);
                let random_value_coins = side_values[random_index_coins];

                // spawn new cars
                if self.car_timer > 0 {
                    self.car_timer -= 1;
                } else if self.cars.len() < 32 {
                    let mut valid_position = false;
                    let mut new_car_pos = Vec2::default();
                    while !valid_position {
                        let uniform = Uniform::new(0, possible_values.len());
                        let random_index = rng.sample(uniform);
                        new_car_pos = Vec2 {
                            x: possible_values[random_index],
                            y: H + 8.0,
                        };

                        // Check if the new position overlaps with existing cars
                        valid_position = !self
                            .cars
                            .iter()
                            .any(|car| new_car_pos.distance(car.pos) <= COLLISION_DISTANCE)
                            && !self
                                .coins
                                .iter()
                                .any(|coin| new_car_pos.distance(coin.pos) <= COLLISION_DISTANCE);
                    }

                    self.cars.push(Sprite {
                        pos: new_car_pos,
                        vel: Vec2 { x: 0.0, y: -2.0 },
                    });
                    self.car_timer = rng.gen_range(30..90);
                }
                // update car velocities every frame
                for car in self.cars.iter_mut() {
                    car.pos += car.vel;
                }

                // between frames, maintain all the cars on the screen that are above position -8.0
                self.cars.retain(|car| car.pos.y > -8.0);

                // if a coin is within the catch distance, add one to the score
                if let Some(idx) = self
                    .coins
                    .iter()
                    .position(|coin: &Sprite| coin.pos.distance(self.guy.pos) <= DROP_OFF_DIST)
                {
                    self.coins.swap_remove(idx);
                    self.score += 1
                }

                // self.coins.retain(|coin| coin.pos.y > -8.0);

                // Spawn new coins
                if self.coin_timer > 0 {
                    self.coin_timer -= 1;
                } else if self.coins.len() < 32 {
                    let mut valid_position = false;
                    let mut new_coin_pos = Vec2::default();
                    while !valid_position {
                        let uniform_coin = Uniform::new(0, side_values.len());
                        let random_index_coin = rng.sample(uniform_coin);
                        new_coin_pos = Vec2 {
                            x: side_values[random_index_coin],
                            y: H + 8.0,
                        };

                        // Check if the new position overlaps with existing cars or coins
                        valid_position = !self
                            .cars
                            .iter()
                            .any(|car| new_coin_pos.distance(car.pos) <= COLLISION_DISTANCE)
                            && !self
                                .coins
                                .iter()
                                .any(|coin| new_coin_pos.distance(coin.pos) <= COLLISION_DISTANCE);
                    }
                    self.coins.push(Sprite {
                        pos: new_coin_pos,
                        vel: Vec2 { x: 0.0, y: -2.0 },
                    });
                    self.coin_timer = rng.gen_range(30..90);
                }
                // Update coins
                for coin in self.coins.iter_mut() {
                    coin.pos += coin.vel;
                }
                self.coins.retain(|coin| coin.pos.y > -8.0);

                // Spawn new pavements
                if self.pavement_timer > 0 {
                    self.pavement_timer -= 1;
                } else if self.pavements.len() < 33 {
                    let newest_right_idx = self.pavements.len() - 2;
                    let newest_left_idx = self.pavements.len() - 1;
                    // create a right pavement
                    self.pavements.push(Sprite {
                        pos: Vec2 {
                            x: W - 2.0,
                            // add the next sprite one window height's length above the center of the most recently created sprite
                            y: self.pavements[newest_right_idx].pos[1] + H,
                        },
                        vel: Vec2 {
                            x: 0.0,
                            y: PAVEMENT_SPEED,
                        },
                    });
                    // create a left pavement
                    self.pavements.push(Sprite {
                        pos: Vec2 {
                            x: 2.0,
                            y: self.pavements[newest_left_idx].pos[1] + H,
                        },
                        vel: Vec2 { x: 0.0, y: -1.0 },
                    });
                    self.pavement_timer = rng.gen_range(30..90);
                }
                // Update pavements
                for pavement in self.pavements.iter_mut() {
                    pavement.pos += pavement.vel;
                }
                self.pavements.retain(|pavement| pavement.pos.y > -H / 2.0);

                // Increase speed multipliers over time
                self.car_speed_multiplier += 0.001 * acc;
                self.coin_speed_multiplier += 0.001 * acc;

                // Update cars with increased speed
                for car in self.cars.iter_mut() {
                    car.pos += car.vel * self.car_speed_multiplier;
                }

                // Update coins with increased speed
                for coin in self.coins.iter_mut() {
                    coin.pos += coin.vel * self.coin_speed_multiplier;
                }
            }
        }
    }
    fn render(&mut self, engine: &mut Engine) {
        let score_str = self.score.to_string();
        let text_len = score_str.len();

        let sprite_count =
            self.walls.len() + self.pavements.len() + self.cars.len() + self.coins.len() + 3;

        engine.renderer.sprites.resize_sprite_group(
            &engine.renderer.gpu,
            0,
            sprite_count + text_len,
        );

        match self.game_state {
            GameState::TitleScreen => {
                // let (transforms, uvs) = engine.renderer.sprites.get_sprites_mut(0);
                // transforms[0] = SPRITE {
                //     center: Vec2 {
                //         x: W / 2.0,
                //         y: H / 2.0,
                //     },
                //     size: Vec2 { x: W, y: H },
                // }
                // .into();
                // uvs[0] = SheetRegion::new(0, 0, 640, 480, 640, 480); // Adjust UV coordinates if needed
            }
            GameState::InGame => {
                let (transforms, uvs) = engine.renderer.sprites.get_sprites_mut(0);

                // set bg image
                transforms[0] = SPRITE {
                    center: Vec2 {
                        x: W / 2.0,
                        y: H / 2.0,
                    },
                    size: Vec2 { x: W, y: H },
                }
                .into();
                uvs[0] = SheetRegion::new(0, 0, 0, 16, 640, 480);

                // set walls
                const WALL_START: usize = 1;
                let guy_idx = WALL_START + self.walls.len();
                for (wall, (transform, uv)) in self.walls.iter().zip(
                    transforms[WALL_START..guy_idx]
                        .iter_mut()
                        .zip(uvs[WALL_START..guy_idx].iter_mut()),
                ) {
                    *transform = (*wall).into();
                    *uv = SheetRegion::new(0, 0, 480, 12, 8, 8);
                }
                // set guy
                transforms[guy_idx] = SPRITE {
                    center: self.guy.pos,
                    size: Vec2 { x: 38.4, y: 65.33 },
                }
                .into();

                // animate the guy character
                let ones_place = self.curr_frame % 10;
                match ones_place {
                    0 => {
                        uvs[guy_idx] = SheetRegion::new(0, 100, 498, 1, 14, 18);
                    }
                    1 => {
                        uvs[guy_idx] = SheetRegion::new(0, 114, 480, 1, 14, 18);
                    }
                    2 => {
                        uvs[guy_idx] = SheetRegion::new(0, 114, 498, 1, 14, 18);
                    }
                    _ => {
                        // for other cases, if they come up
                    }
                    }

                // set cop
                // let cop_idx = guy_idx + 1;
                // transforms[cop_idx] = SPRITE {
                //     center: self.cop.pos,
                //     size: Vec2 { x: 38.4, y: 65.33 },
                // }
                // .into();

                // animate the cop character
                // let ones_place = self.curr_frame % 10;
                // match ones_place {
                //     0 => {
                //         uvs[cop_idx] = SheetRegion::new(0, 177, 498, 0, 14, 18);
                //     }
                //     1 => {
                //         uvs[cop_idx] = SheetRegion::new(0, 191, 480, 0, 14, 18);
                //     }
                //     2 => {
                //         uvs[cop_idx] = SheetRegion::new(0, 191, 498, 0, 14, 18);
                //     }
                //     _ => {
                //         // for other cases, if they come up
                //     }
                // }

                // set pavement
                let pavement_start = guy_idx + 1;
                for (pavement, (transform, uv)) in self.pavements.iter().zip(
                    transforms[pavement_start..]
                        .iter_mut()
                        .zip(uvs[pavement_start..].iter_mut()),
                ) {
                    *transform = SPRITE {
                        center: pavement.pos,
                        size: Vec2 { x: 300.0, y: H },
                    }
                    .into();
                    *uv = SheetRegion::new(0, 640, 0, 5, 45, 748);
                }

                // set car
                let car_start = pavement_start + self.pavements.len();

                for (car, (transform, uv)) in self.cars.iter().zip(
                    transforms[car_start..]
                        .iter_mut()
                        .zip(uvs[car_start..].iter_mut()),
                ) {
                    *transform = SPRITE {
                        center: car.pos,
                        size: Vec2 { x: 38.4, y: 65.33 },
                    }
                    .into();
                    *uv = SheetRegion::new(0, 27, 525, 3, 27, 32);
                }

                // set coin
                let coin_start = car_start + self.cars.len();
                for (coin, (transform, uv)) in self.coins.iter().zip(
                    transforms[coin_start..]
                        .iter_mut()
                        .zip(uvs[coin_start..].iter_mut()),
                ) {
                    *transform = SPRITE {
                        center: coin.pos,
                        size: Vec2 { x: 33.0, y: 38.0 },
                    }
                    .into();
                    *uv = SheetRegion::new(0, 20, 480, 2, 16, 16);
                }

                let sprite_count = coin_start + self.coins.len();

                self.font.draw_text(
                    &mut engine.renderer.sprites,
                    0,
                    sprite_count,
                    &score_str,
                    Vec2 {
                        x: 16.0,
                        y: H - 16.0,
                    }
                    .into(),
                    16.0,
                );
                let text_start = coin_start + self.coins.len();
                engine.renderer.sprites.resize_sprite_group(
                    &engine.renderer.gpu,
                    0,
                    text_start + text_len,
                );
                engine.renderer.sprites.upload_sprites(
                    &engine.renderer.gpu,
                    0,
                    0..sprite_count + text_len,
                );
                engine
                    .renderer
                    .sprites
                    .set_camera_all(&engine.renderer.gpu, self.camera);
            }
        }
    }
}
fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}
