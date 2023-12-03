use engine;
use engine::wgpu;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::{distributions::Uniform, Rng};
const W: f32 = 768.0;
const H: f32 = 1280.0;
// const GUY_SPEED: f32 = 4.0;
const PAVEMENT_SPEED: f32 = -1.0;
const SPRITE_MAX: usize = 1000;
const COLLISION_DISTANCE: f32 = 22.0;
// sound/audio --> use Kira
use kira::{
    manager::{
        AudioManager, AudioManagerSettings,
        backend::DefaultBackend,
    },
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

const COP_DISTANCE: f32 = 42.0;
const COLLISION_STEPS: usize = 3;
const GUY_Y_POS: f32 = 24.0;
struct Guy {
    pos: Vec2,
    is_jumping: bool,
    jump_velocity: f32,
    fwd_jump_frames: usize,
    is_visible: bool,
}

struct Sprite {
    pos: Vec2,
    vel: Vec2,
}

enum GameState {
    TitleScreen,
    InGame,
    GameOver, 
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
    font_end: engine::BitFont,
    curr_frame: usize,
    frame_counter: usize,
    frame_direction: isize,
    game_over: bool,
    game_state: GameState,
    // coin sound
    audio_manager: AudioManager<DefaultBackend>,
    coin_sound: StaticSoundData,
}

impl engine::Game for Game {
    // create new game instance
    fn new(engine: &mut Engine) -> Self {
        let camera = Camera {
            screen_pos: [0.0, 0.0],
            screen_size: [W, H],
        };

        #[cfg(not(target_arch = "wasm32"))]
        let start_img = image::open("../content/title_screen2.png").unwrap().into_rgba8();
        let start_tex = engine.renderer.gpu.create_texture(&start_img, wgpu::TextureFormat::Rgba8UnormSrgb, start_img.dimensions(), Some("start-sprite.png"),);
        let end_img = image::open("../content/end_screen.png").unwrap().into_rgba8();
        let end_tex = engine.renderer.gpu.create_texture(&end_img, wgpu::TextureFormat::Rgba8UnormSrgb, end_img.dimensions(), Some("end-sprite.png"),);

        let sprite_img = image::open("../content/spritesheet.png")
            .unwrap()
            .into_rgba8();
        let sprite_tex = engine.renderer.gpu.create_texture(
            &sprite_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            sprite_img.dimensions(),
            Some("spr-demo.png"),
        );

        // game sprite group
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &sprite_tex,
            // &title_screen_tex,
            vec![Transform::zeroed(); SPRITE_MAX], //bg, three walls, guy, a few cars
            vec![SheetRegion::zeroed(); SPRITE_MAX],
            camera,
        );

        // start sprite group
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &start_tex,
            vec![Transform::zeroed(); 1], //bg, three walls, guy, a few cars
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        // end sprite group
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &end_tex,
            vec![Transform::zeroed(); 1], //bg, three walls, guy, a few cars
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        let guy = Guy {
            pos: Vec2 {
                x: 378.66,
                y: GUY_Y_POS,
            },
            is_jumping: false,
            jump_velocity: 10.0,
            fwd_jump_frames: 0,
            is_visible: true,
        };
        let cop = Guy {
            pos: Vec2 {
                x: 378.66,
                y: -50.0,
            },
            is_jumping: false,
            jump_velocity: 10.0,
            fwd_jump_frames: 0,
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
        let font_end = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 0, 868, 0, 80, 8),
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

        // coin sound
        // Create an audio manager
        let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
        // Load the coin sound
        let coin_sound = StaticSoundData::from_file("/Users/rachelyang/game-engine-2d/content/coin.mp3", StaticSoundSettings::default()).unwrap();


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
            font_end,
            curr_frame: 0,
            frame_counter: 0,
            frame_direction: 1,
            game_over: false,
            game_state: GameState::TitleScreen,
            // coin sound
            audio_manager,
            coin_sound,
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
                    engine.renderer.sprites.remove_sprite_group(1);
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
                let mut curr_col = self.guy.pos.x;
                let position = possible_values
                    .iter()
                    .position(|&r| (curr_col - r).abs() < 1.0);

                let mut curr_index = position.map(|index| index as f32).unwrap_or_default();

                // calculate x position of character
                // for left or right movement in defined steps (possible_values)
                let mut dir = 0.0;
                if engine.input.is_key_pressed(engine::Key::Left) {
                    dir = -1.0
                } else if engine.input.is_key_pressed(engine::Key::Right) {
                    dir = 1.0
                }

                // for jumping
                if engine.input.is_key_pressed(engine::Key::Up) && !self.guy.is_jumping {
                    println!("jump!");
                    self.guy.is_jumping = true;
                }

                // track the number of frames the guy has been jumping for
                if self.guy.is_jumping {
                    self.guy.fwd_jump_frames += 1;

                    // Continue the animation for 12 frames
                    if self.guy.fwd_jump_frames <= 12 {
                        // make the distance traveled between frames progressively less
                        self.guy.pos.y += self.guy.jump_velocity;
                        self.guy.jump_velocity -= 0.2;
                    } else if self.guy.pos.y >= 50.0 {
                        self.guy.pos.y -= 2.3;
                    } else {
                        // End the jumping animation
                        self.guy.is_jumping = false;
                        self.guy.fwd_jump_frames = 0;
                        self.guy.pos.y = 50.0;
                        self.guy.jump_velocity = 10.0;
                    }

                    if self.cop.is_visible && self.cop.fwd_jump_frames <= 100 {
                        self.cop.fwd_jump_frames += 1;
                    }

                    if self.cop.fwd_jump_frames > 100 && self.cop.pos.y >= 0.0 {
                        self.cop.pos.y -= 1.0;
                    } else if self.cop.pos.y < 0.0 && self.cop.is_visible {
                        // end cop visibility
                        self.cop.fwd_jump_frames = 0;
                        self.cop.is_visible = false;
                        self.cop.pos.y = -50.0;
                    }
                }

                // for continuous left or right movement
                // let dir = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
                if -1.0 < curr_index as f32 + dir && curr_index as f32 + dir < 3.0 {
                    let curr_loc = curr_index + dir;
                    curr_col = possible_values[curr_loc as usize];
                }

                // update character's column
                self.guy.pos.x = curr_col;
                self.cop.pos.x = curr_col;

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
                        // TODO: for multiple guys should access self.guys[guy_idx].
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

                // create columns for cars/coins
                let uniform = Uniform::new(0, possible_values.len());
                let random_index = rng.sample(uniform);
                let random_value = possible_values[random_index];

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
                    // self.cars.push(Car {
                    //     pos: Vec2 {
                    //         x: random_value,
                    //         y: H + 8.0,
                    //     },
                    //     vel: Vec2 {
                    //         x: 0.0,
                    //         // y: rng.gen_range((-4.0)..(-1.0)),
                    //         y: -2.0,
                    //     },
                    // });
                    self.car_timer = rng.gen_range(30..90);
                }
                // update car velocities every frame
                for car in self.cars.iter_mut() {
                    car.pos += car.vel;
                }
                // if any car is within the catch distance of the guy, mark a collision
                if !self.guy.is_jumping {
                    if let Some(idx) = self
                        .cars
                        .iter()
                        .position(|car| car.pos.distance(self.guy.pos) <= COLLISION_DISTANCE)
                    {
                        println!("Score: {}", self.score);
                        engine.renderer.sprites.remove_sprite_group(0);
                        self.game_state = GameState::GameOver;
                    } else if let Some(idx) = self
                        .cars
                        .iter()
                        .position(|car| car.pos.distance(self.guy.pos) <= COP_DISTANCE)
                    {
                        println!("COP!");
                        if !self.cop.is_visible {
                            self.cop.is_visible = true;
                            self.guy.pos.y = GUY_Y_POS + 100.0;
                            self.cop.pos.y = GUY_Y_POS;
                            // if the cop is already on the screen and it's been on the screen for more than the collision cooldown of 50 frames
                        } else if self.cop.is_visible && self.cop.fwd_jump_frames > 50 {
                            self.game_over = true;
                        }
                    }
                }
                // between frames, maintain all the cars on the screen that are above position -8.0
                self.cars.retain(|car| car.pos.y > -8.0);

                // if a coin is within the catch distance, add one to the score
                if let Some(idx) = self
                    .coins
                    .iter()
                    .position(|coin: &Sprite| coin.pos.distance(self.guy.pos) <= COLLISION_DISTANCE)
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
                        let uniform = Uniform::new(0, possible_values.len());
                        let random_index = rng.sample(uniform);
                        new_coin_pos = Vec2 {
                            x: possible_values[random_index],
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
                    // self.coins.push(Coin {
                    //     pos: Vec2 {
                    //         x: random_value,
                    //         y: H + 8.0,
                    //     },
                    //     vel: Vec2 {
                    //         x: 0.0,
                    //         y: -2.0,
                    //         // y: rng.gen_range((-4.0)..(-1.0)),
                    //     },
                    // });
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

                // coin sound
                // Check if the guy collides with a coin
                if let Some(idx) = self.coins.iter().position(|coin| coin.pos.distance(self.guy.pos) <= COLLISION_DISTANCE) {
                    // Play the coin sound
                    self.audio_manager.play(self.coin_sound.clone()).unwrap();

                    // Remove the collected coin
                    self.coins.swap_remove(idx);
                    
                    // Increase the score
                    self.score += 1;
                }
            }
            GameState::GameOver => {
                // hello
            }
        }
    }
    fn render(&mut self, engine: &mut Engine) {

        match self.game_state {
            GameState::TitleScreen => {
                let (transforms, uvs) = engine.renderer.sprites.get_sprites_mut(1);
                transforms[0] = SPRITE {
                    center: Vec2 {
                        x: W / 2.0,
                        y: H / 2.0,
                    },
                    size: Vec2 { x: W, y: H-(H/4.0) },
                }
                .into();
                uvs[0] = SheetRegion::new(0, 0, 0, 0, 768, 864); // Adjust UV coordinates if needed

                engine.renderer.sprites.resize_sprite_group(
                    &engine.renderer.gpu,
                    1,
                    1,
                );
                engine.renderer.sprites.upload_sprites(
                    &engine.renderer.gpu,
                    1,
                    0..1,
                );
                engine
                    .renderer
                    .sprites
                    .set_camera_all(&engine.renderer.gpu, self.camera);
            }
            GameState::InGame => {
                let score_str = self.score.to_string();
                let text_len = score_str.len();

                let sprite_count =
                    self.walls.len() + self.pavements.len() + self.cars.len() + self.coins.len() + 3;

                engine.renderer.sprites.resize_sprite_group(
                    &engine.renderer.gpu,
                    0,
                    sprite_count + text_len,
                );

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
                if !self.guy.is_jumping {
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
                }

                // set cop
                let cop_idx = guy_idx + 1;
                transforms[cop_idx] = SPRITE {
                    center: self.cop.pos,
                    size: Vec2 { x: 38.4, y: 65.33 },
                }
                .into();

                // animate the guy character
                let ones_place = self.curr_frame % 10;
                match ones_place {
                    0 => {
                        uvs[cop_idx] = SheetRegion::new(0, 177, 498, 0, 14, 18);
                    }
                    1 => {
                        uvs[cop_idx] = SheetRegion::new(0, 191, 480, 0, 14, 18);
                    }
                    2 => {
                        uvs[cop_idx] = SheetRegion::new(0, 191, 498, 0, 14, 18);
                    }
                    _ => {
                        // for other cases, if they come up
                    }
                }

                // set pavement
                let pavement_start = cop_idx + 1;
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
            GameState::GameOver => {
                // the end screen sprite group is now at index 0 after removing the first two groups
                let (transforms, uvs) = engine.renderer.sprites.get_sprites_mut(0);
                transforms[0] = SPRITE {
                    center: Vec2 {
                        x: W / 2.0,
                        y: H / 2.0,
                    },
                    size: Vec2 { x: W, y: H-(H/4.0) },
                }
                .into();
                uvs[0] = SheetRegion::new(0, 0, 0, 1, 768, 864); // Adjust UV coordinates if needed

                let score_str = self.score.to_string();
                let end_text_len = score_str.len();

                self.font_end.draw_text(
                    &mut engine.renderer.sprites,
                    0,
                    1,
                    &score_str,
                    Vec2 {
                        x: (W / 2.0) + 60.0,
                        y: (H / 2.0) - 30.0,
                    }
                    .into(),
                    40.0,
                );

                engine.renderer.sprites.resize_sprite_group(
                    &engine.renderer.gpu,
                    0,
                    1 + end_text_len,
                );
                engine.renderer.sprites.upload_sprites(
                    &engine.renderer.gpu,
                    0,
                    0..1 + end_text_len,
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
