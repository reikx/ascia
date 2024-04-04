use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{Read, stdin, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::ascia::camera::{SimpleBVHCamera, SimpleCamera};
use crate::ascia::core::{AsciaEngine, AsciaEnvironment, Local, ObjectNode, ObjectNodeAttribute, ObjectNodeAttributeDispatcher, PresetAsciaEnvironment, PresetCamera, PresetObjectNodeAttributeDispatcher, RenderChar};
use crate::ascia::math::{Quaternion, Vec3};

#[cfg(feature = "wgpu")]
use crate::ascia::camera_wgpu::GPUWrapper;
use crate::ascia::color::Color8bit;

pub fn available_preset_cameras() -> Vec<Rc<RefCell<Option<PresetObjectNodeAttributeDispatcher<PresetAsciaEnvironment>>>>>{
    let aov = (PI / 3.0, PI / 4.0);

    let mut cameras = vec![
        SimpleCamera::<PresetAsciaEnvironment>::new(aov, 1).make_attribute_enum().make_shared(),
        SimpleBVHCamera::<PresetAsciaEnvironment>::new(aov, 1).make_attribute_enum().make_shared(),
        SimpleCamera::<PresetAsciaEnvironment>::new(aov, 3).make_attribute_enum().make_shared(),
        SimpleBVHCamera::<PresetAsciaEnvironment>::new(aov, 3).make_attribute_enum().make_shared(),
    ];

    #[cfg(feature = "wgpu")]
    {
        if let Some(c) = GPUWrapper::<PresetAsciaEnvironment, SimpleCamera<PresetAsciaEnvironment>>::generate(SimpleCamera::new(aov, 1)){
            cameras.push(c.make_attribute_enum().make_shared());
        }
        if let Some(c) = GPUWrapper::<PresetAsciaEnvironment, SimpleCamera<PresetAsciaEnvironment>>::generate(SimpleCamera::new(aov, 3)){
            cameras.push(c.make_attribute_enum().make_shared());
        }
        if let Some(c) = GPUWrapper::<PresetAsciaEnvironment, SimpleBVHCamera<PresetAsciaEnvironment>>::generate(SimpleBVHCamera::new(aov, 1)){
            cameras.push(c.make_attribute_enum().make_shared());
        }
        if let Some(c) = GPUWrapper::<PresetAsciaEnvironment, SimpleBVHCamera<PresetAsciaEnvironment>>::generate(SimpleBVHCamera::new(aov, 3)){
            cameras.push(c.make_attribute_enum().make_shared());
        }
    }
    return cameras;
}

pub fn move_camera<E: AsciaEnvironment>(camera_velocity:f32, mut cam_objn: &mut ObjectNode<E, Local>, relative_direction: &Vec3){
    let p = cam_objn.position.clone();
    let d = cam_objn.direction.clone();
    cam_objn.position = p + d.rotate(&(relative_direction.normalize() * camera_velocity));
}

pub fn rotate_camera<E: AsciaEnvironment>(camera_rotation_speed: f32, mut cam_objn: &mut ObjectNode<E, Local>, relative_axis: &Vec3){
    let d = cam_objn.direction.clone();
    cam_objn.direction = d * Quaternion::new(relative_axis, camera_rotation_speed, 1.0);
}

pub fn preset_camera_info(attr: &Rc<RefCell<Option<PresetObjectNodeAttributeDispatcher<PresetAsciaEnvironment>>>>) -> String{
    if let Some(PresetObjectNodeAttributeDispatcher::Camera(c)) = &*RefCell::borrow(attr){
        #[cfg(not(feature = "wgpu"))]
        return match c {
            PresetCamera::SimpleCamera(cam) => { format!("simple cpu {}x", cam.sampling_size) }
            PresetCamera::SimpleBVHCamera(cam) => { format!("simple cpu bvh {}x", cam.sampling_size) }
        };

        #[cfg(feature = "wgpu")]
        return match c {
            PresetCamera::SimpleCamera(cam) => { format!("simple cpu {}x", cam.sampling_size) }
            PresetCamera::SimpleCameraGPU(cam) => { format!("simple gpu {}x", cam.cpu_camera.sampling_size) }
            PresetCamera::SimpleBVHCamera(cam) => { format!("simple cpu bvh {}x", cam.sampling_size) }
            PresetCamera::SimpleBVHCameraGPU(cam) => { format!("simple gpu bvh {}x", cam.cpu_camera.sampling_size)}
        };
    }
    return "unknown".to_string();
}

#[cfg(feature = "termios-controller")]
pub struct TermiosController<'a, E: AsciaEnvironment>{
    input: RefCell<File>,
    event_fn: Box<dyn FnMut(&u8, &mut AsciaEngine<E>) + 'a>,
}

#[cfg(feature = "termios-controller")]
impl<'a, E:AsciaEnvironment> TermiosController<'a, E>{
    pub fn generate(event_fn: impl FnMut(&u8, &mut AsciaEngine<E>) + 'a) -> Option<Self>{
        let rawfdstdin = stdin().as_raw_fd();
        if let Ok(mut termi) = termios::Termios::from_fd(rawfdstdin){
            termi.c_lflag &= !termios::os::target::ICANON;
            termi.c_lflag &= !termios::os::target::ECHO;
            termi.c_cc[termios::os::target::VMIN] = 0;
            termi.c_cc[termios::os::target::VTIME] = 0;

            if let Ok(result) = termios::tcsetattr(rawfdstdin, termios::os::target::TCSANOW, &mut termi){
                return Some(TermiosController{
                    input: RefCell::new( unsafe { File::from_raw_fd(rawfdstdin) }),
                    event_fn: Box::new(event_fn),
                });
            }
            else{
                return None;
            }
        }
        else{
            return None;
        }
    }
    pub fn input(&mut self, engine: &mut AsciaEngine<E>) -> Result<usize, String>{
        let mut v = vec![0;1];
        if let Err(e) = self.input.borrow_mut().read_to_end(&mut v){
            return Err(format!("termios-controller crashed, error: {:?}", e));
        }
        for c in v.iter(){
            (*self.event_fn)(c, engine);
        }
        return Ok(v.len());
    }
}


#[cfg(feature = "export")]
#[derive(Serialize, Deserialize)]
pub struct AsciaRenderedFrame{
    width: usize,
    height: usize,
    engine_time: u128,
    color_pallet: Vec<u8>,
    lines: Vec<Vec<String>>
}

#[cfg(feature = "export")]
impl AsciaRenderedFrame{
    pub fn generate(data: &Vec<Vec<RenderChar>>, engine_time: &Duration) -> Option<Self>{
        if data.is_empty() || data[0].is_empty() {
            return None;
        }
        let mut frame = AsciaRenderedFrame{
            width: data[0].len(),
            height: data.len(),
            engine_time: engine_time.as_millis(),
            color_pallet: vec![],
            lines: vec![],
        };
        let mut layers = HashMap::<u8, Vec<Vec<char>>>::new();
        for y in 0..frame.height{
            if data[y].len() != frame.width{
                return None;
            }
            for x in 0..frame.width{
                let rc = data[y][x];
                if rc.c == ' ' {
                    continue;
                }
                if !layers.contains_key(&Color8bit::from(rc.color).data){
                    layers.insert(Color8bit::from(rc.color).data, vec![vec![' '; frame.width]; frame.height]);
                }
                if let Some(cs) = layers.get_mut(&Color8bit::from(rc.color).data){
                    cs[y][x] = rc.c;
                }
            }
        }
        for layer in layers{
            frame.color_pallet.push(layer.0);
            let mut lines = Vec::with_capacity(frame.height);
            for line in layer.1{
                lines.push(line.iter().collect());
            }
            frame.lines.push(lines);
        }
        return Some(frame);
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: &P) -> Result<(), String>{
        if let Ok(mut file) = File::create(path){
            return match serde_json::to_string_pretty(self) {
                Ok(s) => {
                    match file.write_all(s.as_bytes()) {
                        Ok(_) => { Ok(()) }
                        Err(e) => { Err(e.to_string())}
                    }
                }
                Err(e) => {
                    Err(e.to_string())
                }
            };
        }
        return Err(format!("could not create file: {:?}", path.as_ref()));
    }
}

#[cfg(feature = "export")]
pub fn save_frames<P: AsRef<Path>>(frames: &Vec<AsciaRenderedFrame>, path: &P) -> Result<(), String>{
    if let Ok(mut file) = File::create(path){
        return match serde_json::to_string_pretty(frames) {
            Ok(s) => {
                match file.write_all(s.as_bytes()) {
                    Ok(_) => { Ok(()) }
                    Err(e) => { Err(e.to_string())}
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        };
    }
    return Err(format!("could not create file: {:?}", path.as_ref()));
}
