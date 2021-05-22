use std::collections::VecDeque;

use super::*;

const ACCEL: f32 = 10.0;
const ROTATION_SPEED: f32 = 10.0;
const PLAYER_SHOOT_INTERVAL: f32 = 0.3;
const ENEMY_SHOOT_INTERVAL: f32 = 1.47562975;
const PLAYER_SPEED: f32 = 8.0;
const SHOT_SPEED: f32 = 15.0;
const ENEMY_SHOT_SPEED: f32 = 10.0;
const ENEMY_SPEED: f32 = 3.0;
const WORLD_SIZE: f32 = 10.0;
const ENEMY_SPAWN_BASE_INTERVAL: f32 = 10.0002734;
const FAST_FORWARD_MULTIPLIER: f32 = 2.0;
const TIME_ACCEL: f32 = 5.0;

#[derive(Clone)]
struct Entity {
    position: Vec2<f32>,
    velocity: Vec2<f32>,
    target_velocity: Vec2<f32>,
    rotation: f32,
    aim: Option<Vec2<f32>>,
    next_shoot: f32,
    target_position: Vec2<f32>,
}

impl Entity {
    fn new(position: Vec2<f32>, velocity: Vec2<f32>) -> Self {
        Self {
            position,
            velocity,
            target_velocity: velocity,
            rotation: if velocity.len() < 0.1 {
                0.0
            } else {
                velocity.arg()
            },
            aim: None,
            next_shoot: 0.0,
            target_position: position,
        }
    }
    fn update(&mut self, delta_time: f32) {
        self.velocity += (self.target_velocity - self.velocity).clamp(ACCEL * delta_time);
        let mut target_rotation = self.rotation;
        if self.velocity.len() > 0.1 {
            target_rotation = self.velocity.arg();
        }
        if let Some(aim) = self.aim {
            let delta = aim - self.position;
            if delta.len() > 0.1 {
                target_rotation = delta.arg();
            }
        }
        let mut delta_rotation = target_rotation - self.rotation;
        while delta_rotation > f32::PI {
            delta_rotation -= 2.0 * f32::PI;
        }
        while delta_rotation < -f32::PI {
            delta_rotation += 2.0 * f32::PI;
        }
        let max_delta = ROTATION_SPEED * delta_time;
        self.rotation += delta_rotation.clamp(-max_delta, max_delta);
        self.position += self.velocity * delta_time;
        self.next_shoot -= delta_time;
    }
}

#[derive(Clone)]
struct Model {
    current_time: f32,
    player: Option<Entity>,
    enemies: Vec<Entity>,
    player_shots: Vec<Entity>,
    enemy_shots: Vec<Entity>,
    next_enemy: f32,
}

impl Model {
    fn new() -> Self {
        Self {
            current_time: 0.0,
            player: Some(Entity::new(vec2(0.0, 0.0), vec2(0.0, 0.0))),
            enemies: Vec::new(),
            player_shots: Vec::new(),
            next_enemy: 5.0,
            enemy_shots: Vec::new(),
        }
    }
    fn update(&mut self, delta_time: f32, assets: &Assets) {
        self.next_enemy -= delta_time;
        while self.next_enemy < 0.0 {
            self.next_enemy += ENEMY_SPAWN_BASE_INTERVAL / self.current_time;
            assets.pop.play();
            self.enemies.push(Entity::new(
                vec2(
                    0.0,
                    if global_rng().gen_bool(0.5) {
                        5.0
                    } else {
                        -5.0
                    },
                ),
                vec2(
                    global_rng().gen_range(-1.0..1.0),
                    global_rng().gen_range(-1.0..1.0),
                )
                .normalize()
                    * ENEMY_SPEED,
            ));
        }
        if let Some(player) = &mut self.player {
            player.update(delta_time);
            if let Some(aim) = player.aim {
                if player.next_shoot < 0.0 {
                    let delta = aim - player.position;
                    if delta.len() > 0.1 {
                        player.next_shoot = PLAYER_SHOOT_INTERVAL;
                        assets.shoot.play();
                        self.player_shots
                            .push(Entity::new(player.position, delta.normalize() * SHOT_SPEED));
                    }
                }
            }
        }
        for unit in &mut self.enemies {
            while (unit.position - unit.target_position).len() < 0.1 {
                unit.target_position = vec2(
                    global_rng().gen_range(-WORLD_SIZE..WORLD_SIZE),
                    global_rng().gen_range(-WORLD_SIZE..WORLD_SIZE),
                );
            }
            unit.target_velocity = (unit.target_position - unit.position).normalize() * ENEMY_SPEED;
            unit.aim = self.player.as_ref().map(|player| player.position);
            if let Some(aim) = unit.aim {
                if unit.next_shoot < 0.0 {
                    let delta = aim - unit.position;
                    if delta.len() > 0.1 {
                        unit.next_shoot = ENEMY_SHOOT_INTERVAL;
                        assets.spit.play();
                        self.enemy_shots.push(Entity::new(
                            unit.position,
                            delta.normalize() * ENEMY_SHOT_SPEED,
                        ));
                    }
                }
            }
            unit.update(delta_time);
        }
        for shot in &mut self.player_shots {
            shot.update(delta_time);
            let initial_size = self.enemies.len();
            self.enemies
                .retain(|enemy| (enemy.position - shot.position).len() > 1.0);
            if self.enemies.len() != initial_size {
                assets.kill.play();
            }
        }
        for shot in &mut self.enemy_shots {
            shot.update(delta_time);
            if let Some(player) = &mut self.player {
                if (shot.position - player.position).len() < 1.0 {
                    self.player = None;
                    assets.noooo.play();
                }
            }
        }
    }
}

pub struct Game {
    framebuffer_size: Vec2<usize>,
    camera: Camera,
    renderer: Renderer,
    geng: Rc<Geng>,
    assets: Rc<Assets>,
    model: Model,
    time_multiplier: f32,
    history: VecDeque<Model>,
    current_time: f32,
    survive_time: f32,
    zzz: geng::SoundEffect,
    shh: geng::SoundEffect,
}

impl Game {
    pub fn new(geng: &Rc<Geng>, assets: &Rc<Assets>) -> Self {
        let mut result = Self {
            framebuffer_size: vec2(1, 1),
            camera: Camera::new(2.0 * WORLD_SIZE),
            geng: geng.clone(),
            renderer: Renderer::new(geng),
            assets: assets.clone(),
            model: Model::new(),
            time_multiplier: 0.0,
            history: VecDeque::new(),
            current_time: 0.0,
            survive_time: 0.0,
            zzz: {
                let mut effect = assets.zzz.effect();
                effect.set_volume(0.0);
                effect.play();
                effect
            },
            shh: {
                let mut effect = assets.shhh.effect();
                effect.set_volume(0.0);
                effect.play();
                effect
            },
        };
        result.reset();
        result
    }

    fn reset(&mut self) {
        self.model = Model::new();
        self.history = VecDeque::new();
        self.history.push_back(self.model.clone());
        self.current_time = 0.0;
        self.time_multiplier = 0.0;
        self.camera = Camera::new(2.0 * WORLD_SIZE);
    }

    fn draw_unit(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        unit: &Entity,
        texture: &ugli::Texture,
        color: Color<f32>,
    ) {
        self.renderer.draw(
            framebuffer,
            &self.camera,
            Mat4::translate(unit.position.extend(0.0))
                * Mat4::rotate_z(unit.rotation - f32::PI / 2.0)
                * Mat4::translate(vec3(-0.5, -0.5, 0.0)),
            texture,
            color,
        );
    }

    fn draw_model(&self, framebuffer: &mut ugli::Framebuffer, model: &Model) {
        self.renderer.draw(
            framebuffer,
            &self.camera,
            Mat4::scale(
                vec3(
                    WORLD_SIZE * self.assets.background.size().x as f32
                        / self.assets.background.size().y as f32,
                    WORLD_SIZE,
                    0.0,
                ) * 2.0,
            ) * Mat4::translate(vec3(-0.5, -0.5, 0.0)),
            &self.assets.background,
            Color::WHITE,
        );
        self.draw_unit(
            framebuffer,
            &Entity::new(vec2(0.0, 5.0), vec2(0.0, 0.0)),
            &self.assets.nest,
            Color::WHITE,
        );
        self.draw_unit(
            framebuffer,
            &Entity::new(vec2(0.0, -5.0), vec2(0.0, 0.0)),
            &self.assets.nest,
            Color::WHITE,
        );
        for shot in &model.enemy_shots {
            self.draw_unit(framebuffer, shot, &self.assets.enemy_shot, Color::RED);
        }
        for enemy in &model.enemies {
            self.draw_unit(framebuffer, enemy, &self.assets.body, Color::WHITE);
            self.draw_unit(
                framebuffer,
                enemy,
                &self.assets.wings[(self.current_time * 10.0) as usize % self.assets.wings.len()],
                Color::rgba(1.0, 1.0, 1.0, 0.5),
            );
        }
        for shot in &model.player_shots {
            self.draw_unit(framebuffer, shot, &self.assets.player_shot, Color::WHITE);
        }
        if let Some(player) = &self.model.player {
            let unit = player;
            self.renderer.draw(
                framebuffer,
                &self.camera,
                Mat4::translate(unit.position.extend(0.0))
                    * Mat4::rotate_z(unit.rotation - f32::PI / 2.0)
                    * Mat4::translate(vec3(0.0, -0.5, 0.0))
                    * Mat4::scale(vec3(1.0, 2.0 * unit.velocity.len() / PLAYER_SPEED, 1.0))
                    * Mat4::translate(vec3(-0.5, -0.7, 0.0)),
                &self.assets.fire[(self.current_time * 10.0) as usize % self.assets.fire.len()],
                Color::rgba(1.0, 1.0, 1.0, unit.velocity.len() / PLAYER_SPEED),
            );
            self.draw_unit(framebuffer, player, &self.assets.spaceship, Color::WHITE);
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let mut delta_time = delta_time as f32;
        let mut target_time_multiplier = 1.0;
        if self.geng.window().is_key_pressed(geng::Key::Space) {
            target_time_multiplier = FAST_FORWARD_MULTIPLIER;
        }
        if self.geng.window().is_key_pressed(geng::Key::R) {
            target_time_multiplier = -1.0;
        }
        self.time_multiplier += (target_time_multiplier - self.time_multiplier)
            .clamp(-TIME_ACCEL * delta_time, TIME_ACCEL * delta_time);
        delta_time *= self.time_multiplier;

        self.current_time += delta_time;
        if let Some(model) = self.history.front() {
            self.current_time = self.current_time.max(model.current_time);
        }
        self.current_time = self.current_time.max(0.0);

        if delta_time > 0.0 {
            if let Some(player) = &mut self.model.player {
                let mut target_velocity = vec2(0.0, 0.0);
                if self.geng.window().is_key_pressed(geng::Key::W) {
                    target_velocity.y += 1.0;
                }
                if self.geng.window().is_key_pressed(geng::Key::A) {
                    target_velocity.x -= 1.0;
                }
                if self.geng.window().is_key_pressed(geng::Key::S) {
                    target_velocity.y -= 1.0;
                }
                if self.geng.window().is_key_pressed(geng::Key::D) {
                    target_velocity.x += 1.0;
                }
                if target_velocity.len() > 1.0 {
                    target_velocity = target_velocity.normalize();
                }
                target_velocity *= PLAYER_SPEED;
                player.target_velocity = target_velocity;
                if self
                    .geng
                    .window()
                    .is_button_pressed(geng::MouseButton::Left)
                {
                    let aim = self.camera.screen_to_world(
                        self.framebuffer_size.map(|x| x as f32),
                        self.geng.window().mouse_pos().map(|x| x as f32),
                    );
                    player.aim = Some(aim);
                } else {
                    player.aim = None;
                }
                player.position.x = player.position.x.clamp(
                    -WORLD_SIZE * self.framebuffer_size.x as f32 / self.framebuffer_size.y as f32,
                    WORLD_SIZE * self.framebuffer_size.x as f32 / self.framebuffer_size.y as f32,
                );
                player.position.y = player.position.y.clamp(-WORLD_SIZE, WORLD_SIZE);
            }
            self.model.update(delta_time, &self.assets);
            self.model.current_time = self.current_time;
            self.history.push_back(self.model.clone());
            while let Some(model) = self.history.front() {
                if model.current_time < self.current_time - 10.0 {
                    self.history.pop_front();
                } else {
                    break;
                }
            }
        } else {
            while let Some(model) = self.history.back() {
                if model.current_time >= self.current_time {
                    self.model = self.history.pop_back().unwrap();
                } else {
                    break;
                }
            }
        }
        if let Some(player) = &self.model.player {
            self.survive_time = self.current_time;
            // self.shh
            //     .set_volume((player.velocity.len() / PLAYER_SPEED) as f64);
        } else {
            self.shh.set_volume(0.0);
        }
        if self.model.enemies.is_empty() {
            self.zzz.set_volume(0.0);
        } else {
            // self.zzz.set_volume(1.0);
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(Color::rgb(45.0 / 255.0, 35.0 / 255.0, 65.0 / 255.0)),
            None,
        );
        self.draw_model(framebuffer, &self.model);
        self.geng.default_font().draw_aligned(
            framebuffer,
            &format!("TIME SURVIVED: {:.1} seconds", self.survive_time),
            vec2(
                self.framebuffer_size.x as f32 / 2.0,
                self.framebuffer_size.y as f32 - 50.0,
            ),
            0.5,
            40.0,
            Color::WHITE,
        );
        if self.model.player.is_none() {
            self.geng.default_font().draw_aligned(
                framebuffer,
                "YOU DED, hold R to revert time",
                vec2(
                    self.framebuffer_size.x as f32 / 2.0,
                    self.framebuffer_size.y as f32 / 2.0,
                ),
                0.5,
                60.0,
                Color::WHITE,
            );
        }
        self.geng.default_font().draw_aligned(
            framebuffer,
            "WASD to move, LMB to shoot",
            vec2(self.framebuffer_size.x as f32 / 2.0, 50.0),
            0.5,
            40.0,
            Color::WHITE,
        );
        self.geng.default_font().draw_aligned(
            framebuffer,
            "hold Space to fast forward time",
            vec2(self.framebuffer_size.x as f32 / 2.0, 10.0),
            0.5,
            40.0,
            Color::WHITE,
        );
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyDown { key } => match key {
                geng::Key::Backspace => {
                    self.reset();
                }
                _ => {}
            },
            _ => {}
        }
    }
}
