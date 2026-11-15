//! minecraft-rs binary: window, input, and the game loop.

use minecraft_rs::block::Block;
use minecraft_rs::game::camera;
use minecraft_rs::game::player::{MoveInput, Player};
use minecraft_rs::game::raycast;
use minecraft_rs::render::Renderer;
use minecraft_rs::world::mesh::build_mesh;
use minecraft_rs::world::World;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const SEED: u32 = 1337;
const RENDER_DIST: i32 = 6;
const REACH: f32 = 6.0;
const MOUSE_SENS: f32 = 0.0022;
const PALETTE: [Block; 9] = [
    Block::Grass,
    Block::Dirt,
    Block::Stone,
    Block::Sand,
    Block::Wood,
    Block::Leaves,
    Block::Water,
    Block::Planks,
    Block::Bedrock,
];

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    world: World,
    player: Player,
    keys: HashSet<KeyCode>,
    selected: usize,
    last: Instant,
    grabbed: bool,
}

impl App {
    fn new() -> App {
        App {
            window: None,
            renderer: None,
            world: World::new(SEED),
            player: Player::new(glam::Vec3::new(0.5, 50.0, 0.5)),
            keys: HashSet::new(),
            selected: 0,
            last: Instant::now(),
            grabbed: false,
        }
    }

    fn set_grab(&mut self, grab: bool) {
        if let Some(w) = &self.window {
            if grab {
                w.set_cursor_grab(CursorGrabMode::Locked)
                    .or_else(|_| w.set_cursor_grab(CursorGrabMode::Confined))
                    .ok();
            } else {
                w.set_cursor_grab(CursorGrabMode::None).ok();
            }
            w.set_cursor_visible(!grab);
        }
        self.grabbed = grab;
    }

    fn input(&self) -> MoveInput {
        let k = &self.keys;
        MoveInput {
            forward: (k.contains(&KeyCode::KeyW) as i32 - k.contains(&KeyCode::KeyS) as i32) as f32,
            strafe: (k.contains(&KeyCode::KeyD) as i32 - k.contains(&KeyCode::KeyA) as i32) as f32,
            jump: k.contains(&KeyCode::Space),
            down: k.contains(&KeyCode::ShiftLeft),
        }
    }

    fn break_block(&mut self) {
        if let Some(hit) = raycast::raycast(&self.world, self.player.eye(), self.player.look_dir(), REACH)
            && self.world.block_at(hit.block.x, hit.block.y, hit.block.z) != Block::Bedrock
        {
            self.world.set_block(hit.block.x, hit.block.y, hit.block.z, Block::Air);
        }
    }

    fn place_block(&mut self) {
        if let Some(hit) = raycast::raycast(&self.world, self.player.eye(), self.player.look_dir(), REACH) {
            let (x, y, z) = (
                hit.block.x + hit.normal.x,
                hit.block.y + hit.normal.y,
                hit.block.z + hit.normal.z,
            );
            let target = self.world.block_at(x, y, z);
            if matches!(target, Block::Air | Block::Water) && !self.player.overlaps_voxel(x, y, z) {
                self.world.set_block(x, y, z, PALETTE[self.selected]);
            }
        }
    }

    fn tick(&mut self) {
        let dt = self.last.elapsed().as_secs_f32().min(0.05);
        self.last = Instant::now();

        self.player.step(&self.world, self.input(), dt);
        self.world.ensure_chunks(self.player.pos.x, self.player.pos.z, RENDER_DIST);

        let dirty = self.world.take_dirty();
        let renderer = self.renderer.as_mut().unwrap();
        for (cx, cz) in dirty {
            if self.world.chunks.contains_key(&(cx, cz)) {
                let mesh = build_mesh(&self.world, cx, cz);
                renderer.upload_chunk(cx, cz, &mesh);
            }
        }

        let eye = self.player.eye();
        let mvp = camera::view_proj(eye, self.player.look_dir(), renderer.aspect());
        renderer.render(&mvp, eye);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title("minecraft-rs")
                    .with_inner_size(winit::dpi::LogicalSize::new(1280, 720)),
            )
            .unwrap(),
        );
        self.renderer = Some(pollster::block_on(Renderer::new(window.clone(), SEED)));
        self.window = Some(window);

        // Initial world + spawn point.
        self.world.ensure_chunks(0.0, 0.0, RENDER_DIST);
        let surface = self.world.surface_height(0, 0).unwrap_or(30);
        self.player.pos.y = surface as f32 + 2.0;
        for (cx, cz) in self.world.take_dirty() {
            if self.world.chunks.contains_key(&(cx, cz)) {
                let mesh = build_mesh(&self.world, cx, cz);
                self.renderer.as_mut().unwrap().upload_chunk(cx, cz, &mesh);
            }
        }
        self.set_grab(true);
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event
            && self.grabbed
        {
            self.player.yaw -= dx as f32 * MOUSE_SENS;
            self.player.pitch = (self.player.pitch - dy as f32 * MOUSE_SENS).clamp(-1.55, 1.55);
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else { return };
                match event.state {
                    ElementState::Pressed => {
                        if !event.repeat {
                            let slot = match code {
                                KeyCode::Digit1 => Some(0),
                                KeyCode::Digit2 => Some(1),
                                KeyCode::Digit3 => Some(2),
                                KeyCode::Digit4 => Some(3),
                                KeyCode::Digit5 => Some(4),
                                KeyCode::Digit6 => Some(5),
                                KeyCode::Digit7 => Some(6),
                                KeyCode::Digit8 => Some(7),
                                KeyCode::Digit9 => Some(8),
                                _ => None,
                            };
                            if let Some(n) = slot {
                                self.selected = n.min(PALETTE.len() - 1);
                            }
                            match code {
                                KeyCode::KeyF => self.player.flying = !self.player.flying,
                                KeyCode::Escape => self.set_grab(false),
                                _ => {}
                            }
                            if let Some(w) = &self.window {
                                w.set_title(&format!(
                                    "minecraft-rs | block: {:?}{}",
                                    PALETTE[self.selected],
                                    if self.player.flying { " | flying" } else { "" }
                                ));
                            }
                        }
                        self.keys.insert(code);
                    }
                    ElementState::Released => {
                        self.keys.remove(&code);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state != ElementState::Pressed {
                    return;
                }
                if !self.grabbed {
                    self.set_grab(true);
                    return;
                }
                match button {
                    MouseButton::Left => self.break_block(),
                    MouseButton::Right => self.place_block(),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => self.tick(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

fn main() {
    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Poll);
    el.run_app(&mut App::new()).unwrap();
}
