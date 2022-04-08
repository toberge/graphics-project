use std::f32::consts::PI;

use glm;
use glutin::event::VirtualKeyCode;

const FAR: f32 = 100.;

pub trait Camera {
    fn get_position(&self, time: f32) -> glm::Vec3;
    fn create_transformation(&mut self, time: f32, delta_time: f32) -> glm::Mat4;
    fn handle_keys(&mut self, keycode: &VirtualKeyCode, time: f32, delta_time: f32);
    fn handle_mouse(&mut self, delta_xy: (f32, f32));
}

pub enum AnimationStage {
    NONE,
    INTO,
    STATIONARY,
    OUT,
}

pub struct FirstPersonCamera {
    pub x: f32, // Position in world coordinates
    pub y: f32,
    pub z: f32,
    pub yaw: f32,           // Angle around y-axis
    pub pitch: f32,         // Angle around x-axis
    perspective: glm::Mat4, // Cached version of the unchanging perspective matrix
}

pub struct RevolvingCamera {
    pub origin: glm::Vec3,
    pub radius: f32,
    pub height: f32,
    perspective: glm::Mat4, // Cached version of the unchanging perspective matrix
    pub animation_stage: AnimationStage,
    pub start_time: f32,
    pub angle: f32,
    pub duration: f32,
    pub destination: glm::Vec3,
    pub destinations: Vec<glm::Vec3>,
}

impl RevolvingCamera {
    pub fn new(
        origin: glm::Vec3,
        radius: f32,
        height: f32,
        destinations: Vec<glm::Vec3>,
        screen_width: u32,
        screen_height: u32,
    ) -> RevolvingCamera {
        if destinations.len() < 8 {
            panic!("Invalid number of cameras");
        }
        RevolvingCamera {
            origin,
            radius,
            height,
            perspective: glm::perspective(
                screen_width as f32 / screen_height as f32,
                PI / 3.,
                0.5,
                FAR,
            ),
            animation_stage: AnimationStage::NONE,
            start_time: 0.,
            angle: 0.,
            duration: 1.,
            destination: destinations[0],
            destinations,
        }
    }

    fn start_if_needed(&mut self, time: f32, destination: glm::Vec3) {
        match self.animation_stage {
            AnimationStage::NONE => {
                self.destination = destination;
                self.animation_stage = AnimationStage::INTO;
                self.start_time = time;
            }
            AnimationStage::STATIONARY => {
                self.animation_stage = AnimationStage::OUT;
                self.start_time = time;
            }
            _ => {}
        };
    }
}

impl Camera for RevolvingCamera {
    fn get_position(&self, time: f32) -> glm::Vec3 {
        // TODO update this accordingly :)))
        let start = glm::vec3(
            self.radius * self.angle.cos(),
            self.height,
            self.radius * self.angle.sin(),
        );
        let end = self.destination
            + 2.0 * glm::vec3(self.destination.x, 0., self.destination.z).normalize();
        let animation_delta_time = time - self.start_time;
        let factor = match self.animation_stage {
            AnimationStage::NONE => 0.,
            AnimationStage::INTO => (animation_delta_time / self.duration).min(1.),
            AnimationStage::STATIONARY => 1.,
            AnimationStage::OUT => (1. - animation_delta_time / self.duration).min(1.),
        };
        glm::lerp(&start, &end, factor)
    }

    /// Assemble the global transformation matrix
    fn create_transformation(&mut self, time: f32, delta_time: f32) -> glm::Mat4 {
        // Time is either frozen or relative to when we last stopped viewing something
        match self.animation_stage {
            AnimationStage::NONE => {
                self.angle += delta_time;
            }
            _ => {}
        };
        let ground = glm::vec3(
            self.radius * self.angle.cos(),
            0.,
            self.radius * self.angle.sin(),
        );
        let eye = glm::vec3(ground.x, self.height, ground.z);
        let up = glm::cross(
            &(self.origin - eye),
            &glm::cross(&(self.origin - eye), &(ground - eye)),
        )
        .normalize();

        let stationary_eye = self.destination
            + 2.0 * glm::vec3(self.destination.x, 0., self.destination.z).normalize();

        let animation_delta_time = time - self.start_time;
        // Change state if necessary
        if animation_delta_time > self.duration {
            match self.animation_stage {
                AnimationStage::INTO => {
                    self.animation_stage = AnimationStage::STATIONARY;
                }
                AnimationStage::OUT => {
                    self.animation_stage = AnimationStage::NONE;
                }
                _ => {}
            }
            // Reset start time
            self.start_time = time;
        }

        let factor = match self.animation_stage {
            AnimationStage::NONE => 0.,
            AnimationStage::INTO => (animation_delta_time / self.duration).min(1.),
            AnimationStage::STATIONARY => 1.,
            AnimationStage::OUT => (1. - animation_delta_time / self.duration).min(1.),
        };

        // Interpolation inspired by this fine answer: https://stackoverflow.com/a/27192680
        let position = glm::mix(&eye, &stationary_eye, factor);
        let target = glm::slerp(&self.origin, &self.destination, factor);
        let alignment = glm::slerp(&up, &glm::vec3(0., 1., 0.), factor);

        let transformation = glm::look_at(&position, &target, &alignment);
        self.perspective * transformation
    }

    fn handle_keys(&mut self, keycode: &VirtualKeyCode, time: f32, delta_time: f32) {
        match keycode {
            VirtualKeyCode::Key1 => {
                self.start_if_needed(time, self.destinations[0]);
            }
            VirtualKeyCode::Key2 => {
                self.start_if_needed(time, self.destinations[1]);
            }
            VirtualKeyCode::Key3 => {
                self.start_if_needed(time, self.destinations[2]);
            }
            VirtualKeyCode::Key4 => {
                self.start_if_needed(time, self.destinations[3]);
            }
            VirtualKeyCode::Key5 => {
                self.start_if_needed(time, self.destinations[4]);
            }
            VirtualKeyCode::Key6 => {
                self.start_if_needed(time, self.destinations[5]);
            }
            VirtualKeyCode::Key7 => {
                self.start_if_needed(time, self.destinations[6]);
            }
            VirtualKeyCode::Key8 => {
                self.start_if_needed(time, self.destinations[7]);
            }
            _ => {}
        };
    }
    fn handle_mouse(&mut self, delta_xy: (f32, f32)) {}
}

impl FirstPersonCamera {
    pub fn new(screen_width: u32, screen_height: u32) -> FirstPersonCamera {
        FirstPersonCamera {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            perspective: glm::perspective(
                screen_width as f32 / screen_height as f32,
                PI / 3.,
                0.5,
                FAR,
            ),
        }
    }

    pub fn just_rotation(&self) -> glm::Mat4 {
        // Rotate along x according to pitch
        let pitch_rotation: glm::Mat4 = glm::rotation(self.pitch, &glm::vec3(1.0, 0.0, 0.0));
        // Rotate along y according to yaw
        let yaw_rotation: glm::Mat4 = glm::rotation(self.yaw, &glm::vec3(0.0, 1.0, 0.0));
        pitch_rotation * yaw_rotation * glm::identity()
    }
}

impl Camera for FirstPersonCamera {
    fn get_position(&self, time: f32) -> glm::Vec3 {
        glm::vec3(self.x, self.y, self.z)
    }

    /// Assemble the global transformation matrix
    fn create_transformation(&mut self, time: f32, delta_time: f32) -> glm::Mat4 {
        // Rotate along x according to pitch
        let pitch_rotation: glm::Mat4 = glm::rotation(self.pitch, &glm::vec3(1.0, 0.0, 0.0));
        // Rotate along y according to yaw
        let yaw_rotation: glm::Mat4 = glm::rotation(self.yaw, &glm::vec3(0.0, 1.0, 0.0));
        // Translate according to camera.xyz (moving the world, not the camera, thus inverse)
        let camera_translation: glm::Mat4 = glm::translation(&-glm::vec3(self.x, self.y, self.z));

        // Assemble the full view transformation
        self.perspective
                * pitch_rotation // Rotate the world along the camera's x axis
                                    // (which is rotated correctly now)
                * yaw_rotation // Rotate the world along the camera's y axis
                * camera_translation // Move the world so it looks like the camera has moved
                * glm::identity()
    }

    fn handle_keys(&mut self, keycode: &VirtualKeyCode, time: f32, delta: f32) {
        let rot = self.just_rotation();
        match keycode {
            VirtualKeyCode::A => {
                self.x -= rot[0] * delta;
                self.y -= rot[4] * delta;
                self.z -= rot[8] * delta;
            }
            VirtualKeyCode::D => {
                self.x += rot[0] * delta;
                self.y += rot[4] * delta;
                self.z += rot[8] * delta;
            }
            // Use y column for up/down movement
            VirtualKeyCode::E => {
                self.x += rot[1] * delta;
                self.y += rot[5] * delta;
                self.z += rot[9] * delta;
            }
            VirtualKeyCode::Q => {
                self.x -= rot[1] * delta;
                self.y -= rot[5] * delta;
                self.z -= rot[9] * delta;
            }
            // Use z column for fwd/bkwd movement
            VirtualKeyCode::W => {
                self.x -= rot[2] * delta;
                self.y -= rot[6] * delta;
                self.z -= rot[10] * delta;
            }
            VirtualKeyCode::S => {
                self.x += rot[2] * delta;
                self.y += rot[6] * delta;
                self.z += rot[10] * delta;
            }
            //VirtualKeyCode::Left => {
            //    cam.yaw -= delta_time;
            //}
            //VirtualKeyCode::Right => {
            //    cam.yaw += delta_time;
            //}
            //VirtualKeyCode::Up => {
            //    // TODO the angle might be the opposite here, actually
            //    cam.pitch -= delta_time;
            //}
            //VirtualKeyCode::Down => {
            //    cam.pitch += delta_time;
            //}
            _ => {}
        }
    }

    fn handle_mouse(&mut self, delta_xy: (f32, f32)) {
        self.yaw += delta_xy.0;
        self.pitch += delta_xy.1;
    }
}
