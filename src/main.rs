#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gcd::Gcd;
use rand::seq::SliceRandom;
use sfml::audio::*;
use sfml::graphics::*;
use sfml::system::*;
use sfml::window::*;
use std::io::Write;
use std::thread::current;

mod structs;
use structs::*;

mod config;
use config::*;

pub fn audio(name: &str) -> String {
    format!("{RES_AUDIO_PATH}/{name}.wav")
}

pub fn texture(name: &str) -> String {
    format!("{RES_TEXTURES_PATH}/{name}.png")
}

fn get_level(lines: u32) -> u32 {
    return std::cmp::min(lines / LINES_PER_LEVEL, 15);
}

fn get_update_interval(level: u32) -> u32 {
    // From Tetris Worlds, see https://harddrop.com/wiki/Tetris_Worlds#Gravity
    ((0.8 - (level as i32 - 1) as f32 * 0.007).powi(level as i32 - 1) * 1000.0) as u32
}

fn handle_resize_events(code: Key, current_scale: &mut u32, render_window: &mut RenderWindow, scale_file_path: &std::path::PathBuf) {
    let new_current_scale = match code {
        Key::NUM0 | Key::NUM1 => 1,
        Key::NUM2 => 2,
        Key::NUM4 => 4,
        Key::HYPHEN => match current_scale {
            2 => 1,
            4 => 2,
            _ => 1
        },
        Key::EQUAL => match current_scale {
            1 => 2,
            2 => 4,
            4 => 4,
            _ => 1
        },
        _ => *current_scale
    };
    if *current_scale != new_current_scale {
        render_window.set_size(Vector2u::new(WINDOW_WIDTH, WINDOW_HEIGHT) * new_current_scale);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(&scale_file_path)
            .unwrap();
        file.write_all(new_current_scale.to_string().as_bytes()).unwrap();
        file.flush().unwrap();
        *current_scale = new_current_scale;
    }
}

fn main() {
    let context_settings = ContextSettings::default();
    
    let scale_file_path = home::home_dir().unwrap().join(".septadrop-scale");

    let mut current_scale: u32;

    if scale_file_path.exists() {
        current_scale = std::fs::read_to_string(&scale_file_path)
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap();
    } else {
        let mut scale_file = std::fs::File::create(&scale_file_path).unwrap();
        write!(&mut scale_file, "1").unwrap();
        current_scale = 1;
    }

    let mut window = RenderWindow::new(
        VideoMode::new(WINDOW_WIDTH, WINDOW_HEIGHT, 16),
        "septadrop",
        Style::TITLEBAR | Style::CLOSE,
        &context_settings,
    );
    if current_scale != 1 {
        window.set_size(Vector2u::new(WINDOW_WIDTH, WINDOW_HEIGHT) * current_scale);
    }
    window.set_framerate_limit(FPS);
    window.set_key_repeat_enabled(false);

    let icon = Image::from_file(&texture("icon")).unwrap();
    {
        let Vector2u {
            x: width,
            y: height,
        } = icon.size();
        window.set_icon(width, height, icon.pixel_data());
    }

    let mut thread_rng = rand::thread_rng();
    let block_types = BlockType::init_list();

    let mut random_block = || Block::new(block_types.choose(&mut thread_rng).unwrap());

    let mut block = random_block();
    let mut next_block = random_block();

    let mut grid = Grid::new();
    
    let blocks_texture = Texture::from_file(&texture("blocks")).unwrap();
    let mut sprite = Sprite::with_texture(&blocks_texture);

    let background_texture = Texture::from_file(&texture("background")).unwrap();
    let background = Sprite::with_texture(&background_texture);

    let number_renderer = NumberRenderer::default();

    let mut paused_clear = RectangleShape::new();
    paused_clear.set_fill_color(Color::rgb(81, 62, 69));

    let paused_texture = Texture::from_file(&texture("paused")).unwrap();
    let paused_text = {
        let mut paused_text = Sprite::with_texture(&paused_texture);
        let paused_texture_size = paused_texture.size();
        paused_text.set_position(Vector2f::new(
            PLAYFIELD_X as f32 + (GRID_WIDTH as f32 * TILE_SIZE as f32 / 2.0)
                - paused_texture_size.x as f32 / 2.0,
            PLAYFIELD_Y as f32 + (GRID_HEIGHT as f32 * TILE_SIZE as f32 / 2.0)
                - paused_texture_size.y as f32 / 2.0,
        ));
        paused_text
    };

    let highscore_file_path = home::home_dir().unwrap().join(".septadrop");

    let mut highscore: u32;

    if highscore_file_path.exists() {
        highscore = std::fs::read_to_string(&highscore_file_path)
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap();
        let point_gcd = POINTS_1_LINE
            .gcd(POINTS_2_LINES)
            .gcd(POINTS_3_LINES)
            .gcd(POINTS_4_LINES);
        if highscore % point_gcd != 0 {
            println!("It seems your system is misconfigured. Please see this guide for fixing the issue: https://www.youtube.com/watch?v=dQw4w9WgXcQ");
            return;
        }
    } else {
        let mut highscore_file = std::fs::File::create(&highscore_file_path).unwrap();
        write!(&mut highscore_file, "0").unwrap();
        highscore = 0;
    }

    let mut score: u32 = 0;
    let mut lines: u32 = 0;
    let mut blocks: u32 = 0;
    let mut tiles: u32 = 0;

    let mut update_interval = get_update_interval(0);

    let mut rotate = false;
    let mut move_left = false;
    let mut move_right = false;
    let mut move_left_immediate = false;
    let mut move_right_immediate = false;
    let mut fast_forward = false;
    let mut snap = false;
    let mut paused = false;
    let mut paused_from_lost_focus = false;
    let mut update_clock = Clock::default();
    let mut move_clock = Clock::default();
    let mut pause_clock = Clock::default();
    let mut pause_offset: u32 = 0;
    let mut toggle_pause = false;

    // https://sfxr.me/#57uBnWWZeyDTsBRrJsAp2Vwd76cMVrdeRQ7DirNQW5XekKxcrCUNx47Zggh7Uqw4R5FdeUpyk362uhjWmpNHmqxE7JBp3EkxDxfJ1VjzMRpuSHieW6B5iyVFM
    let rotate_buffer = SoundBuffer::from_file(&audio("rotate")).unwrap();
    let mut rotate_sound = Sound::with_buffer(&rotate_buffer);

    // https://sfxr.me/#57uBnWTMa2LUtaPa3P8xWZekiRxNwCPFWpRoPDVXDJM9KHkiGJcs6J62FRcjMY5oVNdT73MtmUf5rXCPvSZWL7AZuTRWWjKbPKTpZjT85AcZ6htUqTswkjksZ
    let snap_buffer = SoundBuffer::from_file(&audio("snap")).unwrap();
    let mut snap_sound = Sound::with_buffer(&snap_buffer);

    // https://sfxr.me/#57uBnWbareN7MJJsWGD8eFCrqjikS9f8JXg8jvmKzMdVtqmRsb81eToSUpnkqgFhvxD2QoAjpw4SmGZHZjbhEiPQKetRSHCHXYFZzD7Q6RVVS9CRSeRAb6bZp
    let game_over_buffer = SoundBuffer::from_file(&audio("game_over")).unwrap();
    let mut game_over_sound = Sound::with_buffer(&game_over_buffer);

    // https://sfxr.me/#7BMHBGMfGk8EHV8czJkUucUm8EMAnMNxiqYyTfKkMpHFJu44GEdD7xP6E8NM3K7RKRExTpagPBAiWf7BLtC52CEWJVGHh8hwDLygoEG86tcPth2UtmfdrXLoh
    let row_clear_buffer = SoundBuffer::from_file(&audio("row_clear")).unwrap();
    let mut row_clear_sound = Sound::with_buffer(&row_clear_buffer);

    // https://sfxr.me/#57uBnWg8448kTPqWAxeDvZ5CP5JWbrfJGWuRcTjva5uX3vvBnEAZ6SfiH9oLKMXgsusuJwGWx6KPfvLfHtqnhLxr476ptGv4jPbfNhQaFMYeMHFdHk9SotQ4X
    let level_up_buffer = SoundBuffer::from_file(&audio("level_up")).unwrap();
    let mut level_up_sound = Sound::with_buffer(&level_up_buffer);

    // https://sfxr.me/#34T6PkzvrkfdahGDBAh1uYGXTwZ8rG54kxfHpgdVCPxqG7yyK5UuqgiK9Z8Q5177itxbkSNfLSHm4zTkemT4iyxJpW89VJx82feaq8qxZeA5AJR2nWZZR59hq
    let new_highscore_buffer = SoundBuffer::from_file(&audio("new_highscore")).unwrap();
    let mut new_highscore_sound = Sound::with_buffer(&new_highscore_buffer);

    while window.is_open() {
        loop {
            match window.poll_event() {
                Some(event) => match event {
                    Event::Closed => window.close(),
                    Event::LostFocus => {
                        toggle_pause = true;
                        paused_from_lost_focus = true;
                    }
                    Event::KeyPressed {
                        code,
                        alt: _,
                        ctrl: true,
                        shift: _,
                        system: _,
                    } => {
                        handle_resize_events(code, &mut current_scale, &mut window, &scale_file_path);
                    },
                    Event::KeyPressed {
                        code,
                        alt: _,
                        ctrl: _,
                        shift: _,
                        system: _,
                    } => match code {
                        Key::ESCAPE => toggle_pause = true,
                        Key::SPACE => snap = true,
                        Key::UP => rotate = true,
                        Key::DOWN => fast_forward = true,
                        Key::LEFT => {
                            move_left = true;
                            move_left_immediate = true;
                            move_clock.restart();
                        }
                        Key::RIGHT => {
                            move_right = true;
                            move_right_immediate = true;
                            move_clock.restart();
                        }
                        _ => {}
                    },
                    Event::KeyReleased {
                        code,
                        alt: _,
                        ctrl: _,
                        shift: _,
                        system: _,
                    } => match code {
                        Key::DOWN => fast_forward = false,
                        Key::LEFT => move_left = false,
                        Key::RIGHT => move_right = false,
                        _ => {}
                    },
                    _ => {}
                },
                None => break,
            }
        }

        if toggle_pause {
            paused = !paused;
            if paused {
                pause_clock.restart();
                paused_clear.set_position(Vector2f::new(PLAYFIELD_X as f32, PLAYFIELD_Y as f32));
                paused_clear.set_size(Vector2f::new(
                    (GRID_WIDTH * TILE_SIZE) as f32,
                    (GRID_HEIGHT * TILE_SIZE) as f32,
                ));
                window.draw(&paused_clear);
                let size = Vector2f::new(
                    (NEXT_WIDTH * TILE_SIZE) as f32,
                    (NEXT_HEIGHT * TILE_SIZE) as f32,
                );
                paused_clear.set_position(Vector2f::new(NEXT_X as f32, NEXT_Y as f32) - size / 2.0);
                paused_clear.set_size(size);
                window.draw(&paused_clear);
                window.draw(&paused_text);
                window.display();
            } else {
                pause_offset = pause_clock.elapsed_time().as_milliseconds() as u32;
            }
            toggle_pause = false;
        }

        if paused {
            loop {
                match window.wait_event() {
                    Some(event) => match event {
                        Event::Closed => window.close(),
                        Event::KeyPressed {
                            code,
                            alt: _,
                            ctrl: true,
                            shift: _,
                            system: _,
                        } => {
                            handle_resize_events(code, &mut current_scale, &mut window, &scale_file_path);
                        }
                        Event::KeyPressed {
                            code: Key::ESCAPE,
                            alt: _,
                            ctrl: _,
                            shift: _,
                            system: _,
                        } => {
                            paused = false;
                            break;
                        },
                        Event::GainedFocus => if paused_from_lost_focus {
                            paused = false;
                            paused_from_lost_focus = false;
                            break;
                        },
                        _ => {}
                    }
                    _ => {}
                }
            }
        }

        let is_update_frame = update_clock.elapsed_time().as_milliseconds() - pause_offset as i32
            > if fast_forward {
                std::cmp::min(update_interval, MAX_FAST_FORWARD_INTERVAL)
            } else {
                update_interval
            } as i32;
        if is_update_frame {
            update_clock.restart();
        }

        let is_move_frame = move_clock.elapsed_time().as_milliseconds() - pause_offset as i32
            > MOVE_FRAME_INTERVAL as i32;
        if is_move_frame {
            move_clock.restart();
        }

        pause_offset = 0;

        // Rotation
        if rotate {
            block.rotation_state += 1;
            // Check to see if new rotation state is overlapping any tiles
            let mut offset_required: i32 = 0;
            for tile in block.get_tiles().iter() {
                if grid.get(tile.x, tile.y).is_some() {
                    // Can't wall kick off of blocks
                    block.rotation_state -= 1;
                    break;
                }
                if tile.x <= 0 {
                    let potential_offset = -tile.x;
                    if potential_offset > offset_required.abs() {
                        offset_required = potential_offset;
                    }
                } else if tile.x >= GRID_WIDTH as i32 {
                    let potential_offset = GRID_WIDTH as i32 - tile.x - 1;
                    if -potential_offset > offset_required.abs() {
                        offset_required = potential_offset;
                    }
                }
            }
            block.position.x += offset_required;
            rotate = false;
            rotate_sound.play();
        }

        // Horizontal movement
        {
            let mut movement = 0;
            if move_left_immediate || (is_move_frame && move_left) {
                movement -= 1;
                move_left_immediate = false;
            }
            if move_right_immediate || (is_move_frame && move_right) {
                movement += 1;
                move_right_immediate = false;
            }
            if movement != 0 {
                for (i, tile) in block.get_tiles().iter().enumerate().rev() {
                    if grid.filled(tile.x + movement, tile.y)
                    {
                        break;
                    }
                    if i == 0 {
                        /*
                            We're going through all the blocks backwards,
                            so the first element is last.
                            (Only for .enumerate().rev(), not .rev().enumerate(),
                            since .enumerate() is what adds indexes.)
                            If we managed to get through all of the tiles without breaking,
                            (haven't found anything that obstructs any tile),
                            we can finally add the movement.
                        */
                        block.position.x += movement;
                    }
                }
            }
        }

        // Snapping
        let snap_offset = {
            let mut snap_offset = 0;
            'outer: loop {
                for tile in block.get_tiles().iter() {
                    let y = tile.y + snap_offset;
                    if grid.filled(tile.x, y) {
                        snap_offset -= 1;
                        break 'outer;
                    }
                }
                snap_offset += 1;
            }
            snap_offset
        };
        let mut landed = snap;
        if snap {
            block.position.y += snap_offset;
            snap = false;
            snap_sound.play();
        } else if is_update_frame {
            // Land checking
            for tile in block.get_tiles().iter() {
                if tile.y == GRID_HEIGHT as i32 - 1
                    || grid.filled(tile.x, tile.y + 1)
                {
                    landed = true;
                    break;
                }
            }
        }
        let landed = landed; // remove mutability

        // Clear window
        // Normally, one would run window.clear(),
        // but the background image covers the entire window.
        window.draw(&background);

        // Draw block
        if !landed {
            for tile in block.get_tiles().iter() {
                let snap_y = tile.y + snap_offset;
                sprite.set_texture_rect(&block.block_type.tile_type.texture_rect);
                sprite.set_position(Vector2f::new(
                    (PLAYFIELD_X as i32 + tile.x * TILE_SIZE as i32) as f32,
                    (PLAYFIELD_Y as i32 + tile.y * TILE_SIZE as i32) as f32,
                ));
                window.draw(&sprite);
                sprite.set_texture_rect(&block.block_type.tile_type.ghost_texture_rect);
                sprite.set_position(Vector2f::new(
                    (PLAYFIELD_X as i32 + tile.x * TILE_SIZE as i32) as f32,
                    (PLAYFIELD_Y as i32 + snap_y * TILE_SIZE as i32) as f32,
                ));
                window.draw(&sprite);
            }
        }

        // Draw next block
        {
            let next_block_tiles = next_block.get_tiles();
            // This is assuming the next block spawns unrotated.
            // Refactoring is needed if random rotations are added
            let x_offset = next_block.block_type.width * TILE_SIZE / 2;
            let y_offset = (next_block.block_type.height + next_block.block_type.starting_line * 2)
                * TILE_SIZE
                / 2;
            for tile in next_block_tiles.iter() {
                sprite.set_texture_rect(&next_block.block_type.tile_type.texture_rect);
                sprite.set_position(Vector2f::new(
                    (NEXT_X + (tile.x - next_block.position.x) as u32 * TILE_SIZE - x_offset)
                        as f32,
                    (NEXT_Y + (tile.y - next_block.position.y) as u32 * TILE_SIZE - y_offset)
                        as f32,
                ));
                window.draw(&sprite);
            }
        }

        // Landing (transferring block to grid and reinitializing)
        if landed {
            tiles += block.get_tiles().len() as u32;
            blocks += 1;
            for tile in block.get_tiles().iter() {
                grid.set(tile.x, tile.y, Some(&block.block_type.tile_type));
            }
            let mut cleared_lines = 0;
            for y in 0..GRID_HEIGHT {
                let mut completed = true;
                for x in 0..GRID_WIDTH {
                    if !grid.filled(x as i32, y as i32) {
                        completed = false;
                        break;
                    }
                }
                if !completed {
                    continue;
                }
                for z in (0..y).rev() {
                    let z = z as i32;
                    for x in 0..GRID_WIDTH {
                        let x = x as i32;
                        grid.set(x, z + 1, grid.get(x, z));
                    }
                }
                cleared_lines += 1;
            }
            let mut scored = match cleared_lines {
                0 => 0,
                1 => POINTS_1_LINE,
                2 => POINTS_2_LINES,
                3 => POINTS_3_LINES,
                _ => POINTS_4_LINES,
            };
            if scored > 0 {
                let level = get_level(lines);
                scored *= level + 1;
                if score + scored > highscore && score < highscore {
                    new_highscore_sound.play();
                }
                score += scored;
                lines += cleared_lines;
                let new_level = get_level(lines);
                if level != new_level {
                    level_up_sound.play();
                }
                if score > highscore {
                    highscore = score;
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .open(&highscore_file_path)
                        .unwrap();
                    file.write_all(highscore.to_string().as_bytes()).unwrap();
                    file.flush().unwrap();
                }
                update_interval = get_update_interval(new_level);
                row_clear_sound.play();
            }
            for tile in next_block.get_tiles().iter() {
                if grid.filled(tile.x, tile.y) {
                    score = 0;
                    lines = 0;
                    blocks = 0;
                    tiles = 0;
                    grid.clear();
                    update_interval = get_update_interval(0);
                    game_over_sound.play();
                    next_block = random_block();
                    break;
                }
            }
            block = next_block;
            next_block = random_block();
        } else if is_update_frame {
            block.position.y += 1;
        }

        // Drawing grid
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let tile_type = grid.get(x as i32, y as i32);
                if tile_type.is_none() {
                    continue;
                }
                let tile_type = tile_type.unwrap();
                sprite.set_texture_rect(&tile_type.texture_rect);
                sprite.set_position(Vector2f::new(
                    (PLAYFIELD_X + x * TILE_SIZE) as f32,
                    (PLAYFIELD_Y + y * TILE_SIZE) as f32,
                ));
                window.draw(&sprite);
            }
        }

        number_renderer.render(&mut window, score, 477, 162);
        number_renderer.render(&mut window, highscore, 477, 202);
        number_renderer.render(&mut window, lines, 477, 242);
        number_renderer.render(&mut window, get_level(lines), 477, 282);
        number_renderer.render(&mut window, blocks, 477, 322);
        number_renderer.render(&mut window, tiles, 477, 362);

        window.display();
    }
}
