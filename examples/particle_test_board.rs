extern crate ascia;

use std::cell::RefCell;
use std::f32::consts::PI;
use std::fs::{File, read};
use std::{env, io, thread};
use std::io::{BufRead, BufReader, Read, stdin};
use std::ops::Add;
use std::os::fd::{AsRawFd, FromRawFd};
use std::rc::Rc;
use std::str::FromStr;
use std::time::{Duration, Instant};
use ascia::ascia::camera::{NaiveBVH, SimpleBVHCamera, SimpleCamera};
use ascia::ascia::camera_gpu::GPUWrapper;
use ascia::ascia::color::{ColorRGBf32, ColorRGBu8};
use ascia::ascia::core::{AsciaEngine, CParticle, LambertMaterial, Local, Material, ObjectNode, ObjectNodeAttribute, Polygon, PresetCamera, PresetLight, PresetMaterial, PresetObjectNodeAttribute};
use ascia::ascia::core::CParticleMode::ARG;
use ascia::ascia::core::PresetObjectNodeAttribute::{Camera, Light};
use ascia::ascia::lights::{PointLight};
use ascia::ascia::math::{Matrix33, Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;

// https://users.cs.utah.edu/~dejohnso/models/teapot.html

fn main() {
    let args: Vec<String> = env::args().collect();
    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };
    let mut engine = AsciaEngine::new(width,height);

    let mut cameras = available_cameras(&engine);
    let mut now_camera_index = 0usize;

    let mut cam_objn = ObjectNode::new("camera");
    cam_objn.direction = Quaternion::new(&Vec3{
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }, -PI * 0.5);
    cam_objn.attribute = cameras[now_camera_index].clone();

    engine.genesis_local.add_child(cam_objn);

    let mut null_container = ObjectNode::new("null container");
    null_container.position = Vec3{
        x:0.0,
        y:0.0,
        z:100.0
    };

    let mut en1 = ObjectNode::new("en1");

    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    for i in 0..2048{
        en1.c_particles.push(CParticle{
            position: Vec3{
                x: 1.0 * (i % 80) as f32 - 40.0,
                y: 1.0 * (i / 80) as f32 - 40.0,
                z: 100.0,
            },
            velocity: Default::default(),
            color: ColorRGBf32{
                r: 1.0,
                g: if i % 2 == 0 { 0.0 } else { 1.0 },
                b: 1.0,
            },
            c: lorem.chars().nth(i % lorem.len()).unwrap(),
            threshold: PI / 400.0,
            mode: ARG,
            _ph: Default::default(),
        });
    }

    null_container.add_child(en1);
    engine.genesis_local.add_child(null_container);

    let light = PointLight{
        color: ColorRGBu8 {
            r:255,
            g:255,
            b:255
        }.into(),
        power: 1.0,
    };

    let mut light_objn = ObjectNode::new("light");
    light_objn.position.y = 100.0;
    light_objn.position.z = -100.0;
    light_objn.attribute = Rc::new(RefCell::new(Light(PresetLight::Point(light))));
    engine.genesis_local.add_child(light_objn);

    let rawfdstdin = stdin().as_raw_fd();

    let mut termi = termios::Termios::from_fd(rawfdstdin).unwrap();
    termi.c_lflag &= !termios::os::target::ICANON;
    termi.c_lflag &= !termios::os::target::ECHO;

    termi.c_cc[termios::os::target::VMIN] = 0;
    termi.c_cc[termios::os::target::VTIME] = 0;

    let resultermios:io::Result<()> = termios::tcsetattr(rawfdstdin, termios::os::target::TCSANOW, &mut termi);
    resultermios.unwrap();

    let mut filestdin = unsafe { File::from_raw_fd(rawfdstdin) };

    let mut input = create_stdin_controller().unwrap();

    let mut last_time = Instant::now();

    for _i in 0..65536 {
        engine.sync_engine_time();
        let engine_time = engine.engine_time();

        let d = 0.5 + f32::sin(engine_time.as_secs_f32() * 0.3 * PI) * 0.5;
        for i in 0..2048 {
            engine.genesis_local.child_mut("null container").unwrap().child_mut("en1").unwrap().c_particles[i].position = d * Vec3 {
                x: 1.0 * (i % 80) as f32 - 40.0,
                y: 1.0 * (i / 80) as f32 - 40.0,
                z: 100.0,
            } + (1.0 - d) * Vec3 {
                x: 80.0 * f32::cos(i as f32 * 0.1) - 40.0,
                y: 1.0 * i as f32 - 10.0,
                z: 100.0 + 80.0 * f32::sin(i as f32 * 0.1)
            };
        }

        move_camera(engine.genesis_local.child_mut("camera").unwrap(), &mut input, 2.0, 0.1, &cameras, &mut now_camera_index);
        engine.update_global_nodes();
        engine.render(engine.genesis_global.child("camera").unwrap());

        while last_time.elapsed().as_millis() < 20 {
            thread::sleep(Duration::from_millis(2));
        }
        let mut dur = last_time.elapsed();

        println!("dur:{}      ", dur.as_millis());
        println!("fps:{}      ", 1000 / dur.as_millis());
        println!("press [W][A][S][D] to move horizontally, [G][H] to move vertically, [I][J][K][L] to roll, [V] to change camera");
        println!("current camera: {}     ", camera_info(&engine.genesis_global.child("camera").unwrap().attribute));

        last_time = Instant::now();
    }
}


fn create_stdin_controller() -> Option<File>{
    let rawfdstdin = stdin().as_raw_fd();
    if let Ok(mut termi) = termios::Termios::from_fd(rawfdstdin){
        termi.c_lflag &= !termios::os::target::ICANON;
        termi.c_lflag &= !termios::os::target::ECHO;
        termi.c_cc[termios::os::target::VMIN] = 0;
        termi.c_cc[termios::os::target::VTIME] = 0;

        if let Ok(result) = termios::tcsetattr(rawfdstdin, termios::os::target::TCSANOW, &mut termi){
            return Some(unsafe { File::from_raw_fd(rawfdstdin) });
        }
        else{
            return None;
        }
    }
    else{
        return None;
    }
}

fn available_cameras(engine: &AsciaEngine) -> Vec<Rc<RefCell<PresetObjectNodeAttribute>>>{
    let aov = (PI / 3.0, PI / 4.0);

    let mut cameras = vec![
        SimpleCamera::new(aov, 1).make_attribute(),
        SimpleBVHCamera::new(aov, 1).make_attribute(),
        SimpleCamera::new(aov, 3).make_attribute(),
        SimpleBVHCamera::new(aov, 3).make_attribute(),
    ];
    if engine.wgpu_daq.is_some(){
        cameras.push(GPUWrapper::<SimpleCamera>::generate(SimpleCamera::new(aov, 1), &engine).make_attribute());
        cameras.push(GPUWrapper::<SimpleBVHCamera>::generate(SimpleBVHCamera::new(aov, 1), &engine).make_attribute());
        cameras.push(GPUWrapper::<SimpleCamera>::generate(SimpleCamera::new(aov, 3), &engine).make_attribute());
        cameras.push(GPUWrapper::<SimpleBVHCamera>::generate(SimpleBVHCamera::new(aov, 3), &engine).make_attribute());
    }
    return cameras;
}

fn move_camera(mut cam_objn: &mut ObjectNode<Local>, input: &mut File, velocity:f32, rotation_speed:f32, cameras: &Vec<Rc<RefCell<PresetObjectNodeAttribute>>>, mut now_camera_index: &mut usize){
    let mut v = vec![0;1];
    if let Err(e) = input.read_to_end(&mut v){
        println!("{}",e);
        panic!();
    }
    for c in v.iter(){
        let p = cam_objn.position.clone();
        let d = cam_objn.direction.clone();
        if *c == b'w'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: velocity,
                y: 0.0,
                z: 0.0,
            });
        }
        else if *c == b's'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: -velocity,
                y: 0.0,
                z: 0.0,
            });
        }
        else if *c == b'a'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: 0.0,
                y: 0.0,
                z: velocity,
            });
        }
        else if *c == b'd'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: 0.0,
                y: 0.0,
                z: -velocity,
            });
        }
        else if *c == b'g'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: 0.0,
                y: -velocity,
                z: 0.0,
            });
        }
        else if *c == b'h'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: 0.0,
                y: velocity,
                z: 0.0,
            });
        }
        else if *c == b'i'{
            cam_objn.direction = d * Quaternion::new(&Vec3{
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },rotation_speed);
        }
        else if *c == b'k'{
            cam_objn.direction = d * Quaternion::new(&Vec3{
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },-rotation_speed);
        }
        else if *c == b'j'{
            cam_objn.direction = Quaternion::new(&Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },-rotation_speed) * d;
        }
        else if *c == b'l'{
            cam_objn.direction = Quaternion::new(&Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },rotation_speed) * d;
        }
        else if *c == b'v'{
            *now_camera_index = if *now_camera_index + 1 >= cameras.len() { 0 } else { *now_camera_index + 1 };
            cam_objn.attribute = cameras[*now_camera_index].clone();
        }
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