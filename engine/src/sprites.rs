use crate::geom;

use frenderer::{SheetRegion, BitFont, SpriteRenderer, WGPU, Camera2D};
use glam::*;
use std::ops::RangeInclusive;

pub struct Sprite {
    pub pos: Vec2,
    pub vel: Vec2,
}

pub fn generate_scrolling_side(pavements: &mut Vec<Sprite>, speed: f32, screen_width: f32, screen_height: f32) {
    // Spawn new pavements
    if pavements.len() < 33 {
        let newest_right_idx = pavements.len() - 2;
        let newest_left_idx = pavements.len() - 1;
        // create a right pavement
        pavements.push(Sprite {
            pos: Vec2 {
                x: screen_width - 2.0,
                // add the next sprite one window height's length above the center of the most recently created sprite
                y: pavements[newest_right_idx].pos[1] + screen_height,
            },
            vel: Vec2 {
                x: 0.0,
                y: speed,
            },
        });
        // create a left pavement
        pavements.push(Sprite {
            pos: Vec2 {
                x: 2.0,
                y: pavements[newest_left_idx].pos[1] + screen_height,
            },
            vel: Vec2 { x: 0.0, y: -1.0 },
        });
    }
    // Update pavements
    for pavement in pavements.iter_mut() {
        pavement.pos += pavement.vel;
    }
    pavements.retain(|pavement| pavement.pos.y > -screen_height / 2.0);
}

pub fn animate_char(ones_place: &usize, uv: &mut SheetRegion, sheet: u16, coords: [u16; 6], depth: u16, w: u16, h: u16) {
    match ones_place {
        0 => {
            *uv = SheetRegion::new(sheet, coords[0], coords[1], depth, w, h);
        }
        1 => {
            *uv = SheetRegion::new(sheet, coords[2], coords[3], depth, w, h);
        }
        2 => {
            *uv = SheetRegion::new(sheet, coords[4], coords[5], depth, w, h);
        }
        _ => {
            // for other cases, if they come up
        }
    }
}

pub fn handle_collisions(character: &mut Sprite, walls: &Vec<geom::SPRITE>, collision_steps: usize) {
    let mut contacts: Vec<(usize, Vec2)> = Vec::with_capacity(walls.len());
    
    for _iter in 0..collision_steps {
        let main_sprite = geom::SPRITE {
            center: character.pos,
            size: Vec2 { x: 38.4, y: 115.0 },
        };
        contacts.clear();

        contacts.extend(
            walls
                .iter()
                .enumerate()
                .filter_map(|(ri, w)| w.displacement(main_sprite).map(|d| (ri, d))),
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

            let wall = walls[*wall_idx];
            let mut disp = wall.displacement(main_sprite).unwrap_or(Vec2::ZERO);
            // We got to a basically zero collision amount
            if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                break;
            }
            // character is left of wall, push left
            if character.pos.x < wall.center.x {
                disp.x *= -1.0;
            }
            // character is below wall, push down
            if character.pos.y < wall.center.y {
                disp.y *= -1.0;
            }
            if disp.x.abs() <= disp.y.abs() {
                character.pos.x += disp.x;
                // so far it seems resolved; for multiple characters this should probably set a flag on the character
            } else if disp.y.abs() <= disp.x.abs() {
                character.pos.y += disp.y;
                // so far it seems resolved; for multiple characters this should probably set a flag on the character
            }
        }
    }
}

pub fn render_start_sprite(sprites: &mut SpriteRenderer, gpu: &mut WGPU, camera: Camera2D, width: f32, height: f32) {
    let (transforms, uvs) = sprites.get_sprites_mut(1);
                transforms[0] = geom::SPRITE {
                    center: Vec2 {
                        x: width / 2.0,
                        y: height / 2.0,
                    },
                    size: Vec2 { x: width, y: height-(height/4.0) },
                }
                .into();
                uvs[0] = SheetRegion::new(0, 0, 0, 0, 768, 864); // Adjust UV coordinates if needed

                sprites.resize_sprite_group(
                    gpu,
                    1,
                    1,
                );
                sprites.upload_sprites(
                    gpu,
                    1,
                    0..1,
                );
                sprites
                    .set_camera_all(gpu, camera);
}

pub fn render_end_sprite(font_end: &BitFont<RangeInclusive<char>>, sprites: &mut SpriteRenderer, gpu: &mut WGPU, score: u32, camera: Camera2D, font_y_offset: f32, width: f32, height: f32) {
    let (transforms, uvs) = sprites.get_sprites_mut(0);
    transforms[0] = geom::SPRITE {
        center: Vec2 {
            x: width / 2.0,
            y: height / 2.0,
        },
        size: Vec2 { x: width, y: height-(height/4.0) },
    }
    .into();
    uvs[0] = SheetRegion::new(0, 0, 0, 1, 768, 864); // Adjust UV coordinates if needed

    let score_str = score.to_string();
    let end_text_len = score_str.len();

    font_end.draw_text(
        sprites,
        0,
        1,
        &score_str,
        Vec2 {
            x: (width / 2.0) + 60.0,
            y: (height / 2.0) + font_y_offset as f32,
        }
        .into(),
        40.0,
    );

    sprites.resize_sprite_group(
        &gpu,
        0,
        1 + end_text_len,
    );
    sprites.upload_sprites(
        &gpu,
        0,
        0..1 + end_text_len,
    );
    sprites
        .set_camera_all(&gpu, camera);
}

pub fn render_game_sprites(font: &BitFont<RangeInclusive<char>>, camera: Camera2D, sprites: &mut SpriteRenderer, sprite_count: usize, score_str: String, gpu: &mut WGPU, height: f32) {
    let text_len = score_str.len();
    font.draw_text(
        sprites,
        0,
        sprite_count,
        &score_str,
        Vec2 {
            x: 16.0,
            y: height - 16.0,
        }
        .into(),
        16.0,
    );
    
    sprites.resize_sprite_group(
        &gpu,
        0,
        sprite_count + text_len,
    );
    sprites.upload_sprites(
        &gpu,
        0,
        0..sprite_count + text_len,
    );
    sprites
        .set_camera_all(&gpu, camera);
}
