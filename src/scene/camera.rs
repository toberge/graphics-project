use glm;
use sdl2::keyboard::Keycode;
use sdl2::sys::KeyCode;

// Copy-paste from assignments before Christmas
// Not intended for final use, should be useful for testing

pub struct Camera {
    pub x: f32, // Position in world coordinates
    pub y: f32,
    pub z: f32,
    pub yaw: f32,           // Angle around y-axis
    pub pitch: f32,         // Angle around x-axis
    perspective: glm::Mat4, // Cached version of the unchanging perspective matrix
}

impl Camera {
    pub fn new(screen_width: u32, screen_height: u32) -> Camera {
        Camera {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            perspective: glm::perspective(
                screen_width as f32 / screen_height as f32,
                0.5,
                1.0,
                1000.0,
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

    /// Assemble the global transformation matrix
    pub fn create_transformation(&self) -> glm::Mat4 {
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

    pub fn handle_keys(&mut self, keycode: Keycode, delta: f32) {
        let rot = self.just_rotation();
        match keycode {
            Keycode::A => {
                self.x -= rot[0] * delta;
                self.y -= rot[4] * delta;
                self.z -= rot[8] * delta;
            }
            Keycode::D => {
                self.x += rot[0] * delta;
                self.y += rot[4] * delta;
                self.z += rot[8] * delta;
            }
            // Use y column for up/down movement
            Keycode::E => {
                self.x += rot[1] * delta;
                self.y += rot[5] * delta;
                self.z += rot[9] * delta;
            }
            Keycode::Q => {
                self.x -= rot[1] * delta;
                self.y -= rot[5] * delta;
                self.z -= rot[9] * delta;
            }
            // Use z column for fwd/bkwd movement
            Keycode::W => {
                self.x -= rot[2] * delta;
                self.y -= rot[6] * delta;
                self.z -= rot[10] * delta;
            }
            Keycode::S => {
                self.x += rot[2] * delta;
                self.y += rot[6] * delta;
                self.z += rot[10] * delta;
            }
            //Keycode::Left => {
            //    cam.yaw -= delta_time;
            //}
            //Keycode::Right => {
            //    cam.yaw += delta_time;
            //}
            //Keycode::Up => {
            //    // TODO the angle might be the opposite here, actually
            //    cam.pitch -= delta_time;
            //}
            //Keycode::Down => {
            //    cam.pitch += delta_time;
            //}
            _ => {}
        }
    }
}