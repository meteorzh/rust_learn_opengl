use std::time::Duration;

use glium::glutin::event::MouseScrollDelta;

use crate::camera::Camera;


/// 鼠标事件处理器特征
pub trait MouseEventHandler {

    /// 处理鼠标移动事件
    fn handle_motion(&mut self, delta: (f64, f64));

    /// 处理鼠标滚轮事件
    fn handle_wheel(&mut self, delta: MouseScrollDelta);

    /// 临时解决问题，需要优化类结构
    fn update_camera(&mut self, camera: &mut Camera, frame_delta: Duration);
}

/// 鼠标交互特征
pub trait MouseInteract {
    
    /// 处理鼠标移动事件
    fn motion_interact(&mut self, delta: (f64, f64));

    /// 处理鼠标滚轮事件
    fn wheel_interact(&mut self, delta: MouseScrollDelta);
}