use engine;
use engine::wgpu;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::{distributions::Uniform, Rng};
use std::fmt;
// sound/audio --> use Kira
use kira::{
    manager::{
        AudioManager, AudioManagerSettings,
        backend::DefaultBackend,
    },
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
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
struct Bus {
    pos: Vec2,
}

struct Sprite {
    pos: Vec2,
    vel: Vec2,
}

#[derive(PartialEq, Debug, Clone)]
enum Job {
    Doctor,
    Firefighter,
    Regular,
    Cop
}

enum CatDog {
    Cat,
    Dog,
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Job::Firefighter => write!(f, "Firefighter"),
            Job::Doctor => write!(f, "Doctor"),
            Job::Regular => write!(f, "Regular"),
            Job::Cop => write!(f, "Cop"),
        }
    }
}

struct Person {
    pos: Vec2,
    vel: Vec2,
    job: Job
}

struct Building {
    pos: Vec2,
    vel: Vec2,
    job: Job
}

struct Animal {
    pos: Vec2,
    vel: Vec2,
    animal_type: CatDog,
}


enum GameState {
    TitleScreen,
    InGame,
    GameOver, //eventually add the GameOver
}

struct Game {
    camera: engine::Camera,
    walls: Vec<SPRITE>,
    bus: Bus,
    animals: Vec<Animal>,
    people: Vec<Person>,
    animal_timer: u32,
    people_timer: u32,
    animal_speed_multiplier: f32,
    building_speed_multiplier: f32,
    buildings: Vec<Building>,
    building_timer: u32,
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
    on_bus: Vec<Person>,
    // sound
    audio_manager: AudioManager<DefaultBackend>,
    drop_sound: StaticSoundData,
    cat_sound: StaticSoundData,
}

impl engine::Game for Game {
    // create new game instance
    fn new(engine: &mut Engine) -> Self {
        let camera = Camera {
            screen_pos: [0.0, 0.0],
            screen_size: [W, H],
        };
        #[cfg(not(target_arch = "wasm32"))]
        let start_img = image::open("../content/title_screen_game2.png").unwrap().into_rgba8();
        let start_tex = engine.renderer.gpu.create_texture(&start_img, wgpu::TextureFormat::Rgba8UnormSrgb, start_img.dimensions(), Some("start-sprite.png"),);
        let end_img = image::open("../content/end_screen_game2.png").unwrap().into_rgba8();
        let end_tex = engine.renderer.gpu.create_texture(&end_img, wgpu::TextureFormat::Rgba8UnormSrgb, end_img.dimensions(), Some("end-sprite.png"),);
        // #[cfg(target_arch = "wasm32")]

        let sprite_img = image::open("../content/run-spritesheet2.png")
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
            vec![Transform::zeroed(); SPRITE_MAX], //bg, three walls, bus, a few animals
            vec![SheetRegion::zeroed(); SPRITE_MAX],
            camera,
        );
        
        // start sprite group
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &start_tex,
            vec![Transform::zeroed(); 1],
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

        let bus = Bus {
            pos: Vec2 {
                x: 378.66,
                y: GUY_Y_POS,
            },
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
            SheetRegion::new(0, 0, 866, 0, 80, 8),
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
        let animal_speed_multiplier = 1.0;
        let building_speed_multiplier = 1.0;

        let on_bus: Vec<Person> = Vec::with_capacity(5);

        // drop sound
        // Create an audio manager
        let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
        // Load the drop sound
        let drop_sound = StaticSoundData::from_file("../content/hotel-bell-ding.mp3", StaticSoundSettings::default()).unwrap();
        // Load the cat sound
        let cat_sound = StaticSoundData::from_file("../content/angry_cat.mp3", StaticSoundSettings::default()).unwrap();

        Game {
            camera,
            bus,
            walls: vec![left_wall, right_wall, floor],
            animals: Vec::with_capacity(8),
            people: Vec::with_capacity(30),
            animal_timer: 0,
            people_timer: 0,
            buildings: Vec::with_capacity(33),
            building_timer: 0,
            animal_speed_multiplier,
            building_speed_multiplier,
            pavements,
            pavement_timer: 0,
            score: 0,
            font,
            font_end,
            curr_frame: 0,
            frame_counter: 0,
            frame_direction: 1,
            game_over: false,
            // sound
            audio_manager,
            drop_sound,
            cat_sound,
            game_state: GameState::TitleScreen,
            on_bus,
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
                // set the speed of animation for bus. Adjust number after modulo.
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
                self.bus.pos.x += dir * GUY_SPEED;

                // for continuous up or down movement
                let dir = engine.input.key_axis(engine::Key::Down, engine::Key::Up);
                self.bus.pos.y += dir * GUY_SPEED;
                self.bus.pos.y += dir * GUY_SPEED;

                let mut contacts = Vec::with_capacity(self.walls.len());

                for _iter in 0..COLLISION_STEPS {
                    let bus_sprite = SPRITE {
                        center: self.bus.pos,
                        size: Vec2 { x: 38.4, y: 65.33 },
                    };
                    contacts.clear();

                    contacts.extend(
                        self.walls
                            .iter()
                            .enumerate()
                            .filter_map(|(ri, w)| w.displacement(bus_sprite).map(|d| (ri, d))),
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
                        // TODO: for multiple buss should access self.buss[bus_idx].
                        let bus_aabb = SPRITE {
                            center: self.bus.pos,
                            size: Vec2 { x: 38.4, y: 65.33 },
                        };
                        let wall = self.walls[*wall_idx];
                        let mut disp = wall.displacement(bus_sprite).unwrap_or(Vec2::ZERO);
                        // We got to a basically zero collision amount
                        if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                            break;
                        }
                        // bus is left of wall, push left
                        if self.bus.pos.x < wall.center.x {
                            disp.x *= -1.0;
                        }
                        // bus is below wall, push down
                        if self.bus.pos.y < wall.center.y {
                            disp.y *= -1.0;
                        }
                        if disp.x.abs() <= disp.y.abs() {
                            self.bus.pos.x += disp.x;
                            // so far it seems resolved; for multiple buss this should probably set a flag on the bus
                        } else if disp.y.abs() <= disp.x.abs() {
                            self.bus.pos.y += disp.y;
                            // so far it seems resolved; for multiple buss this should probably set a flag on the bus
                        }
                    }
                }
                let mut rng = rand::thread_rng();

                // create columns for animals
                let uniform = Uniform::new(0, possible_values.len());
                let random_index = rng.sample(uniform);
                let random_value = possible_values[random_index];

                // create columns for buildings
                let uniform_buildings = Uniform::new(0, side_values.len());
                let random_index_buildings = rng.sample(uniform_buildings);
                let random_value_buildings = side_values[random_index_buildings];

                // spawn new animals
                if self.animal_timer > 0 {
                    self.animal_timer -= 1;
                } else if self.animals.len() < 32 {
                    let mut valid_position = false;
                    let mut new_animal_pos = Vec2::default();
                    while !valid_position {
                        let uniform = Uniform::new(0, possible_values.len());
                        let random_index = rng.sample(uniform);
                        new_animal_pos = Vec2 {
                            x: possible_values[random_index],
                            y: H + 8.0,
                        };

                        // Check if the new position overlaps with existing animals
                        valid_position = !self
                            .animals
                            .iter()
                            .any(|animal| new_animal_pos.distance(animal.pos) <= COLLISION_DISTANCE)
                            && !self
                                .buildings
                                .iter()
                                .any(|building| new_animal_pos.distance(building.pos) <= COLLISION_DISTANCE);
                    }
                    let generated_animal = match rand::thread_rng().gen_range(0..1) {
                        0 => CatDog::Cat,
                        1 => CatDog::Dog,
                        _ => unreachable!(), // Should never happen, just to handle all cases
                    };
                    self.animals.push(Animal {
                        pos: new_animal_pos,
                        vel: Vec2 { x: 0.0, y: -2.0 },
                        animal_type: generated_animal,
                    });
                    self.animal_timer = rng.gen_range(30..90);
                }
                // update animal velocities every frame
                for animal in self.animals.iter_mut() {
                    animal.pos += animal.vel;
                }
                
                // between frames, maintain all the animals on the screen that are above position -8.0
                self.animals.retain(|animal| animal.pos.y > -8.0);

                // spawn new people
                if self.people_timer > 0 {
                    self.people_timer -= 1;
                } else if self.people.len() < 5 {
                    let mut valid_position = false;
                    let mut new_person_pos = Vec2::default();
                    while !valid_position {
                        let uniform = Uniform::new(0, possible_values.len());
                        let random_index = rng.sample(uniform);
                        new_person_pos = Vec2 {
                            x: possible_values[random_index],
                            y: H + 8.0,
                        };

                        // Check if the new position overlaps with existing animals
                        valid_position = !self
                            .animals
                            .iter()
                            .any(|animal| new_person_pos.distance(animal.pos) <= COLLISION_DISTANCE)
                            && !self
                                .buildings
                                .iter()
                                .any(|building| new_person_pos.distance(building.pos) <= COLLISION_DISTANCE);
                    }
                    // TODO: generate a random job
                    let generated_job = match rand::thread_rng().gen_range(0..4) {
                        0 => Job::Doctor,
                        1 => Job::Firefighter,
                        2 => Job::Regular,
                        3 => Job::Cop,
                        _ => unreachable!(), // Should never happen, just to handle all cases
                    };

                    self.people.push(Person {
                        pos: new_person_pos,
                        vel: Vec2 { x: 0.0, y: -2.0 },
                        job: generated_job,
                    });
                    self.people_timer = rng.gen_range(30..180);
                }
                // update people velocities every frame
                for person in self.people.iter_mut() {
                    person.pos += person.vel;
                }

                // Check collision with animals
                if self
                    .animals
                    .iter()
                    .any(|animal| animal.pos.distance(self.bus.pos) <= COLLISION_DISTANCE)
                {
                    // play cat sound
                    // self.audio_manager.play(self.cat_sound.clone()).unwrap(); 
                    println!("Game Over! Your final score: {}", self.score);
                    engine.renderer.sprites.remove_sprite_group(0);
                    self.game_state = GameState::GameOver;
                }

                // if any person is within the catch distance of the bus, mark a collision
                if self.on_bus.len() < 5 {
                    if let Some(idx) = self
                    .people
                    .iter()
                    .position(|person| person.pos.distance(self.bus.pos) <= COLLISION_DISTANCE)
                    {
                        self.on_bus.push(Person {
                            pos: Vec2 {x: 0.0, y: 0.0},
                            vel: Vec2 {x: 0.0, y: 0.0},
                            job: self.people[idx].job.clone(),
                        });
                        println!("On Bus: {}", self.on_bus.len());
                        self.people.swap_remove(idx);
                    }
                }

                self.people.retain(|person| person.pos.y > -8.0);
                // between frames, maintain all the animals on the screen that are above position -8.0
                self.animals.retain(|animal| animal.pos.y > -8.0);


                // if a building is within the catch distance, 
                if let Some(idx) = self
                    .buildings
                    .iter()
                    .position(|building: &Building| building.pos.distance(self.bus.pos) <= DROP_OFF_DIST)
                {
                    let curr_building = &self.buildings[idx].job;

                    // // Retain only the buildings that do not match the job of a person on the bus
                    // self.buildings.retain(|building| !self.on_bus.iter().any(|person| person.job == building.job));

                    // check if the job of the building matches the job of a person on the bus
                    // remove person from the bus if dropped off
                    if self.on_bus.iter().any(|person| person.job == self.buildings[idx].job) {
                        if let Some(person_idx) = self
                            .on_bus
                            .iter()
                            .position(|person| person.job == self.buildings[idx].job)
                        {
                            // play drop sound
                            self.audio_manager.play(self.drop_sound.clone()).unwrap(); 
                            println!("Removed a {} from the bus!", self.buildings[idx].job);
                            self.on_bus.swap_remove(person_idx);
                            println!("number of people on bus: {}", self.on_bus.len());
                            self.score += 1;

                        }
                        self.buildings.swap_remove(idx);
                    }
                    // println!("building job: {}", self.buildings[idx].job);
                }

                self.buildings.retain(|building| building.pos.y > -8.0);

                // Spawn new buildings
                if self.building_timer > 0 {
                    self.building_timer -= 1;
                } else if self.buildings.len() < 32 {
                    let mut valid_position = false;
                    let mut new_building_pos = Vec2::default();
                    while !valid_position {
                        let uniform_building = Uniform::new(0, side_values.len());
                        let random_index_building = rng.sample(uniform_building);
                        new_building_pos = Vec2 {
                            x: side_values[random_index_building],
                            y: H + 8.0,
                        };

                        // Check if the new position overlaps with existing animals or buildings
                        valid_position = !self
                            .animals
                            .iter()
                            .any(|animal| new_building_pos.distance(animal.pos) <= COLLISION_DISTANCE)
                            && !self
                                .buildings
                                .iter()
                                .any(|building| new_building_pos.distance(building.pos) <= COLLISION_DISTANCE);
                    }
                    let generated_job = match rand::thread_rng().gen_range(0..4) {
                        0 => Job::Doctor,
                        1 => Job::Firefighter,
                        2 => Job::Regular,
                        3 => Job::Cop,
                        _ => unreachable!(), // Should never happen, just to handle all cases
                    };
                    self.buildings.push(Building {
                        pos: new_building_pos,
                        vel: Vec2 { x: 0.0, y: -2.0 },
                        job: generated_job,
                    });
                    self.building_timer = rng.gen_range(30..90);
                }
                // Update buildings
                for building in self.buildings.iter_mut() {
                    building.pos += building.vel;
                }
                self.buildings.retain(|building| building.pos.y > -8.0);

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
                self.animal_speed_multiplier += 0.001 * acc;
                self.building_speed_multiplier += 0.001 * acc;

                // Update animals with increased speed
                for animal in self.animals.iter_mut() {
                    animal.pos += animal.vel * self.animal_speed_multiplier;
                }

                // Update buildings with increased speed
                for building in self.buildings.iter_mut() {
                    building.pos += building.vel * self.building_speed_multiplier;
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
                    self.walls.len() + self.pavements.len() + self.animals.len() + self.people.len() + self.buildings.len() + self.on_bus.len() + 4;

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
                let frame_start: usize = WALL_START + self.walls.len();
                for (wall, (transform, uv)) in self.walls.iter().zip(
                    transforms[WALL_START..frame_start]
                        .iter_mut()
                        .zip(uvs[WALL_START..frame_start].iter_mut()),
                ) {
                    *transform = (*wall).into();
                    *uv = SheetRegion::new(0, 0, 480, 12, 8, 8);
                }

                // set sprite counter frame
                transforms[frame_start] = SPRITE {
                    center: Vec2 {
                        x: W - 40.0,
                        y: H - 300.0,
                    },
                    size: Vec2 { x: 60.0, y: 500.0 },
                }
                .into();
                uvs[frame_start] = SheetRegion::new(0, 312, 501, 1, 40, 309);


                // set bus
                let bus_idx = frame_start + 1;
                transforms[bus_idx] = SPRITE {
                    center: self.bus.pos,
                    size: Vec2 { x: 60.0, y: 130.0 },
                }
                .into();
                uvs[bus_idx] = SheetRegion::new(0, 8, 532, 1, 25, 42);

                

        

                // set pavement
                let pavement_start = bus_idx + 1;
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

                // set animal
                let animal_start = pavement_start + self.pavements.len();

                for (animal, (transform, uv)) in self.animals.iter().zip(
                    transforms[animal_start..]
                        .iter_mut()
                        .zip(uvs[animal_start..].iter_mut()),
                ) {
                    *transform = SPRITE {
                        center: animal.pos,
                        size: Vec2 { x: 38.4, y: 65.33 },
                    }
                    .into();
                    match animal.animal_type {
                        CatDog::Cat => {
                            *uv = SheetRegion::new(0, 113, 564, 3, 27, 29);
                        }
                        CatDog::Dog => {
                            *uv = SheetRegion::new(0, 146, 565, 3, 25, 27);
                        }
                        _ => {
                            // other cases
                        }
                    }
                }

                // set people
                let people_start = animal_start + self.animals.len();

                for (person, (transform, uv)) in self.people.iter().zip(
                    transforms[people_start..]
                        .iter_mut()
                        .zip(uvs[people_start..].iter_mut()),
                ) {
                    *transform = SPRITE {
                        center: person.pos,
                        size: Vec2 { x: 38.4, y: 65.33 },
                    }
                    .into();
                    let ones_place = self.curr_frame % 10;
                    match person.job {
                        Job::Firefighter => {
                            match ones_place {
                                0 => {
                                    *uv = SheetRegion::new(0, 134, 480, 0, 16, 19);
                                }
                                1 => {
                                    *uv = SheetRegion::new(0, 134, 499, 0, 16, 19);
                                }
                                2 => {
                                    *uv = SheetRegion::new(0, 150, 498, 0, 16, 19);
                                }
                                _ => {
                                    // for other cases, if they come up
                                }
                            }
                        }
                        Job::Doctor => {
                            match ones_place {
                                0 => {
                                    *uv = SheetRegion::new(0, 212, 480, 0, 14, 17);
                                }
                                1 => {
                                    *uv = SheetRegion::new(0, 212, 497, 0, 14, 17);
                                }
                                2 => {
                                    *uv = SheetRegion::new(0, 226, 497, 0, 14, 17);
                                }
                                _ => {
                                    // for other cases, if they come up
                                }
                            }
                        }
                        Job::Cop => {
                            match ones_place {
                                0 => {
                                    *uv = SheetRegion::new(0, 177, 480, 0, 14, 18);
                                }
                                1 => {
                                    *uv = SheetRegion::new(0, 191, 498, 0, 14, 18);
                                }
                                2 => {
                                    *uv = SheetRegion::new(0, 177, 498, 0, 14, 18);
                                }
                                _ => {
                                    // for other cases, if they come up
                                }
                            }   
                        }
                        Job::Regular => {
                            match ones_place {
                                0 => {
                                    *uv = SheetRegion::new(0, 100, 480, 1, 14, 18);
                                }
                                1 => {
                                    *uv = SheetRegion::new(0, 100, 498, 1, 14, 18);
                                }
                                2 => {
                                    *uv = SheetRegion::new(0, 114, 498, 1, 14, 18);
                                }
                                _ => {
                                    // for other cases, if they come up
                                }
                            }
                        }
                    }
                }

                // set building
                let building_start = people_start + self.people.len();
                for (building, (transform, uv)) in self.buildings.iter().zip(
                    transforms[building_start..]
                        .iter_mut()
                        .zip(uvs[building_start..].iter_mut()),
                ) {
                    *transform = SPRITE {
                        center: building.pos,
                        size: Vec2 { x: 60.0, y: 80.0 },
                    }
                    .into();
                    match building.job {
                        Job::Firefighter => {
                            *uv = SheetRegion::new(0, 132, 518, 1, 44, 42);
                        }
                        Job::Doctor => {
                            *uv = SheetRegion::new(0, 212, 528, 1, 30, 30);
                        }
                        Job::Cop => {
                            *uv = SheetRegion::new(0, 176, 530, 1, 30, 28);
                        }
                        Job::Regular => {
                            *uv = SheetRegion::new(0, 97, 528, 1, 32, 33);
                        }
                        _ => {
                            // for other cases, if they come up
                        }
                    }
                }

                let on_bus_start = building_start + self.buildings.len();
                let bus_seats = vec![ H-120.0, H-210.0, H-300.0, H-390.0, H-480.0 ];
                for (index, (person_on_bus, (transform, uv))) in self.on_bus.iter().zip(
                    transforms[on_bus_start..]
                        .iter_mut()
                        .zip(uvs[on_bus_start..].iter_mut()),
                ).enumerate() {
                    *transform = SPRITE {
                        center: Vec2{ x: W-40.0, y: bus_seats[index]},
                        size: Vec2 { x: 38.4, y: 65.33 },
                    }
                    .into();
                    match person_on_bus.job {
                        Job::Firefighter => {
                            *uv = SheetRegion::new(0, 134, 480, 0, 16, 19);
                        }
                        Job::Doctor => {
                            *uv = SheetRegion::new(0, 212, 480, 0, 14, 17);
                        }
                        Job::Cop => {
                            *uv = SheetRegion::new(0, 177, 480, 0, 14, 18);
                        }
                        Job::Regular => {
                            *uv = SheetRegion::new(0, 100, 480, 0, 14, 18);
                        }
                        _ => {
                            // for other cases, if they come up
                        }
                    }
                }


                let sprite_count = on_bus_start + self.on_bus.len();

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
                let text_start = on_bus_start + self.on_bus.len();
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
                // self.audio_manager.play(self.cat_sound.clone()).unwrap(); 

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
                        y: (H / 2.0) + 50.0,
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
