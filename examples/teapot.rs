extern crate ascia;

use std::cell::{RefCell, RefMut};
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
use ascia::ascia::core::CParticleMode::SPHERE;
use ascia::ascia::core::PresetObjectNodeAttribute::{Camera, Light};
use ascia::ascia::lights::PointLight;
use ascia::ascia::math::{Matrix33, Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };
    let mut engine = AsciaEngine::new(width,height);

    let cameras = available_cameras(&engine);
    let mut now_camera_index = 0usize;

    let mut cam_objn = ObjectNode::new("camera");
    cam_objn.direction = Quaternion::new(&Vec3{
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }, -PI * 0.5, 1.0);

    cam_objn.attribute = cameras[now_camera_index].clone();

    engine.genesis_local.add_child(cam_objn);

    let mut cube_objn = ObjectNode::from("cube",PrimitiveGenerator::cube(200.0, PresetMaterial::Lambert(Default::default())));
    cube_objn.position.z = 400.0;
    engine.genesis_local.add_child(cube_objn);

    let mut null_container = ObjectNode::new("null container");
    null_container.position = Vec3{
        x:0.0,
        y:0.0,
        z:100.0
    };

    for i in 0..1{
        let mut pot = ObjectNode::from(&format!("teapot {}", i), load_teapot("./examples/teapot_bezier1.tris.txt",&PresetMaterial::Lambert(
            LambertMaterial{
                color: ColorRGBf32{
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                },
                priority: 10,
            }
        )));

        pot.position = Vec3{
            x: 40.0 * i as f32 - 20.0,
            y: 0.0,
            z: 0.0,
        }.rotate_by(&Quaternion::new(&Vec3{
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },i as f32 * 0.5, 1.0));

        pot.direction = Quaternion::new(&Vec3{
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }, 0.0, 10.0);
        null_container.add_child(pot);
    }

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
    light_objn.position = Vec3{
        x: 0.0,
        y: 50.0,
        z: -100.0,
    };
    light_objn.attribute = PresetLight::Point(light).make_attribute_enum();

    engine.genesis_local.add_child(light_objn);

    let mut input = create_stdin_controller().unwrap();

    let mut last_time = Instant::now();

    for _i in 0..65536 {
        engine.sync_engine_time();

        move_camera(engine.genesis_local.child_mut("camera").unwrap(),&mut input,2.0,0.1, &cameras, &mut now_camera_index);
        engine.genesis_global = engine.genesis_local.generate_global_nodes();
        engine.render(engine.genesis_global.child("camera").unwrap());

        while last_time.elapsed().as_millis() < 10{
            thread::sleep(Duration::from_millis(2));
        }
        let mut dur = last_time.elapsed();

        println!("dur:{}      ",dur.as_millis());
        println!("fps:{}      ",1000 / dur.as_millis());
        println!("press [W][A][S][D] to move horizontally, [G][H] to move vertically, [I][J][K][L] to roll, [V] to change camera");
        println!("current camera: {}     ", camera_info(&cameras[now_camera_index]));
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
        // SimpleCamera::new(aov, 1).make_attribute(),
        SimpleBVHCamera::new(aov, 1).make_attribute_enum(),
        // SimpleCamera::new(aov, 3).make_attribute(),
        SimpleBVHCamera::new(aov, 3).make_attribute_enum(),
    ];
    if engine.wgpu_daq.is_some(){
        cameras.push(GPUWrapper::<SimpleCamera>::generate(SimpleCamera::new(aov, 1), &engine).make_attribute_enum());
        cameras.push(GPUWrapper::<SimpleBVHCamera>::generate(SimpleBVHCamera::new(aov, 1), &engine).make_attribute_enum());
        cameras.push(GPUWrapper::<SimpleCamera>::generate(SimpleCamera::new(aov, 3), &engine).make_attribute_enum());
        cameras.push(GPUWrapper::<SimpleBVHCamera>::generate(SimpleBVHCamera::new(aov, 3), &engine).make_attribute_enum());
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
            },rotation_speed,1.0);
        }
        else if *c == b'k'{
            cam_objn.direction = d * Quaternion::new(&Vec3{
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },-rotation_speed,1.0);
        }
        else if *c == b'j'{
            cam_objn.direction = Quaternion::new(&Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },-rotation_speed,1.0) * d;
        }
        else if *c == b'l'{
            cam_objn.direction = Quaternion::new(&Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },rotation_speed,1.0) * d;
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


// https://users.cs.utah.edu/~dejohnso/models/teapot.html
fn load_teapot(path:&str, material:&PresetMaterial) -> Vec<Polygon<Local>>{
    let mut polygons:Vec<Polygon<Local>> = vec![];

    if let Ok(f) = File::open(path){
        let mut reader = BufReader::new(f);
        let mut s = String::new();
        if let Ok(len) = reader.read_line(&mut s){
            if let Ok(num_polygon) = usize::from_str(&(s.trim())){
                let mut m = Matrix33{
                    v1: Default::default(),
                    v2: Default::default(),
                    v3: Default::default(),
                };
                for i in 0..num_polygon{
                    s.clear();
                    reader.read_line(&mut s);
                    let mut vs = s.split(" ");

                    m.v1.x = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v1.y = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v1.z = f32::from_str(vs.next().unwrap().trim()).unwrap();

                    s.clear();
                    reader.read_line(&mut s);
                    let mut vs = s.split(" ");
                    m.v2.x = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v2.y = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v2.z = f32::from_str(vs.next().unwrap().trim()).unwrap();

                    s.clear();
                    reader.read_line(&mut s);
                    let mut vs = s.split(" ");
                    m.v3.x = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v3.y = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v3.z = f32::from_str(vs.next().unwrap().trim()).unwrap();

                    reader.read_line(&mut s);

                    polygons.push(Polygon{
                        poses: m,
                        material: material.clone(),
                        _ph: Default::default(),
                    });
                }
            }
            else{
                return vec![];
            }
        }
        else{
            return vec![];
        }
    }
    return polygons;
}