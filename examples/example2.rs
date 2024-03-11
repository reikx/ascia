extern crate ascia;

use std::cell::RefCell;
use std::f32::consts::PI;
use std::{env, thread};
use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;
use std::time::{Duration, Instant};
use ascia::ascia::camera::{SimpleBVHCamera, SimpleCamera};
use ascia::ascia::camera_gpu::GPUWrapper;
use ascia::ascia::color::ColorRGBf32;
use ascia::ascia::core::{AsciaEngine, FlatMaterial, LambertMaterial, Local, Material, ObjectNode, ObjectNodeAttribute, PresetCamera, PresetMaterial, PresetObjectNodeAttribute};
use ascia::ascia::core::PresetObjectNodeAttribute::Camera;
use ascia::ascia::lights::{PointLight};
use ascia::ascia::math::{Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;

struct FlashingFloor{
    size: f32,
    label: String,
    animation_start:Duration,
    animation_duration:Duration,
}

struct RollingCube{
    size: f32,
    label: String,
    animation_start:Duration,
    animation_duration:Duration,
    current_grid: (i64, i64),
    next_grid: (i64, i64),
}

impl RollingCube {
    fn new(size:f32, label: &str, animation_start: Duration, animation_duration: Duration) -> RollingCube{
        let current = ((rand::random::<f64>() * 128.0 - 64.0) as i64,(rand::random::<f64>() * 128.0 - 64.0) as i64);
        let next = if rand::random::<bool>() {
            if rand::random::<bool>() {
                (current.0 + 1i64, current.1)
            }
            else{
                (current.0 - 1i64, current.1)
            }
        } else {
            if rand::random::<bool>() {
                (current.0, current.1 + 1i64)
            }
            else{
                (current.0, current.1 - 1i64)
            }
        };
        return RollingCube{
            size,
            label: label.to_string(),
            animation_start: animation_start,
            animation_duration: animation_duration,
            current_grid: current,
            next_grid: next,
        };
    }

    fn update(&mut self, root: &mut ObjectNode<Local>, engine_time: &Duration){
        let mut self_objn = root.child_mut(&self.label).unwrap();
        if *engine_time > self.animation_start + self.animation_duration{
            self.decide_next();
        }

        let s = (*engine_time - self.animation_start).as_secs_f32() / self.animation_duration.as_secs_f32();
        let theta = PI * (0.75 - s * 0.5);

        let direction = Vec3{
            x: (self.next_grid.0 - self.current_grid.0) as f32,
            y: 0.0,
            z: (self.next_grid.1 - self.current_grid.1) as f32,
        }.normalize();
        let rotator = Quaternion::rotator(&Vec3{
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }, &direction);

        let relative = Vec3{
            x: (0.5 + f32::cos(theta) / f32::sqrt(2.0)) * self.size,
            y: f32::sin(theta) / f32::sqrt(2.0) * self.size,
            z: 0.0,
        };

        self_objn.position = Vec3{
            x: (self.current_grid.0 as f32 + 0.5) * self.size,
            y: 0.0,
            z: (self.current_grid.1 as f32 + 0.5) * self.size,
        } + rotator.rotate(&relative);
        //println!("{:?}", direction);
        //println!("{:?}", rotator.rotate(&relative));
        self_objn.direction = rotator * Quaternion::new(&Vec3{
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },theta - 0.75 * PI, 1.0);
    }

    fn decide_next(&mut self){
        loop{
            let diff = if rand::random::<bool>() {
                if rand::random::<bool>() {
                    (1i64, 0i64)
                }
                else{
                    (-1i64, 0i64)
                }
            } else {
                if rand::random::<bool>() {
                    (0i64, 1i64)
                }
                else{
                    (0i64, -1i64)
                }
            };
            if self.next_grid.0 + diff.0 != self.current_grid.0 && self.next_grid.1 + diff.1 != self.current_grid.1{
                self.current_grid = self.next_grid;
                self.next_grid = (self.next_grid.0 + diff.0, self.next_grid.1 + diff.1);
                break;
            }
        }
        self.animation_start += self.animation_duration;
    }
}

impl FlashingFloor {
    fn new(size:f32, label: &str, animation_start: Duration, animation_duration: Duration) -> FlashingFloor{
        return FlashingFloor{
            size,
            label: label.to_string(),
            animation_start: animation_start,
            animation_duration: animation_duration,
        };
    }
    fn update(&mut self, root: &mut ObjectNode<Local>, engine_time: &Duration) {
        let mut self_objn = root.child_mut(&self.label).unwrap();
        let s = if *engine_time <= self.animation_start{
            1.0
        } else if *engine_time <= self.animation_start + self.animation_duration{
            1.0 - (*engine_time - self.animation_start).as_secs_f32() / self.animation_duration.as_secs_f32()
        } else {
            0.0
        };


        for p in &mut self_objn.polygons{
            if let PresetMaterial::Lambert(m) = &mut p.material{
                m.color = ColorRGBf32{
                    r: s,
                    g: s,
                    b: s,
                };
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };
    let mut engine = AsciaEngine::new(width,height);

    let cube_size = 10.0;

    let mut cubes = vec![];
    for i in 0..32{
        let label = format!("cube {}", i);
        engine.genesis_local.add_child(ObjectNode::from(&label, PrimitiveGenerator::cube(cube_size, PresetMaterial::Lambert(LambertMaterial::default()))));
        cubes.push(RollingCube::new(cube_size, &label, engine.engine_time(), Duration::new(0, 500000000)));
    }


    let mut floors = VecDeque::new();
    for i in 0..32{
        let label = format!("floor {} {}", cubes[i].current_grid.0, cubes[i].current_grid.1);
        let mut f = ObjectNode::from(&label, PrimitiveGenerator::square(cube_size, PresetMaterial::Lambert(LambertMaterial::default())));
        f.position = Vec3{
            x: (cubes[i].current_grid.0 as f32 + 0.5) * cube_size,
            y: 0.0,
            z: (cubes[i].current_grid.1 as f32 + 0.5) * cube_size,
        };
        f.direction = Quaternion::new(&Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }, PI * 0.5, 1.0);
        engine.genesis_local.add_child(f);
        floors.push_back(FlashingFloor::new(cube_size, &label, engine.engine_time(), Duration::new(5,0)));
    }


    let mut cam_objn = ObjectNode::new("camera");
    cam_objn.position = Vec3{
        x: -50.0,
        y: 20.0,
        z: 0.0,
    };
    
    cam_objn.direction = Quaternion::new(&Vec3{
        x: 0.0,
        y: 0.0,
        z: 1.0,
    }, -cam_objn.position.y.atan2(-cam_objn.position.x), 1.0); 

    cam_objn.attribute = PresetCamera::SimpleGPU(GPUWrapper::<SimpleCamera>::generate(SimpleCamera::default(), &engine)).make_attribute_enum();
    cam_objn.attribute = SimpleBVHCamera::default().make_attribute_enum();
    cam_objn.attribute = SimpleCamera::default().make_attribute_enum();

    let mut cam_root = ObjectNode::new("camera root");
    cam_root.add_child(cam_objn);
    engine.genesis_local.add_child(cam_root);

    let mut light_objn = ObjectNode::new("light");
    light_objn.attribute = PointLight{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        power: 1.4,
    }.make_attribute_enum();
    light_objn.position = Vec3{
        x: -50.0,
        y: 150.0,
        z: 50.0,
    };
    engine.genesis_local.add_child(light_objn);

    let mut last_time = Instant::now();
    for _i in 0..65536 {
        engine.sync_engine_time();
        let engine_time = engine.engine_time();
        for c in &mut cubes{
            let label = format!("floor {} {}", c.current_grid.0, c.current_grid.1);
            if engine.genesis_local.child(&label).is_none(){
                let mut f = ObjectNode::from(&label, PrimitiveGenerator::square(cube_size, PresetMaterial::Lambert(LambertMaterial::default())));
                f.position = Vec3{
                    x: (c.current_grid.0 as f32 + 0.5) * cube_size,
                    y: 0.0,
                    z: (c.current_grid.1 as f32 + 0.5) * cube_size,
                };
                f.direction = Quaternion::new(&Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }, PI * 0.5, 1.0);
                engine.genesis_local.add_child(f);
                floors.push_back(FlashingFloor::new(cube_size, &label, engine.engine_time(), Duration::new(3,0)));
            }
            c.update(&mut engine.genesis_local, &engine_time);
        }

        for f in &mut floors{
            f.update(&mut engine.genesis_local, &engine_time);
            if f.animation_start + f.animation_duration < engine_time {
                engine.genesis_local.remove_child(f.label.clone());
            }
        }
        floors.retain(|f|{ f.animation_start + f.animation_duration >= engine_time });

        let cube_0_pos = engine.genesis_local.child("cube 0").unwrap().position;
        let mut camera_root = engine.genesis_local.child_mut("camera root").unwrap();
        camera_root.position = Vec3{
            x: cube_0_pos.x,
            y: cube_size * 0.5,
            z: cube_0_pos.z,
        };
        camera_root.direction = Quaternion::new(&Vec3{
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }, engine_time.as_secs_f32(), 1.0);

        engine.update_global_nodes();
        engine.render(&engine.genesis_global.child("camera root").unwrap().child("camera").unwrap());

        let mut dur = last_time.elapsed();
        if dur.as_millis() < 10{
            thread::sleep(Duration::from_millis(5));
            dur = last_time.elapsed();
        }
        println!("fps:{}      ",1000 / dur.as_millis());
        println!("current camera: {}     ", camera_info(&engine.genesis_global.child("camera root").unwrap().child("camera").unwrap().attribute));

        last_time = Instant::now();
    }
}

fn camera_info(attr: &Rc<RefCell<PresetObjectNodeAttribute>>) -> String{
    if let Camera(c) = &*RefCell::borrow(attr){
        return match c {
            PresetCamera::Simple(cam) => { format!("simple cpu {}x", cam.sampling_size) }
            PresetCamera::SimpleGPU(cam) => { format!("simple gpu {}x", cam.cpu_camera.sampling_size) }
            PresetCamera::SimpleBVH(cam) => { format!("simple cpu bvh {}x", cam.sampling_size) }
            PresetCamera::SimpleBVHGPU(cam) => { format!("simple gpu bvh {}x", cam.cpu_camera.sampling_size)}
        };
    }
    return "unknown".to_string();
}

